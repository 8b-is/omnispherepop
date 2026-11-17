#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

/// Pack four trits per byte, low bits first; tail bits are zero.
pub fn pack(values: &[Trit]) -> Vec<u8> {
    values
        .chunks(4)
        .map(|chunk| {
            chunk.iter().enumerate().fold(0u8, |byte, (i, t)| {
                let code = match t {
                    Trit::Zero => 0,
                    Trit::Pos => 1,
                    Trit::Neg => 2,
                };
                byte | (code << (2 * i))
            })
        })
        .collect()
}

/// Validate length, reserved codes, and canonical padding before decoding.
pub fn unpack(bytes: &[u8], len: usize) -> Result<Vec<Trit>, String> {
    let expected = len / 4 + usize::from(len % 4 != 0);
    if bytes.len() != expected {
        return Err("packed length mismatch".into());
    }
    if len % 4 != 0 && bytes.last().is_some_and(|b| b >> (2 * (len % 4)) != 0) {
        return Err("nonzero tail padding".into());
    }
    (0..len)
        .map(|i| match (bytes[i / 4] >> (2 * (i % 4))) & 3 {
            0 => Ok(Trit::Zero),
            1 => Ok(Trit::Pos),
            2 => Ok(Trit::Neg),
            _ => Err("reserved trit code".into()),
        })
        .collect()
}
