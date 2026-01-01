use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
  pub host: String,
  pub port: u16,
  pub database_url: String,
}

impl AppConfig {
  pub fn from_env() -> Result<Self> {
    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("APP_PORT")
      .unwrap_or_else(|_| "8080".to_string())
      .parse::<u16>()
      .context("APP_PORT must be a u16")?;
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    Ok(Self { host, port, database_url })
  }
}


