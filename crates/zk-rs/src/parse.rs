use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::note::Note;

/// Typed representation of the supported zk frontmatter fields.
struct Frontmatter {
    title: Option<String>,
    date: Option<String>,
    tags: Vec<String>,
    aliases: Vec<String>,
}

impl Frontmatter {
    fn empty() -> Self {
        Self {
            title: None,
            date: None,
            tags: Vec::new(),
            aliases: Vec::new(),
        }
    }
}

/// Parse a note from its content and filesystem metadata.
pub(crate) fn parse_note(
    content: &str,
    path: PathBuf,
    abs_path: PathBuf,
    meta: &fs::Metadata,
) -> Result<Note, Error> {
    let (frontmatter_str, body_start_offset) = extract_frontmatter(content);

    let frontmatter = match frontmatter_str {
        Some(fm_str) => parse_yaml(fm_str, &abs_path)?,
        None => Frontmatter::empty(),
    };

    let content_after_frontmatter = &content[body_start_offset..];

    let (title_ref, body_ref) = extract_title_and_body(content_after_frontmatter, &frontmatter);
    let title = title_ref.to_owned();
    let lead = extract_lead(body_ref).to_owned();
    let body = body_ref.to_owned();

    // Now that title/body are owned, frontmatter borrow is released.
    let modified = timestamp_from_mtime(meta);
    let created = extract_created(&frontmatter)
        .unwrap_or_else(|| timestamp_from_birthtime(meta).unwrap_or(modified));

    // Merge frontmatter tags with inline tags (colon tags and hashtags) from body.
    let inline_tags = extract_inline_tags(content_after_frontmatter);
    let mut tags = frontmatter.tags;
    let mut seen: HashSet<String> = tags.iter().map(|t| t.to_lowercase()).collect();
    for tag in inline_tags {
        let lower = tag.to_lowercase();
        if seen.insert(lower) {
            tags.push(tag);
        }
    }

    let outgoing_links = extract_wiki_link_stems(&body);

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();

    let filename_stem = path
        .file_stem()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();

    let link = format!("[{title}]({})", path.display());

    let word_count = content.split_whitespace().count();

    Ok(Note {
        filename,
        filename_stem,
        path,
        abs_path,
        title,
        link,
        lead,
        body,
        raw_content: content.to_owned(),
        word_count,
        tags,
        aliases: frontmatter.aliases,
        outgoing_links,
        created,
        modified,
    })
}

/// Extract the YAML frontmatter block if present.
/// Returns (Some(frontmatter_content), offset_after_closing_delimiter) or (None, 0).
fn extract_frontmatter(content: &str) -> (Option<&str>, usize) {
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        return (None, 0);
    }

    let after_opening = if content.starts_with("---\r\n") { 5 } else { 4 };

    // Find the closing ---
    if let Some(pos) = content[after_opening..].find("\n---\n") {
        let fm = &content[after_opening..after_opening + pos];
        let end = after_opening + pos + 5; // skip past \n---\n
        (Some(fm), end)
    } else if let Some(pos) = content[after_opening..].find("\n---\r\n") {
        let fm = &content[after_opening..after_opening + pos];
        let end = after_opening + pos + 6;
        (Some(fm), end)
    } else if content[after_opening..].ends_with("\n---") {
        // File ends right after closing delimiter (no trailing newline)
        let fm = &content[after_opening..content.len() - 4];
        (Some(fm), content.len())
    } else {
        // No closing delimiter found -- treat entire content as body
        (None, 0)
    }
}

/// Parse YAML frontmatter into typed fields, normalizing keys to lowercase.
fn parse_yaml(yaml_str: &str, file_path: &Path) -> Result<Frontmatter, Error> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml_str).map_err(|source| Error::Yaml {
            path: file_path.to_string_lossy().into_owned(),
            source,
        })?;

    let map = match value {
        serde_yaml::Value::Mapping(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    result.insert(key.to_lowercase(), v);
                }
            }
            result
        }
        _ => return Ok(Frontmatter::empty()),
    };

    let title = match map.get("title") {
        Some(serde_yaml::Value::String(s)) => Some(s.clone()),
        _ => None,
    };

    let date = match map.get("date") {
        Some(serde_yaml::Value::String(s)) => Some(s.clone()),
        _ => None,
    };

    let tags = extract_tags(&map);
    let aliases = extract_string_list(map.get("aliases"));

    Ok(Frontmatter {
        title,
        date,
        tags,
        aliases,
    })
}

