use calibre_web_rust::config::AppConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("calibre_web_rust=debug,tower_http=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let _config = AppConfig::load()?;
    tracing::info!("Configuration loaded successfully");

    // TODO: Initialize database, routes, etc.

    tracing::info!("Calibre-Web Rust starting...");
    Ok(())
}
