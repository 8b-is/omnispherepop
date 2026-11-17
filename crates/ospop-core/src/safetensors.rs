//! Minimal, strict safetensors reader.
//!
//! Format: 8-byte little-endian header length, a UTF-8 JSON header, then a
//! contiguous byte buffer. Each tensor entry names a dtype, a shape, and a
//! `[begin, end)` byte range into that buffer.
//!
//! Scope is deliberately narrow: dense `F32`, `F16`, and `BF16` tensors with
//! row-major shapes. Everything else — quantized block encodings, shared or
//! overlapping storage, non-finite values in a float payload — is rejected with
//! the offending key, so a caller can never half-load a file.
//!
//! The reader returns `Vec<f32>` per tensor; it does not retain the file's
//! original dtype, because downstream quantization works in f32. Callers that
//! need the raw bytes should read the buffer themselves.

use std::collections::{BTreeMap, BTreeSet};

/// One tensor as read from the file.
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub name: String,
    pub dtype: DType,
    pub shape: Vec<usize>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DType {
    F32,
    F16,
    Bf16,
}

impl DType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "F32" => Some(Self::F32),
            "F16" => Some(Self::F16),
            "BF16" => Some(Self::Bf16),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::F32 => "F32",
            Self::F16 => "F16",
            Self::Bf16 => "BF16",
        }
    }

    fn width(self) -> usize {
        match self {
            Self::F32 => 4,
            Self::F16 | Self::Bf16 => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kind {
    Dense { dtype: DType, shape: Vec<usize> },
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    kind: Kind,
    begin: usize,
    end: usize,
}

/// Parse the header and validate the tensor table, but do not decode payloads.
/// Splitting this out means a malformed header fails before any allocation.
fn parse_header(bytes: &[u8]) -> Result<(BTreeMap<String, Entry>, usize), String> {
    if bytes.len() < 8 {
        return Err("file shorter than the 8-byte header length".into());
    }
    let mut len_bytes = [0u8; 8];
    len_bytes.copy_from_slice(&bytes[..8]);
    let header_len = u64::from_le_bytes(len_bytes) as usize;
    let header_start = 8usize;
    let header_end = header_start
        .checked_add(header_len)
        .ok_or("header length overflow")?;
    if header_end > bytes.len() {
        return Err(format!(
            "header claims {header_len} bytes, file has {}",
            bytes.len() - 8
        ));
    }
    let header = std::str::from_utf8(&bytes[header_start..header_end])
        .map_err(|_| "header is not valid UTF-8".to_string())?;
    let json: serde_json::Value =
        serde_json::from_str(header).map_err(|e| format!("header is not JSON: {e}"))?;
    let obj = json.as_object().ok_or("header is not a JSON object")?;

    let mut entries = BTreeMap::new();
    for (name, spec) in obj {
        if name == "__metadata__" {
            continue;
        }
        if name.is_empty() {
            return Err("empty tensor name".into());
        }
        let spec = spec
            .as_object()
            .ok_or_else(|| format!("{name}: entry is not an object"))?;
        let dtype = spec
            .get("dtype")
            .and_then(|d| d.as_str())
            .ok_or_else(|| format!("{name}: missing dtype"))?;
        let shape: Vec<usize> = spec
            .get("shape")
            .and_then(|s| s.as_array())
            .ok_or_else(|| format!("{name}: missing shape"))?
            .iter()
            .map(|v| {
                v.as_u64()
                    .map(|n| n as usize)
                    .ok_or_else(|| format!("{name}: shape is not a list of integers"))
            })
            .collect::<Result<_, _>>()?;
        let offsets = spec
            .get("data_offsets")
            .and_then(|o| o.as_array())
            .ok_or_else(|| format!("{name}: missing data_offsets"))?;
        if offsets.len() != 2 {
            return Err(format!("{name}: data_offsets must have two entries"));
        }
        let begin = offsets[0]
            .as_u64()
            .ok_or_else(|| format!("{name}: offset not an integer"))? as usize;
        let end = offsets[1]
            .as_u64()
            .ok_or_else(|| format!("{name}: offset not an integer"))? as usize;
        if begin > end {
            return Err(format!("{name}: offsets run backwards"));
        }
        let kind = match DType::parse(dtype) {
            Some(dtype) => Kind::Dense { dtype, shape },
            None => Kind::Other(dtype.to_string()),
        };
        entries.insert(name.clone(), Entry { kind, begin, end });
    }
    Ok((entries, header_end))
}

fn decode(dtype: DType, raw: &[u8]) -> Result<Vec<f32>, String> {
    if raw.len() % dtype.width() != 0 {
        return Err(format!(
            "payload length {} not a multiple of {}",
            raw.len(),
            dtype.width()
        ));
    }
    match dtype {
        DType::F32 => Ok(raw
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()),
        DType::F16 => Ok(raw
            .chunks_exact(2)
            .map(|c| {
                let bits = u16::from_le_bytes([c[0], c[1]]);
                // Half to single: exact for normals, subnormals, and infinities.
                let sign = ((bits >> 15) & 1) as u32;
                let exp = ((bits >> 10) & 0x1f) as u32;
                let mant = (bits & 0x3ff) as u32;
                let out = if exp == 0 {
                    if mant == 0 {
                        sign << 31
                    } else {
                        // Subnormal: normalize by shifting until the leading one lands.
                        let mut e = -1i32;
                        let mut m = mant;
                        while m & 0x400 == 0 {
                            m <<= 1;
                            e -= 1;
                        }
                        let exp32 = (127 - 15 + 1 + e) as u32;
                        (sign << 31) | (exp32 << 23) | ((m & 0x3ff) << 13)
                    }
                } else if exp == 0x1f {
                    (sign << 31) | (0xff << 23) | (mant << 13)
                } else {
                    (sign << 31) | ((exp + 127 - 15) << 23) | (mant << 13)
                };
                f32::from_bits(out)
            })
            .collect()),
        DType::Bf16 => Ok(raw
            .chunks_exact(2)
            .map(|c| f32::from_bits((u16::from_le_bytes([c[0], c[1]]) as u32) << 16))
            .collect()),
    }
}

/// Read every dense tensor in a safetensors file.
///
/// Rejects: unsupported dtypes (reported, not silently skipped), shapes whose
/// element count disagrees with the byte range, overlapping byte ranges, ranges
/// outside the buffer, and non-finite values in a decoded float payload.
pub fn read(bytes: &[u8]) -> Result<Vec<Tensor>, String> {
    let (entries, data_start) = parse_header(bytes)?;
    let data = &bytes[data_start..];

    let mut checked: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut out = Vec::with_capacity(entries.len());
    for (name, entry) in &entries {
        if entry.end > data.len() {
            return Err(format!(
                "{name}: byte range {}..{} outside buffer",
                entry.begin, entry.end
            ));
        }
        if !checked.insert((entry.begin, entry.end)) {
            return Err(format!("{name}: overlaps another tensor's byte range"));
        }
        let (dtype, shape) = match &entry.kind {
            Kind::Dense { dtype, shape } => (*dtype, shape),
            Kind::Other(other) => return Err(format!("{name}: unsupported dtype {other}")),
        };
        let elements: usize = shape.iter().product();
        let expected = elements
            .checked_mul(dtype.width())
            .ok_or_else(|| format!("{name}: byte count overflow"))?;
        if expected != entry.end - entry.begin {
            return Err(format!(
                "{name}: shape {shape:?} needs {expected} bytes, range holds {}",
                entry.end - entry.begin
            ));
        }
        let values = decode(dtype, &data[entry.begin..entry.end])?;

        if values.iter().any(|v| !v.is_finite()) {
            return Err(format!("{name}: non-finite value in payload"));
        }
        out.push(Tensor {
            name: name.clone(),
            dtype,
            shape: shape.clone(),
            values,
        });
    }
    Ok(out)
}

/// Convenience: tensor name to flat values, for loading into a snapshot.
pub fn into_map(tensors: Vec<Tensor>) -> BTreeMap<String, Vec<f32>> {
    tensors.into_iter().map(|t| (t.name, t.values)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_f32(tensors: &[(&str, &[f32])]) -> Vec<u8> {
        let mut header = serde_json::Map::new();
        let mut payload = Vec::new();
        for (name, values) in tensors {
            let begin = payload.len();
            for v in *values {
                payload.extend_from_slice(&v.to_le_bytes());
            }
            header.insert(
                (*name).to_string(),
                serde_json::json!({
                    "dtype": "F32",
                    "shape": [values.len()],
                    "data_offsets": [begin, payload.len()],
                }),
            );
        }
        let header_bytes = serde_json::to_vec(&serde_json::Value::Object(header)).unwrap();
        let mut out = (header_bytes.len() as u64).to_le_bytes().to_vec();
        out.extend_from_slice(&header_bytes);
        out.extend_from_slice(&payload);
        out
    }

    #[test]
    fn round_trips_f32() {
        let bytes = write_f32(&[("a", &[1.0, -2.5]), ("b", &[0.0])]);
        let got = read(&bytes).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].values, vec![1.0, -2.5]);
        assert_eq!(got[1].shape, vec![1]);
        assert_eq!(into_map(got).get("a").unwrap(), &vec![1.0, -2.5]);
    }

    #[test]
    fn decodes_f16_and_bf16() {
        // 1.0 in half is 0x3c00; 1.0 in bf16 is 0x3f80.
        let mut bytes = Vec::new();
        let mut header = serde_json::Map::new();
        header.insert(
            "h".into(),
            serde_json::json!({ "dtype": "F16", "shape": [1], "data_offsets": [0, 2] }),
        );
        header.insert(
            "b".into(),
            serde_json::json!({ "dtype": "BF16", "shape": [1], "data_offsets": [2, 4] }),
        );
        let hb = serde_json::to_vec(&serde_json::Value::Object(header)).unwrap();
        bytes.extend_from_slice(&(hb.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&hb);
        bytes.extend_from_slice(&0x3c00u16.to_le_bytes());
        bytes.extend_from_slice(&0x3f80u16.to_le_bytes());
        // Entries come back sorted by name, so look them up rather than by index.
        let got = into_map(read(&bytes).unwrap());
        assert_eq!(got.get("h").unwrap(), &vec![1.0]);
        assert_eq!(got.get("b").unwrap(), &vec![1.0]);
        let decoded = read(&bytes).unwrap();
        let dtypes: BTreeMap<&str, DType> =
            decoded.iter().map(|t| (t.name.as_str(), t.dtype)).collect();
        assert_eq!(dtypes.get("h"), Some(&DType::F16));
        assert_eq!(dtypes.get("b"), Some(&DType::Bf16));
    }

    #[test]
    fn malformed_inputs_are_rejected() {
        assert!(read(&[0u8; 4]).is_err());
        let mut short = (3u64).to_le_bytes().to_vec();
        short.extend_from_slice(b"{}");
        assert!(read(&short).is_err(), "header length beyond EOF");

        // Unsupported dtype must name the key.
        let mut header = serde_json::Map::new();
        header.insert(
            "q".into(),
            serde_json::json!({ "dtype": "Q4_0", "shape": [8], "data_offsets": [0, 8] }),
        );
        let hb = serde_json::to_vec(&serde_json::Value::Object(header)).unwrap();
        let mut bytes = (hb.len() as u64).to_le_bytes().to_vec();
        bytes.extend_from_slice(&hb);
        bytes.extend_from_slice(&[0u8; 8]);
        let err = read(&bytes).unwrap_err();
        assert!(err.contains('q') && err.contains("Q4_0"), "{err}");

        // Shape that disagrees with the byte range.
        let bytes = write_f32(&[("a", &[1.0, 2.0])]);
        let mut tampered = bytes.clone();
        let hlen = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
        let h: serde_json::Value = serde_json::from_slice(&bytes[8..8 + hlen]).unwrap();
        let mut obj = h.as_object().unwrap().clone();
        obj.get_mut("a").unwrap()["shape"] = serde_json::json!([3]);
        let new_header = serde_json::to_vec(&serde_json::Value::Object(obj.clone())).unwrap();
        tampered.truncate(0);
        tampered.extend_from_slice(&(new_header.len() as u64).to_le_bytes());
        tampered.extend_from_slice(&new_header);
        tampered.extend_from_slice(&bytes[8 + hlen..]);
        assert!(read(&tampered).unwrap_err().contains("needs 12 bytes"));

        // Non-finite payload is refused rather than propagated.
        let bytes = write_f32(&[("n", &[f32::NAN])]);
        assert!(read(&bytes).unwrap_err().contains("non-finite"));
    }
}
