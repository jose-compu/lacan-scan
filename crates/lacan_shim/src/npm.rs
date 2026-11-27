//! AIS004: npm lifecycle hooks inside package.json `scripts` only.

use regex::Regex;
use std::sync::LazyLock;

static SCRIPT_HOOK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)"(preinstall|postinstall|install)"\s*:"#).unwrap());

fn scripts_block(text: &str) -> Option<(usize, &str)> {
    let lower = text.to_ascii_lowercase();
    let key = "\"scripts\"";
    let start = lower.find(key)?;
    let after_key = &text[start + key.len()..];
    let brace = after_key.find('{')?;
    let body_start = start + key.len() + brace + 1;
    let rest = &text[body_start..];
    let mut depth = 1usize;
    let mut end = 0usize;
    for (i, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    Some((body_start, &rest[..end]))
}

fn line_global_number(text: &str, block_start: usize, block: &str, line_in_block: usize) -> i64 {
    let offset = block
        .lines()
        .take(line_in_block)
        .map(|l| l.len() + 1)
        .sum::<usize>();
    let abs = block_start + offset;
    let line = text[..abs].matches('\n').count() + 1;
    line as i64
}

fn script_value_line(block_line: &str) -> Option<&str> {
    let colon = block_line.find(':')?;
    let rest = block_line[colon + 1..].trim();
    if rest.starts_with('"') {
        let inner = rest.trim_start_matches('"');
        let end = inner.find('"')?;
        Some(&inner[..end])
    } else {
        None
    }
}

fn risky_lifecycle_script(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.contains("curl ")
        || lower.contains("wget ")
        || lower.contains("http://")
        || lower.contains("https://")
        || lower.contains("base64")
        || lower.contains("| sh")
        || lower.contains("| bash")
    {
        return true;
    }
    if lower.contains("only-allow") {
        return false;
    }
    if lower.starts_with("node -e") || lower.starts_with("node -e ") {
        return false;
    }
    if lower.contains("npx only-allow") {
        return false;
    }
    true
}

/// First 1-based line of a risky npm lifecycle hook inside `scripts`, or 0.
pub fn npm_lifecycle_hook_line(text: &str) -> i64 {
    let Some((start, block)) = scripts_block(text) else {
        return 0;
    };
    for (i, line) in block.lines().enumerate() {
        if !SCRIPT_HOOK.is_match(line) {
            continue;
        }
        let risky = script_value_line(line)
            .map(risky_lifecycle_script)
            .unwrap_or(true);
        if risky {
            return line_global_number(text, start, block, i);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::npm_lifecycle_hook_line;

    #[test]
    fn dependency_named_install_is_clean() {
        let pkg = r#"{
  "dependencies": { "install": "^0.13.0" },
  "scripts": { "build": "echo ok" }
}"#;
        assert_eq!(npm_lifecycle_hook_line(pkg), 0);
    }

    #[test]
    fn scripts_install_hits() {
        let pkg = r#"{
  "scripts": { "install": "node-gyp rebuild" }
}"#;
        assert!(npm_lifecycle_hook_line(pkg) > 0);
    }

    #[test]
    fn preinstall_only_allow_clean() {
        let pkg = r#"{
  "scripts": { "preinstall": "npx only-allow pnpm" }
}"#;
        assert_eq!(npm_lifecycle_hook_line(pkg), 0);
    }

    #[test]
    fn preinstall_node_e_clean() {
        let pkg = r#"{
  "scripts": { "preinstall": "node -e \"require('fs').exit(0)\"" }
}"#;
        assert_eq!(npm_lifecycle_hook_line(pkg), 0);
    }
}