use async_trait::async_trait;
use serde_json::Value;

use super::base::BaseTool;

pub struct WebFetchTool {
    max_size: usize,
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self {
            max_size: 100_000,
        }
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

pub fn html_to_text(html: &str) -> String {
    let mut text = html.to_string();
    // Strip tags
    let re = regex::Regex::new(r"<script[^>]*>[\s\S]*?</script>").unwrap();
    text = re.replace_all(&text, "").to_string();
    let re = regex::Regex::new(r"<style[^>]*>[\s\S]*?</style>").unwrap();
    text = re.replace_all(&text, "").to_string();
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    text = re.replace_all(&text, "").to_string();
    // Decode common entities
    text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    // Collapse whitespace
    let re = regex::Regex::new(r"\n{3,}").unwrap();
    text = re.replace_all(&text, "\n\n").to_string();
    text.trim().to_string()
}

#[async_trait]
impl BaseTool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch a URL and return its content as text."
    }

    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "The URL to fetch"},
                "format": {"type": "string", "description": "Output format: text, html, or markdown"}
            },
            "required": ["url"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'url' argument")?;

        let format = args
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url.to_string()
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; WebFetchTool/1.0)")
            .build()?;

        let resp = client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Ok(format!("HTTP Error: {}", resp.status()));
        }

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = resp.text().await?;

        let body = if body.len() > self.max_size {
            body[..self.max_size].to_string()
        } else {
            body
        };

        let result = if content_type.contains("text/html") && format != "html" {
            html_to_text(&body)
        } else {
            body
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_text_basic() {
        let html = "<html><body><p>Hello <b>World</b></p></body></html>";
        let result = html_to_text(html);
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
        assert!(!result.contains("<p>"));
    }

    #[test]
    fn test_html_to_text_strips_script() {
        let html = "<script>alert('x')</script><p>Content</p>";
        let result = html_to_text(html);
        assert!(!result.contains("alert"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_html_to_text_entities() {
        let html = "&amp; &lt; &gt; &quot;";
        let result = html_to_text(html);
        assert!(result.contains("& < > \""));
    }

    #[test]
    fn test_webfetch_schema() {
        let tool = WebFetchTool::new();
        assert_eq!(tool.name(), "web_fetch");
        let schema = tool.params_schema();
        assert!(schema["properties"]["url"].is_object());
    }
}
