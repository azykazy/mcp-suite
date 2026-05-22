use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use serde_json::Value;
use std::fmt::Write as FmtWrite;
use std::fs;

pub fn run(args: &Value) -> Result<String> {
    let a = args["a"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'a' (file path) is required"))?;
    let b = args["b"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'b' (file path) is required"))?;
    let context = args["context"].as_u64().unwrap_or(3) as usize;

    let a_text = fs::read_to_string(a).with_context(|| format!("Cannot read '{a}'"))?;
    let b_text = fs::read_to_string(b).with_context(|| format!("Cannot read '{b}'"))?;

    let diff = TextDiff::from_lines(&a_text, &b_text);
    let groups: Vec<_> = diff.grouped_ops(context);

    if groups.is_empty() {
        return Ok(format!("Files are identical: {a} == {b}\n"));
    }

    let mut output = String::new();
    writeln!(output, "--- {a}").unwrap();
    writeln!(output, "+++ {b}").unwrap();

    for group in &groups {
        let old_start = group.first().map_or(0, |op| op.old_range().start) + 1;
        let old_len: usize = group.iter().map(|op| op.old_range().len()).sum();
        let new_start = group.first().map_or(0, |op| op.new_range().start) + 1;
        let new_len: usize = group.iter().map(|op| op.new_range().len()).sum();
        writeln!(output, "@@ -{old_start},{old_len} +{new_start},{new_len} @@").unwrap();

        for op in group {
            for change in diff.iter_changes(op) {
                let prefix = match change.tag() {
                    ChangeTag::Delete => '-',
                    ChangeTag::Insert => '+',
                    ChangeTag::Equal => ' ',
                };
                let value = change.value();
                write!(output, "{prefix}{value}").unwrap();
                if !value.ends_with('\n') {
                    writeln!(output, "\n\\ No newline at end of file").unwrap();
                }
            }
        }
    }

    Ok(output)
}