/// Extract title and body from content after frontmatter.
/// Title comes from frontmatter `title` key, or first markdown heading.
fn extract_title_and_body<'a>(
    content: &'a str,
    frontmatter: &'a Frontmatter,
) -> (&'a str, &'a str) {
    // Check frontmatter for title
    if let Some(title) = &frontmatter.title {
        return (title.as_str(), content.trim());
    }

    // Fall back to first markdown heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(heading) = extract_heading(trimmed) {
            // Body is everything after this heading line
            if let Some(pos) = content.find(line) {
                let after_heading = &content[pos + line.len()..];
                let body = after_heading.trim_start_matches(['\n', '\r']).trim();
                return (heading, body);
            }
        }
        // First non-empty, non-heading line -- no title found
        break;
    }

    ("", content.trim())
}

/// Extract heading text from a markdown heading line.
fn extract_heading(line: &str) -> Option<&str> {
    if line.starts_with('#') {
        let stripped = line.trim_start_matches('#');
        if stripped.is_empty() || stripped.starts_with(' ') {
            return Some(stripped.trim());
        }
    }
    None
}

/// Extract lead: body up to first blank line.
fn extract_lead(body: &str) -> &str {
    if body.is_empty() {
        return "";
    }
    // Split on double newline (blank line)
    if let Some(pos) = body.find("\n\n") {
        body[..pos].trim()
    } else if let Some(pos) = body.find("\r\n\r\n") {
        body[..pos].trim()
    } else {
        body.trim()
    }
}

/// Extract tags from frontmatter. Checks keys: tags, tag, keywords, keyword.
/// Handles YAML sequences and space-separated strings. Strips `#` prefixes, deduplicates.
fn extract_tags(map: &HashMap<String, serde_yaml::Value>) -> Vec<String> {
    let tag_keys = ["tags", "tag", "keywords", "keyword"];
    let mut tags = Vec::new();
    let mut seen = HashSet::new();

    for key in &tag_keys {
        if let Some(value) = map.get(*key) {
            collect_tags(value, &mut tags, &mut seen);
        }
    }

    tags
}

fn collect_tags(value: &serde_yaml::Value, tags: &mut Vec<String>, seen: &mut HashSet<String>) {
    match value {
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                if let serde_yaml::Value::String(s) = item {
                    add_tag(s, tags, seen);
                }
            }
        }
        serde_yaml::Value::String(s) => {
            // Space-separated tags
            for part in s.split_whitespace() {
                add_tag(part, tags, seen);
            }
        }
        _ => {}
    }
}

fn add_tag(raw: &str, tags: &mut Vec<String>, seen: &mut HashSet<String>) {
    let tag = raw.trim_start_matches('#').trim();
    if !tag.is_empty() {
        let lower = tag.to_lowercase();
        if seen.insert(lower) {
            tags.push(tag.to_owned());
        }
    }
}

/// Extract inline tags from note content: colon-separated (`:tag1:tag2:`) and hashtags (`#tag`).
/// Skips fenced code blocks.
fn extract_inline_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        extract_colon_tags(line, &mut tags);
        extract_hashtags(line, &mut tags);
    }

    tags
}

fn is_tag_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// Extract `:colon:separated:tags:` from a line.
fn extract_colon_tags(line: &str, tags: &mut Vec<String>) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b':' && (i == 0 || bytes[i - 1].is_ascii_whitespace()) {
            i += 1; // skip opening :

            // Parse one or more tag names separated by colons: :tag1:tag2:
            loop {
                let tag_start = i;
                while i < len && is_tag_char(bytes[i]) {
                    i += 1;
                }

                if i == tag_start {
                    // Empty tag name — not a valid colon tag sequence
                    break;
                }

                if i >= len || bytes[i] != b':' {
                    // No closing colon — not a valid colon tag
                    break;
                }

                tags.push(line[tag_start..i].to_owned());
                i += 1; // skip closing :

                // If next char is not a tag char, the sequence ended
                if i >= len || !is_tag_char(bytes[i]) {
                    break;
                }
            }
        } else {
            i += 1;
        }
    }
}

/// Extract `#hashtags` from a line.
fn extract_hashtags(line: &str, tags: &mut Vec<String>) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'#' && (i == 0 || bytes[i - 1].is_ascii_whitespace()) {
            let start = i + 1;
            let mut end = start;
            while end < len && is_tag_char(bytes[end]) {
                end += 1;
            }
            if end > start {
                tags.push(line[start..end].to_owned());
                i = end;
                continue;
            }
        }
        i += 1;
    }
}

