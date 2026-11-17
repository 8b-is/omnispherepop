#![forbid(unsafe_code)]
//! `ospop` — weight programming for humans and agents.
//!
//! A thin adapter: every subcommand builds a [`Program`] and hands it to the
//! runtime, so CLI, MCP, and WASM cannot drift apart.

use clap::{Parser, Subcommand};
use ospop_core::snapshot::Snapshot;
use ospop_runtime::{LensView, Program, Runtime, Threshold};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const VERSION: &str = concat!(
    "ospop ",
    env!("CARGO_PKG_VERSION"),
    "\nOmnispherepop: weight programming for humans and agents",
    "\n(c) 2026 Peter Lodri Sec, 88B (8b-is), Standard Galactic",
    "\nSPDX-License-Identifier: MIT OR Apache-2.0",
);

#[derive(Parser)]
#[command(name = "ospop", about = "Weight programming for humans and agents", long_version = VERSION)]
struct Cli {
    /// Interleave plain-language explanation with the command's output.
    #[arg(long, global = true)]
    eli5: bool,
    /// Emit machine-readable JSON instead of human text.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show snapshot identity and tensors.
    Inspect,
    /// Quantize a tensor to ternary with per-group scales.
    Quantize {
        #[arg(long, default_value = "layer.weight")]
        tensor: String,
        #[arg(long, default_value_t = 128)]
        group_size: usize,
        #[arg(long)]
        threshold: Option<f32>,
    },
    /// Render one tensor through a view.
    Lens {
        #[arg(long, default_value = "layer.weight")]
        tensor: String,
        #[arg(long, default_value = "ternary")]
        view: String,
    },
    /// Diff a tensor against a scaled copy and prove the patch reverses.
    Diff {
        #[arg(long, default_value = "layer.weight")]
        tensor: String,
        #[arg(long, default_value_t = 0.5)]
        scale: f32,
    },
    /// Check graph reachability from a root tensor.
    Prune {
        #[arg(long, default_value = "layer.weight")]
        root: String,
    },
    /// List the curriculum, or run one lesson's lab.
    Learn {
        lesson: Option<String>,
        #[arg(long)]
        list: bool,
    },
    /// Explain a topic in plain language.
    Explain { topic: String },
    /// Read a snapshot artifact from stdin or a file.
    Load { path: Option<PathBuf> },
}

