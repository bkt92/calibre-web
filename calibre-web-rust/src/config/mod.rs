//! Configuration management
//!
//! Loads configuration from TOML files and environment variables.

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;

/// Database configuration
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of database connections
    pub max_connections: u32,
}

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port number to listen on
    pub port: u16,
    /// Number of worker threads
    pub workers: usize,
}

/// Library settings
#[derive(Debug, Deserialize, Clone)]
pub struct LibrarySettings {
    /// Path to the Calibre library
    pub library_path: String,
    /// Path to store cached book covers
    pub cover_path: String,
    /// Path to store uploaded files
    pub upload_path: String,
}

/// Main application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Library settings
    pub library: LibrarySettings,
}

impl AppConfig {
    /// Load configuration from default paths
    ///
    /// Looks for configuration files in the following order:
    /// 1. config/default.toml
    /// 2. config/local.toml (optional, overrides defaults)
    ///
    /// Environment variables with the `CALIBRE_WEB__` prefix can override
    /// any configuration value. For example:
    /// - `CALIBRE_WEB__SERVER__PORT=8083`
    /// - `CALIBRE_WEB__DATABASE__URL=postgresql://localhost/mydb`
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_paths(&["config/default.toml", "config/local.toml"])
    }

    /// Load configuration from specific paths
    ///
    /// Later files override earlier ones. Environment variables
    /// can override all file-based configuration.
    pub fn load_from_paths(paths: &[&str]) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();

        // Load config files in order (later files override earlier ones)
        for path in paths {
            if Path::new(path).exists() {
                builder = builder.add_source(File::with_name(path));
            }
        }

        // Override with environment variables (CALIBRE_WEB__ prefix)
        builder = builder.add_source(
            Environment::with_prefix("CALIBRE_WEB")
                .prefix_separator("__")
                .separator("__"),
        );

        builder.build()?.try_deserialize()
    }

    /// Load configuration from a single file path
    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from(path))
            .build()?
            .try_deserialize()
    }
}
