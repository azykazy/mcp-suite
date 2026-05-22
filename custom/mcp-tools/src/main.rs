use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

mod diff;
mod file_outline;
mod find;
mod git_diff;
mod git_log;
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
                    },
                    "no_ignore": {
                        "type": "boolean",
                        "description": "If true, ignore .gitignore rules and search all files (default: false)"
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
            "name": "git_diff",
            "description": "Run git diff in a repository. Supports working tree, staged, and commit-range diffs. Token-efficient.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo": {
                        "type": "string",
                        "description": "Path to the git repository (default: \".\")"
                    },
                    "staged": {
                        "type": "boolean",
                        "description": "Show staged changes (--staged). Default: false"
                    },
                    "from": {
                        "type": "string",
                        "description": "Start ref (commit, branch, tag). E.g. \"HEAD~1\" or \"main\""
                    },
                    "to": {
                        "type": "string",
                        "description": "End ref. Used together with `from`"
                    },
                    "path": {
                        "type": "string",
                        "description": "Limit diff to a specific file or directory"
                    },
                    "context": {
                        "type": "integer",
                        "description": "Context lines around changes (default: 3)"
                    }
                },
                "required": []
            }
        },
        {
            "name": "git_log",
            "description": "Show compact git commit history. Returns hash, date, author, subject per line. Token-efficient.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo": {
                        "type": "string",
                        "description": "Path to the git repository (default: \".\")"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of commits to return (default: 20)"
                    },
                    "from": {
                        "type": "string",
                        "description": "Start ref (exclusive). Shows commits from this ref to `to` (or HEAD). E.g. \"HEAD~5\" or \"main\""
                    },
                    "to": {
                        "type": "string",
                        "description": "End ref (inclusive). Used together with `from` (default: HEAD)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Limit history to commits touching this file or directory"
                    }
                },
                "required": []
            }
        },
        {
            "name": "file_outline",
            "description": "Extract function/class/struct/type signatures from a source file. Returns line numbers and signatures only — no function bodies. Supported: .rs .py .ts .tsx .js .jsx .go. Token-efficient alternative to reading whole files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the source file"
                    }
                },
                "required": ["path"]
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
        "file_outline" => file_outline::run(args),
        "git_diff" => git_diff::run(args),
        "git_log" => git_log::run(args),
        "find" => find::run(args),
        other => anyhow::bail!("Unknown tool: {other}"),
    }
}
