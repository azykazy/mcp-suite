use anyhow::Result;
use serde_json::Value;
use std::path::Path;
use walkdir::WalkDir;

pub fn run(args: &Value) -> Result<String> {
    let root = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'path' is required"))?;
    let pattern = args["pattern"].as_str();
    let file_type = args["type"].as_str().unwrap_or("any");
    let max_depth = args["max_depth"].as_u64().map(|n| n as usize);
    let max_results = args["max_results"].as_u64().unwrap_or(200) as usize;

    if !Path::new(root).exists() {
        anyhow::bail!("Path not found: {root}");
    }

    let mut walker = WalkDir::new(root).sort_by_file_name();
    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    let mut output = String::new();
    let mut count = 0;

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        // Filter by type
        let matches_type = match file_type {
            "f" => entry.file_type().is_file(),
            "d" => entry.file_type().is_dir(),
            _ => true,
        };
        if !matches_type {
            continue;
        }

        // Skip root entry itself for file-only searches
        if entry.depth() == 0 && file_type != "d" {
            continue;
        }

        let path_str = entry.path().to_string_lossy();
        let filename = entry.file_name().to_string_lossy();

        // Filter by pattern (glob or substring)
        if let Some(pat) = pattern {
            let matched = if pat.contains('*') || pat.contains('?') {
                glob_match(&filename, pat) || glob_match(&path_str, pat)
            } else {
                filename.contains(pat) || path_str.contains(pat)
            };
            if !matched {
                continue;
            }
        }

        if count >= max_results {
            output.push_str(&format!("\n# Truncated at {max_results} results\n"));
            break;
        }

        output.push_str(&format!("{path_str}\n"));
        count += 1;
    }

    if output.is_empty() {
        output.push_str("No results found\n");
    }

    Ok(output)
}

fn glob_match(text: &str, pattern: &str) -> bool {
    match_bytes(text.as_bytes(), pattern.as_bytes())
}

fn match_bytes(text: &[u8], pattern: &[u8]) -> bool {
    match (text.first(), pattern.first()) {
        (_, Some(b'*')) => {
            match_bytes(text, &pattern[1..])
                || (!text.is_empty() && match_bytes(&text[1..], pattern))
        }
        (Some(_), Some(b'?')) => match_bytes(&text[1..], &pattern[1..]),
        (Some(tc), Some(pc)) if tc == pc => match_bytes(&text[1..], &pattern[1..]),
        (None, None) => true,
        _ => false,
    }
}
