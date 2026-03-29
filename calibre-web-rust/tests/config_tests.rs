use calibre_web_rust::config::AppConfig;
use tempfile::TempDir;

#[test]
fn test_load_default_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("default.toml");

    std::fs::write(
        &config_path,
        r#"
[database]
url = "postgresql://localhost/test"
max_connections = 10

[server]
host = "0.0.0.0"
port = 8083
workers = 4

[library]
library_path = "/tmp/library"
cover_path = "/tmp/covers"
upload_path = "/tmp/upload"
"#,
    ).unwrap();

    let config = AppConfig::load_from_path(&config_path).unwrap();
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 8083);
    assert_eq!(config.database.url, "postgresql://localhost/test");
    assert_eq!(config.database.max_connections, 10);
    assert_eq!(config.library.library_path, "/tmp/library");
    assert_eq!(config.library.cover_path, "/tmp/covers");
    assert_eq!(config.library.upload_path, "/tmp/upload");
}

#[test]
fn test_env_var_override() {
    use std::env;

    // Set environment variable
    env::set_var("CALIBRE_WEB__SERVER__PORT", "9999");

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    std::fs::write(
        &config_path,
        r#"
[database]
url = "postgresql://localhost/test"
max_connections = 10

[server]
host = "0.0.0.0"
port = 8083
workers = 4

[library]
library_path = "/tmp/library"
cover_path = "/tmp/covers"
upload_path = "/tmp/upload"
"#,
    ).unwrap();

    // Use load_from_paths which includes env var overrides
    let config = AppConfig::load_from_paths(&[config_path.to_str().unwrap()]).unwrap();
    // Env var should override file value
    assert_eq!(config.server.port, 9999);

    env::remove_var("CALIBRE_WEB__SERVER__PORT");
}

#[test]
fn test_file_override_behavior() {
    let temp_dir = TempDir::new().unwrap();
    let default_path = temp_dir.path().join("default.toml");
    let local_path = temp_dir.path().join("local.toml");

    // Write default config
    std::fs::write(
        &default_path,
        r#"
[database]
url = "postgresql://localhost/test"
max_connections = 10

[server]
host = "0.0.0.0"
port = 8083
workers = 4

[library]
library_path = "/tmp/library"
cover_path = "/tmp/covers"
upload_path = "/tmp/upload"
"#,
    ).unwrap();

    // Write local override
    std::fs::write(
        &local_path,
        r#"
[server]
port = 9999
"#,
    ).unwrap();

    let config = AppConfig::load_from_paths(&[
        default_path.to_str().unwrap(),
        local_path.to_str().unwrap(),
    ]).unwrap();

    // Local config should override default
    assert_eq!(config.server.port, 9999);
    // Default value should remain when not overridden
    assert_eq!(config.server.workers, 4);
}
