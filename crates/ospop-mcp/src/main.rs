#![forbid(unsafe_code)]
//! `ospop-mcp` — MCP server exposing Omnispherepop operations as agent tools.
//!
//! stdio JSON-RPC, no runtime dependencies beyond serde_json: the eight tools
//! all build a `Program` and call the same runtime the CLI uses.

use ospop_runtime::{LensView, Program, Runtime, Scheme, Threshold};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(serde::Deserialize)]
struct Request {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

struct Tool {
    name: &'static str,
    description: &'static str,
    input: Value,
}

fn tools() -> Vec<Tool> {
    let tensor = || json!({ "type": "string", "description": "tensor name from ospop_inspect" });
    vec![
        Tool { name: "ospop_load_demo", description: "Load the bundled demo snapshot and return its identity.", input: json!({ "type": "object", "properties": {} }) },
        Tool { name: "ospop_load_safetensors", description: "Load a safetensors file (F32/F16/BF16 dense tensors) from a local path and return its identity.", input: json!({ "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] }) },
        Tool { name: "ospop_inspect", description: "Snapshot identity and tensor shapes.", input: json!({ "type": "object", "properties": {} }) },
        Tool {
            name: "ospop_quantize",
            description: "Ternary PTQ with per-group scales; returns sparsity, packed size, and reconstruction error.",
            input: json!({ "type": "object", "properties": {
                "tensor": tensor(),
                "group_size": { "type": "integer", "default": 128 },
                "threshold": { "type": "number", "description": "omit for auto (0.7 * mean |w|)" },
                "scheme": { "type": "string", "enum": ["ternary", "qdelta"], "default": "ternary" },
                "level": { "type": "integer", "description": "qdelta A6P detail level, 0..14" }
            }, "required": ["tensor"] }),
        },
        Tool {
            name: "ospop_lens",
            description: "Render one tensor through a view: ternary, heatmap, graph, diff, sparsity-histogram.",
            input: json!({ "type": "object", "properties": { "tensor": tensor(), "view": { "type": "string", "enum": ["ternary", "heatmap", "graph", "diff", "sparsity-histogram"] } }, "required": ["tensor", "view"] }),
        },
        Tool { name: "ospop_prune", description: "Report reachable tensors from a root vertex in the snapshot ultragraph.", input: json!({ "type": "object", "properties": { "root": tensor() }, "required": ["root"] }) },
        Tool { name: "ospop_diff", description: "Diff a tensor against a scaled copy and prove the patch reverses.", input: json!({ "type": "object", "properties": { "tensor": tensor(), "scale": { "type": "number", "default": 0.5 } }, "required": ["tensor"] }) },
        Tool { name: "ospop_explain", description: "Plain-language lesson for a topic such as ternary, ultragraph, delta, mcp.", input: json!({ "type": "object", "properties": { "topic": { "type": "string" } }, "required": ["topic"] }) },
        Tool { name: "ospop_learn", description: "List the curriculum or run one lesson lab with its checkpoint verdict.", input: json!({ "type": "object", "properties": { "lesson": { "type": "string" } } }) },
    ]
}

fn call_tool(rt: &mut Runtime, name: &str, args: &Value) -> Result<Value, String> {
    let s = |k: &str, d: &str| args.get(k).and_then(Value::as_str).unwrap_or(d).to_string();
    let program = match name {
        "ospop_load_demo" => {
            rt.load_demo()?;
            return rt.execute(&Program::Inspect);
        }
        "ospop_load_safetensors" => {
            let path = s("path", "");
            let bytes = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
            let n = rt.load_safetensors(&bytes)?;
            return rt.execute(&Program::Inspect).map(|mut v| {
                v["loaded_tensors"] = json!(n);
                v
            });
        }
        "ospop_inspect" => Program::Inspect,
        "ospop_quantize" => Program::Quantize {
            tensor: s("tensor", "layer.weight"),
            group_size: args
                .get("group_size")
                .and_then(Value::as_u64)
                .unwrap_or(128) as usize,
            threshold: args
                .get("threshold")
                .and_then(Value::as_f64)
                .map(|t| Threshold::Fixed(t as f32))
                .unwrap_or(Threshold::Auto),
            scheme: match s("scheme", "ternary").as_str() {
                "qdelta" => Scheme::Qdelta {
                    level: args.get("level").and_then(Value::as_u64).unwrap_or(6) as usize,
                },
                _ => Scheme::Ternary,
            },
        },
        "ospop_lens" => {
            let view = s("view", "ternary");
            let view = LensView::parse(&view).ok_or_else(|| format!("unknown view {view}"))?;
            Program::Lens {
                tensor: s("tensor", "layer.weight"),
                view,
            }
        }
        "ospop_prune" => Program::Prune {
            root: s("root", "layer.weight"),
        },
        "ospop_diff" => Program::Diff {
            tensor: s("tensor", "layer.weight"),
            scale: args.get("scale").and_then(Value::as_f64).unwrap_or(0.5) as f32,
        },
        "ospop_explain" => Program::Explain {
            topic: s("topic", "ternary"),
        },
        "ospop_learn" => Program::Learn {
            lesson: args
                .get("lesson")
                .and_then(Value::as_str)
                .map(str::to_string),
        },
        other => return Err(format!("unknown tool {other}")),
    };
    rt.execute(&program)
}

fn serve() -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    // Sessions start with the bundled demo snapshot so the first tool call works;
    // model-file IO is a separate adapter (see docs/ARCHITECTURE.md).
    let mut rt = Runtime::new();
    rt.load_demo().map_err(|e| e.to_string())?;
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let _ = writeln!(
                    out,
                    "{}",
                    json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": e.to_string() } })
                );
                let _ = out.flush();
                continue;
            }
        };
        let id = req.id.clone().unwrap_or(Value::Null);
        let response = match req.method.as_str() {
            "initialize" => json!({ "jsonrpc": "2.0", "id": id, "result": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "ospop-mcp", "version": env!("CARGO_PKG_VERSION") },
            } }),
            "notifications/initialized" => continue,
            "tools/list" => {
                json!({ "jsonrpc": "2.0", "id": id, "result": { "tools": tools().iter().map(|t| json!({
                "name": t.name, "description": t.description, "inputSchema": t.input,
            })).collect::<Vec<_>>() } })
            }
            "tools/call" => {
                let name = req.params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = req
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                match call_tool(&mut rt, name, &args) {
                    Ok(result) => {
                        json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result).unwrap_or_default() }] } })
                    }
                    Err(e) => {
                        json!({ "jsonrpc": "2.0", "id": id, "result": { "isError": true, "content": [{ "type": "text", "text": e }] } })
                    }
                }
            }
            "ping" => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
            other => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("unknown method {other}") } })
            }
        };
        let _ = writeln!(out, "{}", response);
        let _ = out.flush();
    }
    Ok(())
}