/// Check if a markdown link URL is a wiki-link (bare stem or ./stem or ../stem).
/// Mirrors the WIKI_LINK_RE pattern in weave/src/md.rs.
fn wiki_link_stem(url: &str) -> Option<&str> {
    let mut rest = url;
    loop {
        if rest.starts_with("../") {
            rest = &rest[3..];
        } else if rest.starts_with("./") {
            rest = &rest[2..];
        } else {
            break;
        }
    }
    if !rest.is_empty() && rest.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(rest)
    } else {
        None
    }
}

/// Extract all wiki-link stems from a markdown body.
/// Scans for `](url)` patterns; skips fenced code blocks.
fn extract_wiki_link_stems(body: &str) -> Vec<String> {
    let mut stems = Vec::new();
    let mut in_code_block = false;

    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let mut rest = line;
        while let Some(pos) = rest.find("](") {
            rest = &rest[pos + 2..];
            let end = rest.find([')', '\n']).unwrap_or(rest.len());
            if let Some(stem) = wiki_link_stem(&rest[..end]) {
                stems.push(stem.to_owned());
            }
            rest = &rest[end..];
        }
    }
    stems
}

/// Extract a list of strings from a YAML value (sequence or single string).
fn extract_string_list(value: Option<&serde_yaml::Value>) -> Vec<String> {
    match value {
        Some(serde_yaml::Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| match v {
                serde_yaml::Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        Some(serde_yaml::Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

/// Parse the `date` field from frontmatter as a jiff::Timestamp.
fn extract_created(frontmatter: &Frontmatter) -> Option<jiff::Timestamp> {
    parse_date_string(frontmatter.date.as_deref()?)
}

/// Try multiple date formats to parse a string into a jiff::Timestamp.
fn parse_date_string(s: &str) -> Option<jiff::Timestamp> {
    let s = s.trim();

    // Try full ISO 8601 / RFC 3339 (e.g. "2024-01-15T10:30:00Z" or with offset)
    if let Ok(ts) = s.parse::<jiff::Timestamp>() {
        return Some(ts);
    }

    // Try as civil datetime "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DDTHH:MM:SS"
    if let Ok(dt) = s.parse::<jiff::civil::DateTime>()
        && let Ok(ts) = dt.to_zoned(jiff::tz::TimeZone::UTC)
    {
        return Some(ts.timestamp());
    }

    // Try as civil date "YYYY-MM-DD"
    if let Ok(d) = s.parse::<jiff::civil::Date>()
        && let Ok(ts) = d.to_zoned(jiff::tz::TimeZone::UTC)
    {
        return Some(ts.timestamp());
    }

    None
}

fn timestamp_from_mtime(meta: &fs::Metadata) -> jiff::Timestamp {
    meta.modified()
        .ok()
        .and_then(|st| jiff::Timestamp::try_from(st).ok())
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
}

fn timestamp_from_birthtime(meta: &fs::Metadata) -> Option<jiff::Timestamp> {
    meta.created()
        .ok()
        .and_then(|st| jiff::Timestamp::try_from(st).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\ntitle: Hello\ntags: [a, b]\n---\n# Hello\n\nBody here.";
        let (fm, offset) = extract_frontmatter(content);
        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert!(fm.contains("title: Hello"));
        assert_eq!(&content[offset..], "# Hello\n\nBody here.");
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a heading\n\nSome body.";
        let (fm, offset) = extract_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_extract_heading() {
        assert_eq!(extract_heading("# Hello World"), Some("Hello World".into()));
        assert_eq!(
            extract_heading("## Sub heading"),
            Some("Sub heading".into())
        );
        assert_eq!(extract_heading("Not a heading"), None);
        assert_eq!(extract_heading("#nospace"), None);
    }

    #[test]
    fn test_extract_tags_sequence() {
        let mut map = HashMap::new();
        map.insert(
            "tags".to_string(),
            serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("rust".into()),
                serde_yaml::Value::String("#coding".into()),
            ]),
        );
        let tags = extract_tags(&map);
        assert_eq!(tags, vec!["rust", "coding"]);
    }

    #[test]
    fn test_extract_tags_string() {
        let mut map = HashMap::new();
        map.insert(
            "tags".to_string(),
            serde_yaml::Value::String("rust #coding notes".into()),
        );
        let tags = extract_tags(&map);
        assert_eq!(tags, vec!["rust", "coding", "notes"]);
    }

    #[test]
    fn test_extract_tags_dedup() {
        let mut map = HashMap::new();
        map.insert(
            "tags".to_string(),
            serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("rust".into()),
                serde_yaml::Value::String("Rust".into()),
            ]),
        );
        let tags = extract_tags(&map);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "rust");
    }

    #[test]
    fn test_extract_lead() {
        let body = "First paragraph here.\n\nSecond paragraph.";
        assert_eq!(extract_lead(body), "First paragraph here.");
    }

    #[test]
    fn test_extract_lead_no_blank_line() {
        let body = "Only one paragraph.";
        assert_eq!(extract_lead(body), "Only one paragraph.");
    }

    #[test]
    fn test_parse_date_string() {
        assert!(parse_date_string("2024-01-15").is_some());
        assert!(parse_date_string("2024-01-15 10:30:00").is_some());
        assert!(parse_date_string("2024-01-15T10:30:00Z").is_some());
        assert!(parse_date_string("not a date").is_none());
    }

    #[test]
    fn test_title_from_frontmatter() {
        let fm = Frontmatter {
            title: Some("FM Title".into()),
            date: None,
            tags: Vec::new(),
            aliases: Vec::new(),
        };
        let (title, body) = extract_title_and_body("# Heading\n\nBody text", &fm);
        assert_eq!(title, "FM Title");
        assert_eq!(body, "# Heading\n\nBody text");
    }

    #[test]
    fn test_title_from_heading() {
        let fm = Frontmatter::empty();
        let (title, body) = extract_title_and_body("# My Title\n\nSome body.", &fm);
        assert_eq!(title, "My Title");
        assert_eq!(body, "Some body.");
    }

    #[test]
    fn test_colon_tags() {
        let mut tags = Vec::new();
        extract_colon_tags(":rust:", &mut tags);
        assert_eq!(tags, vec!["rust"]);
    }

    #[test]
    fn test_colon_tags_chained() {
        let mut tags = Vec::new();
        extract_colon_tags(":se:programming:", &mut tags);
        assert_eq!(tags, vec!["se", "programming"]);
    }

    #[test]
    fn test_colon_tags_ignores_mid_line() {
        // Colon after non-whitespace (like URLs) should be skipped.
        let mut tags = Vec::new();
        extract_colon_tags("see https://example.com:8080/path", &mut tags);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_hashtags() {
        let mut tags = Vec::new();
        extract_hashtags("#rust #coding", &mut tags);
        assert_eq!(tags, vec!["rust", "coding"]);
    }

    #[test]
    fn test_hashtags_not_headings() {
        // Markdown heading should not produce a tag.
        let mut tags = Vec::new();
        extract_hashtags("# Heading", &mut tags);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_inline_tags_skips_code_blocks() {
        let content = "body\n\n```\n:code:tag:\n#codetag\n```\n\n:real:";
        let tags = extract_inline_tags(content);
        assert_eq!(tags, vec!["real"]);
    }

    #[test]
    fn test_inline_tags_real_note() {
        // Mimics the actual note format found in the user's zettelkasten.
        let content = "# Expression problem\n\nA challenge in programming.\n\n:se:programming:";
        let tags = extract_inline_tags(content);
        assert_eq!(tags, vec!["se", "programming"]);
    }

    #[test]
    fn test_wiki_link_stem_bare() {
        assert_eq!(wiki_link_stem("abc123"), Some("abc123"));
    }

    #[test]
    fn test_wiki_link_stem_relative() {
        assert_eq!(wiki_link_stem("./abc123"), Some("abc123"));
        assert_eq!(wiki_link_stem("../abc123"), Some("abc123"));
        assert_eq!(wiki_link_stem("../../abc123"), Some("abc123"));
    }

    #[test]
    fn test_wiki_link_stem_rejects_url() {
        assert_eq!(wiki_link_stem("https://example.com"), None);
        assert_eq!(wiki_link_stem("file.md"), None);
    }

    #[test]
    fn test_extract_wiki_link_stems_basic() {
        let body = "See [note one](abc) and [note two](./def).";
        let stems = extract_wiki_link_stems(body);
        assert_eq!(stems, vec!["abc", "def"]);
    }

    #[test]
    fn test_extract_wiki_link_stems_skips_urls() {
        let body = "See [external](https://example.com) and [note](abc).";
        let stems = extract_wiki_link_stems(body);
        assert_eq!(stems, vec!["abc"]);
    }

    #[test]
    fn test_extract_wiki_link_stems_skips_code_blocks() {
        let body = "Before.\n\n```\n[code](link_in_code)\n```\n\n[real](abc).";
        let stems = extract_wiki_link_stems(body);
        assert_eq!(stems, vec!["abc"]);
    }
}
