pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub provider: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            api_key: std::env::var("KAGENT_API_KEY")
                .map_err(|_| anyhow::anyhow!("KAGENT_API_KEY not set"))?,
            base_url: std::env::var("KAGENT_BASE_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            model: std::env::var("KAGENT_MODEL")
                .unwrap_or_else(|_| "claude-haiku-4-5".to_string()),
            provider: std::env::var("KAGENT_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
        })
    }
}
