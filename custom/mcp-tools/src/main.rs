use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

mod diff;
mod find;
mod grep;

#[derive(Deserialize)]
struct RpcRequest {
    #[serde(default)]
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

fn main() {
    let stdin = io::stdin();
    let mut out = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => handle(req),
            Err(e) => Some(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": {"code": -32700, "message": format!("Parse error: {e}")}
            })),
        };

        if let Some(resp) = response {
            writeln!(out, "{resp}").ok();
            out.flush().ok();
        }
    }
}

fn handle(req: RpcRequest) -> Option<Value> {
    if req.method.starts_with("notifications/") {
        return None;
    }

    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "mcp-tools", "version": "0.1.0"}
            }
        })),
        "ping" => Some(json!({"jsonrpc": "2.0", "id": id, "result": {}})),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {"tools": tools_list()}
        })),
        "tools/call" => {
            let name = req.params["name"].as_str().unwrap_or("");
            let args = &req.params["arguments"];
            let (text, is_error) = match call_tool(name, args) {
                Ok(t) => (t, false),
                Err(e) => (format!("Error: {e}"), true),
            };
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{"type": "text", "text": text}],
                    "isError": is_error
                }
            }))
        }
        other => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {other}")
            }
        })),
    }
}

fn tools_list() -> Value {
    json!([
        {
            "name": "grep",
            "description": "Search for regex pattern in files or directories. Returns compact `path:line: content` output. Token-efficient.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "paths": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of file or directory paths to search"
                    },
                    "context": {
                        "type": "integer",
                        "description": "Lines of context around each match (default: 0)"
                    },
                    "ignore_case": {
                        "type": "boolean",
                        "description": "Case-insensitive matching (default: false)"
                    },
                    "max_matches": {
                        "type": "integer",
                        "description": "Maximum matches to return (default: 100)"
                    }
                },
                "required": ["pattern", "paths"]
            }
        },
        {
            "name": "diff",
            "description": "Compare two files and show differences in unified diff format. Token-efficient.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "a": {
                        "type": "string",
                        "description": "Path to the first file"
                    },
                    "b": {
                        "type": "string",
                        "description": "Path to the second file"
                    },
                    "context": {
                        "type": "integer",
                        "description": "Context lines around changes (default: 3)"
                    }
                },
                "required": ["a", "b"]
            }
        },
        {
            "name": "find",
            "description": "Find files or directories matching a pattern. Returns newline-separated paths. Token-efficient.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Root directory to search from"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern or substring to match against filename or path (optional)"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["f", "d", "any"],
                        "description": "Filter: f=files only, d=dirs only, any=all (default: any)"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum directory depth to recurse (optional)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum results to return (default: 200)"
                    }
                },
                "required": ["path"]
            }
        }
    ])
}

fn call_tool(name: &str, args: &Value) -> Result<String> {
    match name {
        "grep" => grep::run(args),
        "diff" => diff::run(args),
        "find" => find::run(args),
        other => anyhow::bail!("Unknown tool: {other}"),
    }
}
