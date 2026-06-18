use async_trait::async_trait;
use serde_json::{Value, json};

use super::Tool;

pub struct FetchUrl;

#[async_trait]
impl Tool for FetchUrl {
    fn name(&self) -> &str {
        "fetch_url"
    }

    fn description(&self) -> &str {
        "Fetch the content of a URL and return it as plain text. \
         HTML tags are stripped; useful for reading documentation, web pages, or APIs."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch."
                }
            },
            "required": ["url"]
        })
    }

    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        let url = args["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing 'url'"))?;

        let client = reqwest::Client::builder()
            .user_agent("kagent/0.1")
            .build()?;

        let response = client.get(url).send().await?;
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = response.text().await?;

        let text = if content_type.contains("text/html") {
            strip_html(&body)
        } else {
            body
        };

        Ok(json!({
            "status": status,
            "content_type": content_type,
            "text": text,
        }))
    }
}

pub struct FetchUrlJs;

#[async_trait]
impl Tool for FetchUrlJs {
    fn name(&self) -> &str {
        "fetch_url_js"
    }

    fn description(&self) -> &str {
        "Fetch a URL using a headless Chrome browser that executes JavaScript before \
         extracting the page text. Use this for SPAs or JS-rendered pages. \
         Requires Chrome or Chromium installed on the system."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to fetch." }
            },
            "required": ["url"]
        })
    }

    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        use chromiumoxide::{Browser, BrowserConfig};
        use futures::StreamExt;
        use tokio::time::{Duration, timeout};

        let url = args["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing 'url'"))?;

        let config = BrowserConfig::builder()
            .build()
            .map_err(|e| anyhow::anyhow!("browser config: {e}"))?;

        let (mut browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| anyhow::anyhow!("failed to launch Chrome (is it installed?): {e}"))?;

        tokio::spawn(async move { while handler.next().await.is_some() {} });

        let page = browser.new_page(url).await?;
        let _ = timeout(Duration::from_secs(10), page.wait_for_navigation()).await;

        let text: String = page
            .evaluate("document.body ? document.body.innerText : ''")
            .await?
            .into_value()?;

        browser.close().await?;

        Ok(json!({ "text": text }))
    }
}

fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script_or_style = false;
    let mut tag_buf = String::new();

    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                tag_buf.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let tag = tag_buf.trim().to_lowercase();
                if tag == "script" || tag == "style" {
                    in_script_or_style = true;
                } else if tag == "/script" || tag == "/style" {
                    in_script_or_style = false;
                }
                // Treat block-level closing tags as line breaks for readability
                if matches!(
                    tag.as_str(),
                    "/p" | "/div"
                        | "/li"
                        | "/h1"
                        | "/h2"
                        | "/h3"
                        | "/h4"
                        | "/h5"
                        | "/h6"
                        | "/tr"
                        | "/br"
                        | "br"
                        | "br/"
                ) {
                    out.push('\n');
                }
            }
            _ if in_tag => {
                tag_buf.push(ch);
            }
            _ if in_script_or_style => {}
            _ => out.push(ch),
        }
    }

    // Collapse runs of whitespace/blank lines
    let mut result = String::with_capacity(out.len());
    let mut prev_blank = false;
    for line in out.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank {
                result.push('\n');
            }
            prev_blank = true;
        } else {
            result.push_str(trimmed);
            result.push('\n');
            prev_blank = false;
        }
    }
    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}