fn main() {
    if let Err(e) = serve() {
        eprintln!("ospop-mcp: {e}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Repo-relative path to the committed real safetensors checkpoint.
    fn model_path() -> String {
        let root = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/tiny-gpt2.safetensors"
        );
        std::fs::canonicalize(root)
            .unwrap_or_else(|_| root.into())
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn safetensors_tool_loads_real_tensors() {
        let mut rt = Runtime::new();
        let v = call_tool(
            &mut rt,
            "ospop_load_safetensors",
            &json!({ "path": model_path() }),
        )
        .unwrap();
        assert_eq!(v["loaded_tensors"], 64, "tiny-random-gpt2 has 64 tensors");
        assert_eq!(v["tensor_count"], 64);
    }

    #[test]
    fn catalog_matches_the_runtime() {
        let names: Vec<&str> = tools().iter().map(|t| t.name).collect();
        assert_eq!(names.len(), 9);
        for t in &tools() {
            let mut args = json!({ "tensor": "layer.weight", "view": "ternary", "topic": "ternary", "root": "layer.weight", "lesson": "wp.ternary" });
            // A fresh session per tool: loading a file replaces the snapshot.
            let mut rt = Runtime::new();
            rt.load_demo().unwrap();
            if t.name == "ospop_load_safetensors" {
                // Exercise the reader against a real file rather than a stub path.
                args["path"] = json!(model_path());
            }
            assert!(call_tool(&mut rt, t.name, &args).is_ok(), "{}", t.name);
        }
    }

    #[test]
    fn quantize_tool_reports_compression() {
        let mut rt = Runtime::new();
        rt.load_demo().unwrap();
        let v = call_tool(
            &mut rt,
            "ospop_quantize",
            &json!({ "tensor": "layer.weight" }),
        )
        .unwrap();
        assert!(v["packed_bytes"].as_u64().unwrap() < v["float_bytes"].as_u64().unwrap());
        assert_eq!(v["scheme"], "ternary");
    }

    #[test]
    fn unknown_tool_and_bad_view_error() {
        let mut rt = Runtime::new();
        rt.load_demo().unwrap();
        assert!(call_tool(&mut rt, "ospop_nope", &json!({})).is_err());
        assert!(call_tool(
            &mut rt,
            "ospop_lens",
            &json!({ "tensor": "layer.weight", "view": "bogus" })
        )
        .is_err());
    }
}
