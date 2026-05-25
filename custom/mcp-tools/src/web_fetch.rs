use anyhow::Result;
use ego_tree::NodeRef;
use scraper::{node::Node, Html, Selector};
use serde_json::Value;
use std::sync::Arc;

const TIMEOUT_SECS: u64 = 15;
const SYSTEM_CA: &str = "/etc/ssl/certs/ca-certificates.crt";

// Tags whose entire subtree is discarded
const SKIP: &[&str] = &[
    "script", "style", "noscript", "head", "svg", "template", "iframe",
];

fn build_tls() -> Arc<rustls::ClientConfig> {
    let mut root_store = rustls::RootCertStore::empty();
    if let Ok(file) = std::fs::File::open(SYSTEM_CA) {
        let mut reader = std::io::BufReader::new(file);
        for cert in rustls_pemfile::certs(&mut reader).flatten() {
            root_store.add(cert).ok();
        }
    }
    Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    )
}

pub fn run(args: &Value) -> Result<String> {
    let url = args["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("url is required"))?;
    let max_chars = args["max_chars"].as_u64().map(|n| n as usize);
    let selector_str = args["selector"].as_str();

    let agent = ureq::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .tls_config(build_tls())
        .build();

    let response = agent
        .get(url)
        .set("User-Agent", "mcp-tools/0.1 (AI assistant; text extraction)")
        .set("Accept", "text/html,text/plain,*/*")
        .call()
        .map_err(|e| anyhow::anyhow!("HTTP error: {e}"))?;

    let content_type = response.content_type().to_string();
    let body = response.into_string()?;

    if !content_type.contains("html") {
        return Ok(match max_chars {
            Some(n) => truncate(&body, n).to_string(),
            None => body,
        });
    }

    let document = Html::parse_document(&body);
    let md = html_to_md(&document, selector_str)?;

    let content = match max_chars {
        Some(n) => truncate(&md, n).to_string(),
        None => md,
    };
    Ok(format!("[{url}]\n\n{content}"))
}

// ── HTML → Markdown 変換 ─────────────────────────────────────────────────

fn html_to_md(document: &Html, selector_str: Option<&str>) -> Result<String> {
    let mut out = String::new();

    if let Some(sel_str) = selector_str {
        let sel = Selector::parse(sel_str)
            .map_err(|_| anyhow::anyhow!("Invalid CSS selector: {sel_str}"))?;
        for el in document.select(&sel) {
            walk(*el, &mut out);
        }
    } else {
        let body_sel = Selector::parse("body").unwrap();
        let node = document
            .select(&body_sel)
            .next()
            .map(|el| *el)
            .unwrap_or_else(|| *document.root_element());
        walk(node, &mut out);
    }

    Ok(clean(&out))
}

fn walk(node: NodeRef<'_, Node>, out: &mut String) {
    match node.value() {
        Node::Text(t) => {
            let s: &str = t.as_ref();
            let norm = s.split_whitespace().collect::<Vec<_>>().join(" ");
            if !norm.is_empty() {
                out.push_str(&norm);
                out.push(' ');
            }
        }
        Node::Element(el) => {
            let tag = el.name();
            if SKIP.contains(&tag) {
                return;
            }
            match tag {
                "h1" => heading(node, out, 1),
                "h2" => heading(node, out, 2),
                "h3" => heading(node, out, 3),
                "h4" | "h5" | "h6" => heading(node, out, 4),
                "p" => {
                    out.push_str("\n\n");
                    walk_children(node, out);
                    out.push_str("\n\n");
                }
                "br" => out.push('\n'),
                "hr" => out.push_str("\n\n---\n\n"),
                "strong" | "b" => {
                    out.push_str("**");
                    walk_children(node, out);
                    out.push_str("** ");
                }
                "em" | "i" => {
                    out.push('*');
                    walk_children(node, out);
                    out.push_str("* ");
                }
                "code" => {
                    let parent_is_pre = node
                        .parent()
                        .and_then(|p| p.value().as_element())
                        .map(|e| e.name() == "pre")
                        .unwrap_or(false);
                    if !parent_is_pre {
                        out.push('`');
                        out.push_str(raw_text(node).trim());
                        out.push_str("` ");
                    }
                }
                "pre" => {
                    let lang = code_lang(node);
                    out.push_str(&format!("\n\n```{lang}\n"));
                    out.push_str(raw_text(node).trim());
                    out.push_str("\n```\n\n");
                }
                "a" => {
                    let href = el.attr("href").unwrap_or("").to_string();
                    let mut text = String::new();
                    walk_children(node, &mut text);
                    let text = text.trim().to_string();
                    if href.is_empty() || text.is_empty() {
                        out.push_str(&text);
                    } else {
                        out.push_str(&format!("[{text}]({href}) "));
                    }
                }
                "ul" => {
                    out.push('\n');
                    list_items(node, out, false);
                    out.push('\n');
                }
                "ol" => {
                    out.push('\n');
                    list_items(node, out, true);
                    out.push('\n');
                }
                "li" => walk_children(node, out),
                "table" => {
                    out.push('\n');
                    out.push_str(&table_to_md(node));
                    out.push('\n');
                }
                // table internals handled only via table_to_md
                "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" => {}
                "blockquote" => {
                    let mut bq = String::new();
                    walk_children(node, &mut bq);
                    for line in clean(&bq).lines() {
                        out.push_str(&format!("> {line}\n"));
                    }
                }
                "img" => {
                    if let Some(alt) = el.attr("alt") {
                        if !alt.is_empty() {
                            out.push_str(&format!("[image: {alt}] "));
                        }
                    }
                }
                // block-level containers
                "div" | "section" | "article" | "main" | "aside" | "header" | "footer"
                | "nav" | "figure" | "figcaption" | "details" | "summary" | "form"
                | "fieldset" => {
                    out.push('\n');
                    walk_children(node, out);
                    out.push('\n');
                }
                // inline / unknown: just walk
                _ => walk_children(node, out),
            }
        }
        _ => {}
    }
}

// ── ヘルパー ────────────────────────────────────────────────────────────

fn heading(node: NodeRef<'_, Node>, out: &mut String, level: usize) {
    let mut text = String::new();
    walk_children(node, &mut text);
    let text = text.trim().to_string();
    if !text.is_empty() {
        let prefix = "#".repeat(level);
        out.push_str(&format!("\n\n{prefix} {text}\n\n"));
    }
}

fn walk_children(node: NodeRef<'_, Node>, out: &mut String) {
    for child in node.children() {
        walk(child, out);
    }
}

fn list_items(node: NodeRef<'_, Node>, out: &mut String, ordered: bool) {
    let mut i = 1usize;
    for child in node.children() {
        if let Node::Element(el) = child.value() {
            if el.name() == "li" {
                let mut text = String::new();
                walk_children(child, &mut text);
                let text = text.trim().to_string();
                if !text.is_empty() {
                    if ordered {
                        out.push_str(&format!("{i}. {text}\n"));
                        i += 1;
                    } else {
                        out.push_str(&format!("- {text}\n"));
                    }
                }
            }
        }
    }
}

/// `pre` / `code` ブロックからテキストをそのまま取得（空白保持）
fn raw_text(node: NodeRef<'_, Node>) -> String {
    let mut out = String::new();
    collect_raw(node, &mut out);
    out
}

fn collect_raw(node: NodeRef<'_, Node>, out: &mut String) {
    for child in node.children() {
        match child.value() {
            Node::Text(t) => out.push_str(t),
            Node::Element(_) => collect_raw(child, out),
            _ => {}
        }
    }
}

/// `<pre>` の直下 `<code class="language-*">` から言語名を取得
fn code_lang(pre_node: NodeRef<'_, Node>) -> String {
    for child in pre_node.children() {
        if let Node::Element(el) = child.value() {
            if el.name() == "code" {
                if let Some(class) = el.attr("class") {
                    for cls in class.split_whitespace() {
                        if let Some(lang) = cls.strip_prefix("language-") {
                            return lang.to_string();
                        }
                    }
                }
            }
        }
    }
    String::new()
}

/// `<table>` を Markdown テーブル形式に変換
fn table_to_md(node: NodeRef<'_, Node>) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();
    collect_rows(node, &mut rows);

    if rows.is_empty() {
        return String::new();
    }
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }

    let mut out = String::new();
    for (i, row) in rows.iter().enumerate() {
        let cells: Vec<String> = (0..cols)
            .map(|j| row.get(j).cloned().unwrap_or_default())
            .collect();
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
        if i == 0 {
            out.push_str(&format!("| {} |\n", vec!["---"; cols].join(" | ")));
        }
    }
    out
}

