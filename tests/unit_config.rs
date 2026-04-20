use umami_cli::config::Config;

#[test]
fn config_default_is_empty() {
    let config = Config::default();
    assert!(config.server_url.is_none());
    assert!(config.token.is_none());
    assert!(config.username.is_none());
}

#[test]
fn config_roundtrip_serialize() {
    let config = Config {
        server_url: Some("https://analytics.example.com".into()),
        token: Some("jwt-token-here".into()),
        username: Some("admin".into()),
    };
    let serialized = toml::to_string_pretty(&config).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();
    assert_eq!(
        deserialized.server_url.as_deref(),
        Some("https://analytics.example.com")
    );
    assert_eq!(deserialized.token.as_deref(), Some("jwt-token-here"));
    assert_eq!(deserialized.username.as_deref(), Some("admin"));
}

#[test]
fn config_deserialize_partial() {
    let toml_str = r#"server_url = "https://example.com""#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.server_url.as_deref(), Some("https://example.com"));
    assert!(config.token.is_none());
    assert!(config.username.is_none());
}

#[test]
fn config_deserialize_empty() {
    let config: Config = toml::from_str("").unwrap();
    assert!(config.server_url.is_none());
}
