use anyhow::Result;
use ignore::WalkBuilder;
use serde_json::Value;
use std::path::Path;

pub fn run(args: &Value) -> Result<String> {
    let path = args["path"].as_str().unwrap_or(".");
    let max_depth = args["max_depth"].as_u64().map(|d| d as usize);
    let no_ignore = args["no_ignore"].as_bool().unwrap_or(false);

    if !Path::new(path).exists() {
        anyhow::bail!("Path not found: {path}");
    }

    let mut builder = WalkBuilder::new(path);
    builder.hidden(false);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);
    if let Some(d) = max_depth {
        builder.max_depth(Some(d));
    }

    let mut entries: Vec<(usize, bool, String)> = Vec::new();
    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let depth = entry.depth();
        if depth == 0 {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".git" {
            continue;
        }
        // skip entries inside .git
        if entry
            .path()
            .components()
            .any(|c| c.as_os_str() == ".git")
        {
            continue;
        }
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        entries.push((depth, is_dir, name));
    }

    if entries.is_empty() {
        return Ok(format!("{path}/\n(empty)\n"));
    }

    let mut out = format!("{path}/\n");
    for (depth, is_dir, name) in &entries {
        let indent = "  ".repeat(depth - 1);
        if *is_dir {
            out.push_str(&format!("{indent}{name}/\n"));
        } else {
            out.push_str(&format!("{indent}{name}\n"));
        }
    }

    Ok(out)
}