fn collect_rows(node: NodeRef<'_, Node>, rows: &mut Vec<Vec<String>>) {
    for child in node.children() {
        if let Node::Element(el) = child.value() {
            match el.name() {
                "tr" => {
                    let mut cells = Vec::new();
                    for cell in child.children() {
                        if let Node::Element(cel) = cell.value() {
                            if matches!(cel.name(), "td" | "th") {
                                let mut text = String::new();
                                walk_children(cell, &mut text);
                                cells.push(text.trim().to_string());
                            }
                        }
                    }
                    if !cells.is_empty() {
                        rows.push(cells);
                    }
                }
                "thead" | "tbody" | "tfoot" => collect_rows(child, rows),
                _ => {}
            }
        }
    }
}

// ── 後処理 ──────────────────────────────────────────────────────────────

/// 連続空行を1行に圧縮し、各行末の余分な空白を除去
fn clean(s: &str) -> String {
    let mut result = String::new();
    let mut blank = 0usize;
    for line in s.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            blank += 1;
            if blank <= 1 {
                result.push('\n');
            }
        } else {
            blank = 0;
            result.push_str(trimmed);
            result.push('\n');
        }
    }
    result.trim().to_string()
}

fn truncate(s: &str, max_chars: usize) -> &str {
    if s.len() <= max_chars {
        return s;
    }
    let mut end = max_chars;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
