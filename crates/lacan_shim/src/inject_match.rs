//! AIS102 / AIS103 line helpers (skip comments, locales, defensive pattern lists).

use regex::Regex;
use std::sync::LazyLock;

static AIS102_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)ignore (all )?(previous|prior|above) instructions").unwrap(),
        Regex::new(r"(?i)disregard (all )?(previous|prior) (instructions|prompts)").unwrap(),
        Regex::new(r"(?i)you are now (in )?(developer|DAN|jailbreak) mode").unwrap(),
    ]
});

static AIS103_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"<\|im_start\|>").unwrap(),
        Regex::new(r"<\|im_end\|>").unwrap(),
        Regex::new(r"<\|eot_id\|>").unwrap(),
        Regex::new(r"\[INST\]").unwrap(),
        Regex::new(r"<<SYS>>").unwrap(),
    ]
});

fn path_is_locale_bundle(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("/translations/") || lower.contains("/i18n/") || lower.ends_with("/locale.json")
}

fn line_is_comment_or_doc(path: &str, line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#')
        || trimmed.starts_with("//")
        || trimmed.starts_with('*')
        || trimmed.starts_with("/*")
    {
        return true;
    }
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(ext.as_str(), "md" | "rst" | "txt")
        && !trimmed.starts_with("```")
        && trimmed.starts_with('>')
}

fn near_defensive_pattern_list(lines: &[&str], idx: usize) -> bool {
    let start = idx.saturating_sub(8);
    let window = &lines[start..=idx.min(lines.len() - 1)];
    window.iter().any(|l| {
        let upper = l.to_ascii_uppercase();
        upper.contains("_PATTERNS")
            || upper.contains("BLOCKLIST")
            || upper.contains("THREAT")
            || upper.contains("PROMPT_INJECTION")
    })
}

/// First AIS102 hit line, or 0.
pub fn instruction_override_line(path: &str, text: &str) -> i64 {
    let lines: Vec<&str> = text.lines().collect();
    if path_is_locale_bundle(path) {
        return 0;
    }
    for (idx, line) in lines.iter().enumerate() {
        if line_is_comment_or_doc(path, line) && near_defensive_pattern_list(&lines, idx) {
            continue;
        }
        let t = line.trim();
        if (t.starts_with('"') || t.starts_with('\'')) && near_defensive_pattern_list(&lines, idx) {
            continue;
        }
        for pat in AIS102_PATTERNS.iter() {
            if pat.is_match(line) {
                return (idx as i64) + 1;
            }
        }
    }
    0
}

/// First AIS103 hit line skipping comments, or 0.
pub fn control_token_line(path: &str, text: &str) -> i64 {
    for (idx, line) in text.lines().enumerate() {
        if line_is_comment_or_doc(path, line) {
            continue;
        }
        for pat in AIS103_PATTERNS.iter() {
            if pat.is_match(line) {
                return (idx as i64) + 1;
            }
        }
    }
    0
}