use anyhow::{bail, Result};
use serde_json::Value;
use std::process::Command;

pub fn run(args: &Value) -> Result<String> {
    let repo = args["repo"].as_str().unwrap_or(".");
    let staged = args["staged"].as_bool().unwrap_or(false);
    let from = args["from"].as_str();
    let to = args["to"].as_str();
    let path = args["path"].as_str();
    let context = args["context"].as_u64().unwrap_or(3);

    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo);
    cmd.arg("diff");
    cmd.arg(format!("-U{context}"));

    if staged {
        cmd.arg("--staged");
    }

    match (from, to) {
        (Some(f), Some(t)) => {
            cmd.arg(f);
            cmd.arg(t);
        }
        (Some(f), None) => {
            cmd.arg(f);
        }
        _ => {}
    }

    cmd.arg("--");
    if let Some(p) = path {
        cmd.arg(p);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.is_empty() {
        return Ok("No differences found.\n".to_string());
    }

    Ok(stdout.into_owned())
}
