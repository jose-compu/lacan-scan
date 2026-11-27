//! AIS008: pipe-to-shell only on lines that can execute.

use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static PIPE_SHELL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(curl|wget)[^\n]*\|\s*(ba)?sh").unwrap());

fn is_script_extension(path: &str) -> bool {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "sh" | "bash" | "zsh" | "yml" | "yaml" | "md" | "markdown" | "ipynb"
    )
}

fn skip_asset_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.contains("swagger-ui") && lower.ends_with(".js") {
        return true;
    }
    if lower.contains("/docs/") && lower.ends_with(".min.js") {
        return true;
    }
    false
}

fn help_or_doc_string_line(line: &str) -> bool {
    let trimmed = line.trim();
    (trimmed.starts_with('"') || trimmed.starts_with('\''))
        && PIPE_SHELL.is_match(line)
        && !line.contains("subprocess")
        && !line.contains("os.system")
        && !line.contains("child_process")
        && (line.contains("\\n")
            || line.contains("install_instructions")
            || line.contains("FileNotFoundError")
            || line.contains("`"))
}

/// True when `line` (1-based) is an executable pipe-to-shell risk for this path.
pub fn line_is_pipe_shell_risk(path: &str, text: &str, line: i64) -> bool {
    if skip_asset_path(path) {
        return false;
    }
    let idx = (line - 1).max(0) as usize;
    let Some(raw) = text.lines().nth(idx) else {
        return false;
    };
    if !PIPE_SHELL.is_match(raw) {
        return false;
    }
    if is_script_extension(path) {
        return true;
    }
    !help_or_doc_string_line(raw)
}

/// First risky pipe-to-shell line for AIS008 supplemental filtering, or 0.
pub fn pipe_to_shell_risk_line(path: &str, text: &str) -> i64 {
    if skip_asset_path(path) {
        return 0;
    }
    for (idx, line) in text.lines().enumerate() {
        if !PIPE_SHELL.is_match(line) {
            continue;
        }
        if is_script_extension(path) {
            return (idx as i64) + 1;
        }
        if help_or_doc_string_line(line) {
            continue;
        }
        return (idx as i64) + 1;
    }
    0
}