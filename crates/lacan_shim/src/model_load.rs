// lacan:skip=AIS203:pattern literals and unit tests in this module
//! AIS203: Hugging Face `from_pretrained` / `trust_remote_code` at real call sites.

use regex::Regex;
use std::sync::LazyLock;
use tree_sitter::{Node, Parser};

fn file_uses_hf_stack(text: &str) -> bool {
    text.contains("transformers") || text.contains("huggingface_hub")
}

fn hf_receiver_name(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.starts_with("auto")
        || lower == "pipeline"
        || lower.contains("pretrainedmodel")
}

fn call_is_pinned(call_text: &str) -> bool {
    call_text.contains("revision")
        || call_text.contains("commit_hash")
        || call_text.contains("hub_kwargs")
}

fn trust_remote_literal_true(line: &str) -> bool {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"trust_remote_code\s*=\s*True\b").unwrap());
    RE.is_match(line)
        && !line.contains("trust_remote_code=self")
        && !line.contains("trust_remote_code=config")
}

fn node_in_docstring(node: Node, _src: &[u8]) -> bool {
    let mut cur = node;
    while let Some(parent) = cur.parent() {
        if matches!(
            parent.kind(),
            "module" | "class_definition" | "function_definition"
        ) {
            if let Some(body) = parent.child_by_field_name("body") {
                if let Some(first) = body.named_child(0) {
                    if first.kind() == "expression_statement" {
                        if let Some(child) = first.named_child(0) {
                            if matches!(child.kind(), "string" | "concatenated_string") {
                                if node.start_byte() >= child.start_byte()
                                    && node.end_byte() <= child.end_byte()
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        cur = parent;
    }
    false
}

fn scan_hf_from_pretrained(text: &str) -> i64 {
    if !file_uses_hf_stack(text) {
        return 0;
    }
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .is_err()
    {
        return 0;
    }
    let Some(tree) = parser.parse(text, None) else {
        return 0;
    };
    let src = text.as_bytes();
    walk_hf(tree.root_node(), src)
}

fn walk_hf(node: Node, src: &[u8]) -> i64 {
    if node.kind() == "call" {
        if let Some(func) = node.child_by_field_name("function") {
            if func.kind() == "attribute" {
                if let Some(method) = func.child_by_field_name("attribute") {
                    if method.utf8_text(src).ok() == Some("from_pretrained") {
                        if let Some(recv) = func.child_by_field_name("object") {
                            if let Ok(recv_text) = recv.utf8_text(src) {
                                if hf_receiver_name(recv_text) && !node_in_docstring(node, src) {
                                    if let Ok(call_text) = node.utf8_text(src) {
                                        if !call_is_pinned(call_text) {
                                            return node.start_position().row as i64 + 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let hit = walk_hf(child, src);
        if hit > 0 {
            return hit;
        }
    }
    0
}

/// First unpinned Hugging Face `from_pretrained` call line in a Python file, or 0.
pub fn hf_unpinned_from_pretrained_line(path: &str, text: &str) -> i64 {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if ext != "py" {
        return 0;
    }
    if !text.contains("from_pretrained") || !file_uses_hf_stack(text) {
        return 0;
    }
    scan_hf_from_pretrained(text)
}

/// First line with literal `trust_remote_code=True`, or 0.
pub fn hf_trust_remote_code_line(path: &str, text: &str) -> i64 {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if ext != "py" {
        return 0;
    }
    for (idx, line) in text.lines().enumerate() {
        if trust_remote_literal_true(line) {
            return (idx as i64) + 1;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::{hf_trust_remote_code_line, hf_unpinned_from_pretrained_line};

    #[test]
    fn vertex_from_pretrained_clean() {
        let src = "from vertexai import TextEmbeddingModel\nm = TextEmbeddingModel.from_pretrained('id')\n";
        assert_eq!(hf_unpinned_from_pretrained_line("m.py", src), 0);
    }

    #[test]
    fn hf_unpinned_hits() {
        let src = "from transformers import AutoModel\nm = AutoModel.from_pretrained('x/y')\n";
        assert_eq!(hf_unpinned_from_pretrained_line("m.py", src), 2);
    }

    #[test]
    fn hf_pinned_clean() {
        let src =
            "from transformers import AutoModel\nm = AutoModel.from_pretrained('x', revision='abc')\n";
        assert_eq!(hf_unpinned_from_pretrained_line("m.py", src), 0);
    }

    #[test]
    fn config_trust_remote_clean() {
        let src = "AutoModel.from_pretrained(n, trust_remote_code=self.config.trust_remote_code)\n";
        assert_eq!(hf_trust_remote_code_line("m.py", src), 0);
    }
}