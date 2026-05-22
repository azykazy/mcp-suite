use anyhow::{Context, Result};
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

pub fn run(args: &Value) -> Result<String> {
    let pattern = args["pattern"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'pattern' is required"))?;
    let paths: Vec<String> = args["paths"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("'paths' is required (array of strings)"))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let context = args["context"].as_u64().unwrap_or(0) as usize;
    let ignore_case = args["ignore_case"].as_bool().unwrap_or(false);
    let max_matches = args["max_matches"].as_u64().unwrap_or(100) as usize;
    let no_ignore = args["no_ignore"].as_bool().unwrap_or(false);

    let re = RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
        .context("Invalid regex pattern")?;

    let mut output = String::new();
    let mut total_matches = 0usize;
    let mut truncated = false;

    'outer: for path_str in &paths {
        let path = Path::new(path_str);
        if !path.exists() {
            output.push_str(&format!("# Warning: '{path_str}' not found\n"));
            continue;
        }

        let files: Vec<PathBuf> = if path.is_dir() {
            let mut entries: Vec<PathBuf> = WalkBuilder::new(path)
                .hidden(false)
                .git_ignore(!no_ignore)
                .git_global(!no_ignore)
                .git_exclude(!no_ignore)
                .ignore(!no_ignore)
                .build()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .map(|e| e.path().to_path_buf())
                .collect();
            entries.sort();
            entries
        } else {
            vec![path.to_path_buf()]
        };

        for file_path in files {
            if total_matches >= max_matches {
                truncated = true;
                break 'outer;
            }

            let added = grep_file(
                &file_path,
                &re,
                context,
                max_matches - total_matches,
                &mut output,
            )?;
            total_matches += added;
        }
    }

    if truncated {
        output.push_str(&format!(
            "\n# Truncated: {total_matches} matches shown. Increase max_matches to see more.\n"
        ));
    } else if total_matches == 0 {
        output.push_str("No matches found\n");
    }

    Ok(output)
}

fn is_binary(path: &Path) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 8192];
    let n = file.read(&mut buf).unwrap_or(0);
    buf[..n].contains(&0)
}

fn grep_file(
    path: &Path,
    re: &regex::Regex,
    context: usize,
    limit: usize,
    output: &mut String,
) -> Result<usize> {
    if is_binary(path) {
        return Ok(0);
    }

    let file = fs::File::open(path)?;
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .map(|l| l.unwrap_or_default())
        .collect();

    let path_str = path.to_string_lossy();

    if context == 0 {
        let mut count = 0;
        for (i, line) in lines.iter().enumerate() {
            if count >= limit {
                break;
            }
            if re.is_match(line) {
                output.push_str(&format!("{}:{}: {}\n", path_str, i + 1, line));
                count += 1;
            }
        }
        return Ok(count);
    }

    // With context: collect match indices, build merged blocks, output
    let match_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| re.is_match(line))
        .map(|(i, _)| i)
        .collect();

    if match_indices.is_empty() {
        return Ok(0);
    }

    // Merge overlapping context windows into blocks
    let mut blocks: Vec<(usize, usize)> = vec![];
    let mut blk_start = match_indices[0].saturating_sub(context);
    let mut blk_end = (match_indices[0] + context + 1).min(lines.len());

    for &m in &match_indices[1..] {
        let m_start = m.saturating_sub(context);
        let m_end = (m + context + 1).min(lines.len());
        if m_start <= blk_end {
            blk_end = blk_end.max(m_end);
        } else {
            blocks.push((blk_start, blk_end));
            blk_start = m_start;
            blk_end = m_end;
        }
    }
    blocks.push((blk_start, blk_end));

    let match_set: HashSet<usize> = match_indices.iter().cloned().collect();
    let mut count = 0;
    let mut first = true;

    for (start, end) in blocks {
        if count >= limit {
            break;
        }
        if !first {
            output.push_str("--\n");
        }
        first = false;

        for i in start..end {
            let marker = if match_set.contains(&i) { ">" } else { " " };
            output.push_str(&format!("{}{}:{}: {}\n", marker, path_str, i + 1, lines[i]));
            if match_set.contains(&i) {
                count += 1;
                if count >= limit {
                    break;
                }
            }
        }
    }

    Ok(count)
}
