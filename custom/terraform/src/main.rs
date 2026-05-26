use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

mod terraform;

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
                "serverInfo": {"name": "terraform-mcp", "version": "0.1.0"}
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
            "name": "tf_init",
            "description": "Run terraform init to initialize a working directory. Downloads providers and modules. Strips noisy output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    },
                    "upgrade": {
                        "type": "boolean",
                        "description": "Upgrade providers/modules to latest allowed versions (default: false)"
                    },
                    "reconfigure": {
                        "type": "boolean",
                        "description": "Reconfigure backend, ignoring any saved configuration (default: false)"
                    },
                    "backend": {
                        "type": "boolean",
                        "description": "Enable backend configuration. Set false to skip (default: true)"
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_validate",
            "description": "Validate Terraform configuration files. Returns structured JSON with error details.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_fmt",
            "description": "Check or fix Terraform file formatting.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory to format (default: \".\")"
                    },
                    "check": {
                        "type": "boolean",
                        "description": "Check mode only — do not write changes (default: false)"
                    },
                    "diff": {
                        "type": "boolean",
                        "description": "Show diff of formatting changes (default: false)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Process subdirectories recursively (default: false)"
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_plan",
            "description": "Generate a Terraform execution plan. Filters out noisy refresh/reading state lines for token-efficient output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    },
                    "out": {
                        "type": "string",
                        "description": "Save plan to file for later use with tf_apply"
                    },
                    "target": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Limit plan to specific resource addresses"
                    },
                    "var": {
                        "type": "object",
                        "description": "Variable overrides as key-value string pairs"
                    },
                    "var_file": {
                        "type": "string",
                        "description": "Path to a .tfvars variable definitions file"
                    },
                    "destroy": {
                        "type": "boolean",
                        "description": "Plan a destroy operation (default: false)"
                    },
                    "refresh_only": {
                        "type": "boolean",
                        "description": "Only refresh state without planning changes (default: false)"
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_apply",
            "description": "Apply Terraform changes. DESTRUCTIVE: modifies real infrastructure. Requires auto_approve: true to proceed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    },
                    "auto_approve": {
                        "type": "boolean",
                        "description": "Must be true to apply. If false, returns an error with a reminder to review the plan first."
                    },
                    "plan_file": {
                        "type": "string",
                        "description": "Apply a saved plan file (from tf_plan with out parameter)"
                    },
                    "target": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Limit apply to specific resource addresses"
                    },
                    "var": {
                        "type": "object",
                        "description": "Variable overrides as key-value string pairs"
                    },
                    "var_file": {
                        "type": "string",
                        "description": "Path to a .tfvars variable definitions file"
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_destroy",
            "description": "Destroy Terraform-managed infrastructure. DESTRUCTIVE: permanently removes resources. Requires auto_approve: true.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    },
                    "auto_approve": {
                        "type": "boolean",
                        "description": "Must be true to destroy. If false, returns an error with a reminder to review first."
                    },
                    "target": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Limit destroy to specific resource addresses"
                    },
                    "var": {
                        "type": "object",
                        "description": "Variable overrides as key-value string pairs"
                    },
                    "var_file": {
                        "type": "string",
                        "description": "Path to a .tfvars variable definitions file"
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_output",
            "description": "Read output values from Terraform state. Returns pretty-printed JSON.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    },
                    "name": {
                        "type": "string",
                        "description": "Specific output name. Omit to return all outputs."
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_state_list",
            "description": "List resources tracked in Terraform state.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    },
                    "id": {
                        "type": "string",
                        "description": "Filter resources by infrastructure ID"
                    },
                    "address": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Limit listing to specific resource addresses"
                    }
                },
                "required": []
            }
        },
        {
            "name": "tf_workspace",
            "description": "Manage Terraform workspaces: list, show current, select, create, or delete.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory containing .tf files (default: \".\")"
                    },
                    "command": {
                        "type": "string",
                        "enum": ["list", "show", "select", "new", "delete"],
                        "description": "Workspace subcommand (default: list)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Workspace name — required for select, new, delete"
                    }
                },
                "required": []
            }
        }
    ])
}

fn call_tool(name: &str, args: &Value) -> Result<String> {
    match name {
        "tf_init" => terraform::init(args),
        "tf_validate" => terraform::validate(args),
        "tf_fmt" => terraform::fmt(args),
        "tf_plan" => terraform::plan(args),
        "tf_apply" => terraform::apply(args),
        "tf_destroy" => terraform::destroy(args),
        "tf_output" => terraform::output(args),
        "tf_state_list" => terraform::state_list(args),
        "tf_workspace" => terraform::workspace(args),
        other => anyhow::bail!("Unknown tool: {other}"),
    }
}
