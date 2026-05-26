use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde_json::Value;

pub fn init(args: &Value) -> Result<String> {
    let dir = args["dir"].as_str().unwrap_or(".");
    let mut tf_args: Vec<String> = vec!["init".into(), "-no-color".into()];

    if args["upgrade"].as_bool().unwrap_or(false) {
        tf_args.push("-upgrade".into());
    }
    if args["reconfigure"].as_bool().unwrap_or(false) {
        tf_args.push("-reconfigure".into());
    }
    if args["backend"].as_bool() == Some(false) {
        tf_args.push("-backend=false".into());
    }

    exec_and_clean(dir, &tf_args)
}

pub fn validate(args: &Value) -> Result<String> {
    let dir = args["dir"].as_str().unwrap_or(".");
    let tf_args: Vec<String> = vec!["validate".into(), "-json".into()];

    let (out, success) = run_terraform(dir, &tf_args)?;

    match serde_json::from_str::<Value>(&out) {
        Ok(v) => {
            let pretty = serde_json::to_string_pretty(&v)?;
            if !success {
                anyhow::bail!("{pretty}");
            }
            Ok(pretty)
        }
        Err(_) => {
            if !success {
                anyhow::bail!("{}", clean_output(&out));
            }
            Ok(clean_output(&out))
        }
    }
}

pub fn fmt(args: &Value) -> Result<String> {
    let dir = args["dir"].as_str().unwrap_or(".");
    let check = args["check"].as_bool().unwrap_or(false);
    let diff = args["diff"].as_bool().unwrap_or(false);
    let recursive = args["recursive"].as_bool().unwrap_or(false);

    let mut tf_args: Vec<String> = vec!["fmt".into(), "-no-color".into()];
    if check {
        tf_args.push("-check".into());
    }
    if diff {
        tf_args.push("-diff".into());
    }
    if recursive {
        tf_args.push("-recursive".into());
    }

    let (out, success) = run_terraform(dir, &tf_args)?;
    let cleaned = clean_output(&out);

    if !success && check {
        return Ok(format!(
            "Files need formatting:\n{}",
            if cleaned.is_empty() {
                "(run with diff: true to see changes)"
            } else {
                &cleaned
            }
        ));
    }
    if !success {
        anyhow::bail!("{cleaned}");
    }
    if cleaned.is_empty() {
        Ok("All files are properly formatted.".to_string())
    } else {
        Ok(cleaned)
    }
}

pub fn plan(args: &Value) -> Result<String> {
    let dir = args["dir"].as_str().unwrap_or(".");
    let mut tf_args: Vec<String> = vec!["plan".into(), "-no-color".into()];

    if let Some(out) = args["out"].as_str() {
        tf_args.push(format!("-out={out}"));
    }
    if let Some(vf) = args["var_file"].as_str() {
        tf_args.push(format!("-var-file={vf}"));
    }
    if args["destroy"].as_bool().unwrap_or(false) {
        tf_args.push("-destroy".into());
    }
    if args["refresh_only"].as_bool().unwrap_or(false) {
        tf_args.push("-refresh-only".into());
    }
    if let Some(targets) = args["target"].as_array() {
        for t in targets {
            if let Some(s) = t.as_str() {
                tf_args.push(format!("-target={s}"));
            }
        }
    }
    if let Some(vars) = args["var"].as_object() {
        for (k, v) in vars {
            if let Some(s) = v.as_str() {
                tf_args.push(format!("-var={k}={s}"));
            }
        }
    }

    exec_and_clean(dir, &tf_args)
}

pub fn apply(args: &Value) -> Result<String> {
    if args["auto_approve"].as_bool() != Some(true) {
        anyhow::bail!(
            "apply requires auto_approve: true. \
             Review the plan with tf_plan first, then set auto_approve: true to proceed."
        );
    }

    let dir = args["dir"].as_str().unwrap_or(".");
    let mut tf_args: Vec<String> =
        vec!["apply".into(), "-no-color".into(), "-auto-approve".into()];

    if let Some(vf) = args["var_file"].as_str() {
        tf_args.push(format!("-var-file={vf}"));
    }
    if let Some(targets) = args["target"].as_array() {
        for t in targets {
            if let Some(s) = t.as_str() {
                tf_args.push(format!("-target={s}"));
            }
        }
    }
    if let Some(vars) = args["var"].as_object() {
        for (k, v) in vars {
            if let Some(s) = v.as_str() {
                tf_args.push(format!("-var={k}={s}"));
            }
        }
    }
    if let Some(pf) = args["plan_file"].as_str() {
        tf_args.push(pf.to_string());
    }

    exec_and_clean(dir, &tf_args)
}

pub fn destroy(args: &Value) -> Result<String> {
    if args["auto_approve"].as_bool() != Some(true) {
        anyhow::bail!(
            "destroy requires auto_approve: true. \
             Review what will be destroyed with tf_plan first, then set auto_approve: true to proceed."
        );
    }

    let dir = args["dir"].as_str().unwrap_or(".");
    let mut tf_args: Vec<String> =
        vec!["destroy".into(), "-no-color".into(), "-auto-approve".into()];

    if let Some(vf) = args["var_file"].as_str() {
        tf_args.push(format!("-var-file={vf}"));
    }
    if let Some(targets) = args["target"].as_array() {
        for t in targets {
            if let Some(s) = t.as_str() {
                tf_args.push(format!("-target={s}"));
            }
        }
    }
    if let Some(vars) = args["var"].as_object() {
        for (k, v) in vars {
            if let Some(s) = v.as_str() {
                tf_args.push(format!("-var={k}={s}"));
            }
        }
    }

    exec_and_clean(dir, &tf_args)
}

