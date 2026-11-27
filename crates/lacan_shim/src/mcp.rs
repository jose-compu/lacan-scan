//! AIS006: known vendor MCP HTTPS endpoints.

/// True when an MCP config chunk points at an allowlisted vendor HTTPS endpoint.
pub fn mcp_remote_url_allowed(chunk: &str) -> bool {
    let lower = chunk.to_ascii_lowercase();
  lower.contains("mcp.linear.app")
        || lower.contains("mcp.cursor.com")
        || lower.contains("mcp.figma.com")
}

#[cfg(test)]
mod tests {
    use super::mcp_remote_url_allowed;

    #[test]
    fn linear_app_allowed() {
        assert!(mcp_remote_url_allowed(
            r#""url": "https://mcp.linear.app/mcp""#
        ));
    }
}