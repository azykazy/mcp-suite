use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn run(args: &Value) -> Result<String> {
    let path = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("path is required"))?;

    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {path}: {e}"))?;

    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let pattern: &str = match ext {
        "rs" => {
            r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s|^[ \t]*(?:pub(?:\([^)]*\))?\s+)?struct\s|^[ \t]*(?:pub(?:\([^)]*\))?\s+)?enum\s|^[ \t]*(?:pub(?:\([^)]*\))?\s+)?trait\s|^[ \t]*impl(?:<[^>]*>)?\s|^[ \t]*(?:pub(?:\([^)]*\))?\s+)?type\s+\w"
        }
        "py" => r"(?m)^[ \t]*(?:async\s+)?def\s|^[ \t]*class\s",
        "ts" | "tsx" | "js" | "jsx" => {
            r"(?m)^[ \t]*(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s|^[ \t]*(?:export\s+)?(?:default\s+)?class\s|^[ \t]*(?:export\s+)?interface\s|^[ \t]*(?:export\s+)?type\s+\w+\s*="
        }
        "go" => r"(?m)^func\s|^type\s+\w+\s+(?:struct|interface)",
        _ => {
            return Ok(format!(
                "Unsupported file type: .{ext}\nSupported: .rs .py .ts .tsx .js .jsx .go\n"
            ));
        }
    };

    let re = Regex::new(pattern)?;
    let mut out = String::new();

    for (i, line) in content.lines().enumerate() {
        if re.is_match(line) {
            let sig = line.trim_end().trim_end_matches('{').trim_end_matches(':').trim_end();
            let sig = if sig.len() > 120 { &sig[..120] } else { sig };
            out.push_str(&format!("{:4}: {sig}\n", i + 1));
        }
    }

    if out.is_empty() {
        return Ok("No symbols found.\n".to_string());
    }

    Ok(out)
}