fn print_human(value: &serde_json::Value, eli5: bool) -> Result<(), String> {
    let out = io::stdout();
    let mut out = out.lock();
    let op = value.get("op").and_then(|v| v.as_str()).unwrap_or("result");
    match op {
        "quantize" => {
            let sparsity = value["sparsity"].as_f64().unwrap_or(0.0);
            let packed = value["packed_bytes"].as_u64().unwrap_or(0);
            let floats = value["float_bytes"].as_u64().unwrap_or(0);
            let _ = writeln!(
                out,
                "tensor {}: {} values in {} groups",
                value["tensor"].as_str().unwrap_or("?"),
                value["values"].as_u64().unwrap_or(0),
                value["groups"].as_u64().unwrap_or(0)
            );
            let _ = writeln!(
                out,
                "trits: {} bytes vs {} float bytes ({:.1}% smaller)",
                packed,
                floats,
                if floats > 0 {
                    100.0 * (1.0 - packed as f64 / floats as f64)
                } else {
                    0.0
                }
            );
            let _ = writeln!(
                out,
                "sparsity {:.1}%  mse {:.4}  scale range [{:.4}, {:.4}]",
                sparsity * 100.0,
                value["mse"].as_f64().unwrap_or(0.0),
                value["min_scale"].as_f64().unwrap_or(0.0),
                value["max_scale"].as_f64().unwrap_or(0.0)
            );
            if eli5 {
                let _ = writeln!(
                    out,
                    "\nELI5: dimmer switches became light switches (dim/off/bright), so each weight costs 2 bits instead of 32."
                );
            }
        }
        "lens" => {
            let view = value["view"].as_str().unwrap_or("?");
            let _ = writeln!(
                out,
                "lens {} on {}",
                view,
                value["tensor"].as_str().unwrap_or("?")
            );
            if let Some(trits) = value["trits"].as_array() {
                let line: String = trits
                    .iter()
                    .map(|t| match t.as_i64() {
                        Some(-1) => '-',
                        Some(0) => '.',
                        Some(1) => '+',
                        _ => '?',
                    })
                    .collect();
                for chunk in line.as_bytes().chunks(64) {
                    let _ = writeln!(out, "{}", String::from_utf8_lossy(chunk));
                }
            } else if let Some(buckets) = value["buckets"].as_array() {
                let max = buckets
                    .iter()
                    .filter_map(|b| b.as_u64())
                    .max()
                    .unwrap_or(1)
                    .max(1);
                for (i, b) in buckets.iter().enumerate() {
                    let n = b.as_u64().unwrap_or(0);
                    let bar = "#".repeat((n as f64 / max as f64 * 40.0) as usize);
                    let _ = writeln!(out, "{:2}0-{:2}% | {:<40} {}", i * 10, i * 10 + 10, bar, n);
                }
            } else if let Some(rows) = value["rows"].as_array() {
                for row in rows.iter().take(8) {
                    let cells: Vec<String> = row
                        .as_array()
                        .map(|r| {
                            r.iter()
                                .map(|v| {
                                    let x = v.as_f64().unwrap_or(0.0);
                                    let level = ((x.abs().min(1.0)) * 8.0) as usize;
                                    " .:-=+*#%@".chars().nth(level).unwrap_or(' ').to_string()
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let _ = writeln!(out, "{}", cells.join(""));
                }
            } else if let Some(edges) = value.get("edges") {
                let _ = writeln!(
                    out,
                    "vertices {}  edges {}",
                    value["vertices"].as_array().map(|v| v.len()).unwrap_or(0),
                    edges
                );
            } else {
                let _ = writeln!(out, "{}", value["round_trip_ok"]);
            }
            if eli5 {
                let _ = writeln!(out, "\nELI5: this is one slice of the model seen through one lens; same weights, different question.");
            }
        }
        "diff" => {
            let _ = writeln!(
                out,
                "{} -> {} across {} tensor(s), round trip {}",
                value["base_id"].as_str().unwrap_or("?"),
                value["target_id"].as_str().unwrap_or("?"),
                value["edited_tensors"].as_u64().unwrap_or(0),
                value["round_trip_ok"]
            );
            if eli5 {
                let _ = writeln!(out, "\nELI5: the diff is a receipt bound to the base snapshot, so the edit set can be reversed exactly.");
            }
        }
        "prune" => {
            let _ = writeln!(
                out,
                "root {} keeps {} tensor(s), drops {}, {} ultra-edges, restore {}",
                value["root"].as_str().unwrap_or("?"),
                value["kept"].as_u64().unwrap_or(0),
                value["dropped"].as_u64().unwrap_or(0),
                value["edges"].as_u64().unwrap_or(0),
                value["restore_ok"]
            );
        }
        "learn" => match value["lesson"].as_str() {
            None => {
                let _ = writeln!(
                    out,
                    "{} lessons",
                    value["lessons"].as_array().map(|l| l.len()).unwrap_or(0)
                );
                for l in value["lessons"].as_array().into_iter().flatten() {
                    let _ = writeln!(
                        out,
                        "  {:<22} [{:<18}] {}",
                        l["id"].as_str().unwrap_or("?"),
                        l["track"].as_str().unwrap_or("?"),
                        l["title"].as_str().unwrap_or("?")
                    );
                }
            }
            Some(id) => {
                let _ = writeln!(out, "{} — {}", id, value["title"].as_str().unwrap_or("?"));
                let _ = writeln!(
                    out,
                    "\nconcept: {}",
                    value["concept"].as_str().unwrap_or("?")
                );
                let _ = writeln!(out, "\nELI5: {}", value["eli5"].as_str().unwrap_or("?"));
                let obs = &value["observation"];
                let _ = writeln!(
                    out,
                    "\nlab ({}): sparsity {:.1}%  reconstruction {:.4}  round trip {}  curriculum reachable {}",
                    value["lab_target"].as_str().unwrap_or("?"),
                    obs["sparsity"].as_f64().unwrap_or(0.0) * 100.0,
                    obs["reconstruction_error"].as_f64().unwrap_or(0.0),
                    obs["patch_round_trip"],
                    obs["graph_fully_reachable"]
                );
                let _ = writeln!(
                    out,
                    "checkpoint: {}",
                    if value["checkpoint_passed"] == true {
                        "PASS"
                    } else {
                        "FAIL"
                    }
                );
            }
        },
        "explain" => {
            let _ = writeln!(out, "{}", value["eli5"].as_str().unwrap_or("?"));
            let _ = writeln!(
                out,
                "\n{}: {}",
                value["concept"].as_str().unwrap_or(""),
                value["lab"]["target"].as_str().unwrap_or("")
            );
        }
        "inspect" => {
            let _ = writeln!(
                out,
                "snapshot {}",
                value["snapshot_id"].as_str().unwrap_or("?")
            );
            for (name, len) in value["tensors"].as_object().into_iter().flatten() {
                let _ = writeln!(out, "  {:<16} {} values", name, len);
            }
        }
        other => {
            let _ = writeln!(out, "{other}: {}", value);
        }
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let mut rt = Runtime::new();
    rt.load_demo()?;

    let (program, passthrough) = match cli.command {
        Command::Inspect => (Program::Inspect, None),
        Command::Quantize {
            tensor,
            group_size,
            threshold,
        } => (
            Program::Quantize {
                tensor,
                group_size,
                threshold: threshold.map_or(Threshold::Auto, Threshold::Fixed),
            },
            None,
        ),
        Command::Lens { tensor, view } => {
            let view = LensView::parse(&view).ok_or_else(|| format!("unknown view {view}"))?;
            (Program::Lens { tensor, view }, None)
        }
        Command::Diff { tensor, scale } => (Program::Diff { tensor, scale }, None),
        Command::Prune { root } => (Program::Prune { root }, None),
        Command::Learn { lesson, list } => (
            Program::Learn {
                lesson: if list { None } else { lesson },
            },
            None,
        ),
        Command::Explain { topic } => (Program::Explain { topic }, None),
        Command::Load { path } => {
            let bytes = match path {
                Some(p) => std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?,
                None => {
                    let mut buf = Vec::new();
                    io::stdin()
                        .read_to_end(&mut buf)
                        .map_err(|e| e.to_string())?;
                    buf
                }
            };
            let snap = Snapshot::from_bytes(&bytes)?;
            let id = snap.id().to_string();
            let count = snap.tensors().len();
            rt.load_snapshot(snap);
            (Program::Inspect, Some((id, count)))
        }
    };

    let value = rt.execute(&program)?;
    if let Some((id, count)) = passthrough {
        println!("loaded {id} with {count} tensor(s)");
        return Ok(());
    }
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        );
    } else {
        print_human(&value, cli.eli5)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ospop: {e}");
            ExitCode::from(2)
        }
    }
}
