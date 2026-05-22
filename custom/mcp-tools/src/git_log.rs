use anyhow::{bail, Result};
use serde_json::Value;
use std::process::Command;

pub fn run(args: &Value) -> Result<String> {
    let repo = args["repo"].as_str().unwrap_or(".");
    let limit = args["limit"].as_u64().unwrap_or(20);
    let from = args["from"].as_str();
    let to = args["to"].as_str();
    let path = args["path"].as_str();

    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo);
    cmd.arg("log");
    cmd.arg(format!("-{limit}"));
    cmd.arg("--pretty=format:%h  %ad  %an  %s");
    cmd.arg("--date=short");

    match (from, to) {
        (Some(f), Some(t)) => {
            cmd.arg(format!("{f}..{t}"));
        }
        (Some(f), None) => {
            cmd.arg(format!("{f}..HEAD"));
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
        bail!("git log failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok("No commits found.\n".to_string());
    }

    Ok(stdout.into_owned())
}