pub fn output(args: &Value) -> Result<String> {
    let dir = args["dir"].as_str().unwrap_or(".");
    let mut tf_args: Vec<String> = vec!["output".into(), "-json".into()];

    if let Some(name) = args["name"].as_str() {
        tf_args.push(name.to_string());
    }

    let (out, success) = run_terraform(dir, &tf_args)?;

    if !success {
        anyhow::bail!("{}", clean_output(&out));
    }

    match serde_json::from_str::<Value>(&out) {
        Ok(v) => Ok(serde_json::to_string_pretty(&v)?),
        Err(_) => Ok(clean_output(&out)),
    }
}

pub fn state_list(args: &Value) -> Result<String> {
    let dir = args["dir"].as_str().unwrap_or(".");
    let mut tf_args: Vec<String> = vec!["state".into(), "list".into()];

    if let Some(id) = args["id"].as_str() {
        tf_args.push(format!("-id={id}"));
    }
    if let Some(addrs) = args["address"].as_array() {
        for a in addrs {
            if let Some(s) = a.as_str() {
                tf_args.push(s.to_string());
            }
        }
    }

    let (out, success) = run_terraform(dir, &tf_args)?;

    if !success {
        anyhow::bail!("{}", clean_output(&out));
    }

    let trimmed = out.trim().to_string();
    if trimmed.is_empty() {
        Ok("No resources in state.".to_string())
    } else {
        Ok(trimmed)
    }
}

pub fn workspace(args: &Value) -> Result<String> {
    let dir = args["dir"].as_str().unwrap_or(".");
    let cmd = args["command"].as_str().unwrap_or("list");

    let valid = ["list", "show", "select", "new", "delete"];
    if !valid.contains(&cmd) {
        anyhow::bail!(
            "Invalid workspace command: {cmd}. Must be one of: list, show, select, new, delete"
        );
    }

    let mut tf_args: Vec<String> = vec!["workspace".into(), cmd.to_string()];

    if let Some(name) = args["name"].as_str() {
        tf_args.push(name.to_string());
    } else if matches!(cmd, "select" | "new" | "delete") {
        anyhow::bail!("workspace {cmd} requires a name");
    }

    exec_and_clean(dir, &tf_args)
}

// ── internals ────────────────────────────────────────────────────────────────

fn exec_and_clean(dir: &str, tf_args: &[String]) -> Result<String> {
    let (out, success) = run_terraform(dir, tf_args)?;
    let cleaned = clean_output(&out);
    if !success {
        anyhow::bail!("{cleaned}");
    }
    Ok(cleaned)
}

fn run_terraform(dir: &str, args: &[String]) -> Result<(String, bool)> {
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = Command::new("terraform")
        .args(&args_ref)
        .current_dir(dir)
        // Suppress interactive guidance messages intended for terminal users
        .env("TF_IN_AUTOMATION", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute terraform — is it installed and in PATH?")?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let combined = match (stdout.trim().is_empty(), stderr.trim().is_empty()) {
        (true, true) => String::new(),
        (true, false) => stderr,
        (false, true) => stdout,
        (false, false) => format!("{}\n{}", stdout.trim_end(), stderr.trim_start()),
    };

    Ok((combined, output.status.success()))
}

/// Remove ANSI codes, filter noisy Terraform progress lines, compress blank lines.
fn clean_output(text: &str) -> String {
    let stripped = strip_ansi(text);
    let stripped = stripped.replace('\r', "");

    let mut result: Vec<&str> = Vec::new();
    let mut consecutive_blanks: usize = 0;

    for line in stripped.lines() {
        if is_noise(line) {
            continue;
        }
        if line.trim().is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks == 1 {
                result.push("");
            }
        } else {
            consecutive_blanks = 0;
            result.push(line);
        }
    }

    while result.first() == Some(&"") {
        result.remove(0);
    }
    while result.last() == Some(&"") {
        result.pop();
    }

    result.join("\n")
}

/// Returns true for lines that are purely Terraform progress noise.
fn is_noise(line: &str) -> bool {
    let t = line.trim_end();
    // "data.X: Reading..."
    if t.ends_with(": Reading...") {
        return true;
    }
    // "data.X: Read complete after 0s [id=...]"
    if t.contains(": Read complete after ") && t.contains(" [id=") {
        return true;
    }
    // "aws_vpc.main: Refreshing state... [id=...]"
    if t.contains(": Refreshing state... [id=") {
        return true;
    }
    // "X: Still creating/modifying/destroying/reading... [10s elapsed]"
    if t.ends_with(" elapsed]") {
        for verb in &["Still creating", "Still modifying", "Still destroying", "Still reading"] {
            if t.contains(&format!(": {verb}... [")) {
                return true;
            }
        }
    }
    // State lock noise
    matches!(
        t,
        "Acquiring state lock. This may take a moment..."
            | "Releasing state lock. This may take a moment..."
    )
}

/// Strip ANSI CSI escape sequences (e.g. color codes).
fn strip_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            for ch in chars.by_ref() {
                if ch.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}
