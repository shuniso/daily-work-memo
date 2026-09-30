use std::fs;
use std::path::PathBuf;

use daily_work_memo::config::{self, Config, ConfigError, DEFAULT_CONFIG_TOML};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dwm-config-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn entry(id: &str, label: &str, key: &str) -> String {
    format!("[[entry_types]]\nid = \"{id}\"\nlabel = \"{label}\"\nkey = \"{key}\"\n")
}

#[test]
fn default_toml_matches_builtin_default() {
    let config = Config::parse(DEFAULT_CONFIG_TOML).unwrap();
    assert_eq!(config, Config::default());
    assert_eq!(config.entry_types.len(), 7);
    assert!(config.entry_types.iter().all(|e| e.template.is_empty()));
}

#[test]
fn parses_custom_config() {
    let source = format!(
        "font_size = 18\nautosave_debounce_ms = 500\nentry_header = \"## {{label}} {{time}}\"\n{}template = \"- [ ] \"\n",
        entry("action", "やること", "a")
    );
    let config = Config::parse(&source).unwrap();
    assert_eq!(config.font_size, 18);
    assert_eq!(config.autosave_debounce_ms, 500);
    assert_eq!(config.entry_header, "## {label} {time}");
    assert_eq!(config.entry_types[0].template, "- [ ] ");
}

#[test]
fn omitted_fields_use_defaults() {
    let config = Config::parse("font_size = 20\n").unwrap();
    assert_eq!(config.font_size, 20);
    assert_eq!(config.entry_types, Config::default().entry_types);
}

#[test]
fn rejects_duplicate_id() {
    let source = entry("a", "A", "a") + &entry("a", "B", "b");
    assert!(matches!(
        Config::parse(&source),
        Err(ConfigError::Invalid(_))
    ));
}

#[test]
fn rejects_duplicate_key_ignoring_case() {
    let source = entry("a", "A", "a") + &entry("b", "B", "A");
    assert!(matches!(
        Config::parse(&source),
        Err(ConfigError::Invalid(_))
    ));
}

#[test]
fn rejects_empty_label() {
    let source = entry("a", " ", "a");
    assert!(matches!(
        Config::parse(&source),
        Err(ConfigError::Invalid(_))
    ));
}

#[test]
fn rejects_empty_id() {
    let source = entry("", "A", "a");
    assert!(matches!(
        Config::parse(&source),
        Err(ConfigError::Invalid(_))
    ));
}

#[test]
fn rejects_multi_char_key() {
    let source = entry("a", "A", "ab");
    assert!(matches!(
        Config::parse(&source),
        Err(ConfigError::Invalid(_))
    ));
    let source = entry("a", "A", "");
    assert!(matches!(
        Config::parse(&source),
        Err(ConfigError::Invalid(_))
    ));
}

#[test]
fn rejects_extreme_debounce() {
    assert!(Config::parse("autosave_debounce_ms = 0").is_err());
    assert!(Config::parse("autosave_debounce_ms = 600000").is_err());
}

#[test]
fn rejects_relative_data_dir() {
    assert!(Config::parse("data_dir = \"memo\"").is_err());
    assert!(Config::parse("data_dir = \"~/memo\"").is_err());
}

#[test]
fn rejects_invalid_toml() {
    assert!(matches!(
        Config::parse("font_size = = 1"),
        Err(ConfigError::Parse(_))
    ));
}

#[test]
fn rejects_unknown_field() {
    assert!(matches!(
        Config::parse("fontsize = 15"),
        Err(ConfigError::Parse(_))
    ));
}

#[test]
fn load_creates_initial_config_when_missing() {
    let dir = temp_dir("create");
    let path = dir.join("nested").join("config.toml");

    let (config, error) = config::load(&path);
    assert!(error.is_none());
    assert_eq!(config, Config::default());
    assert_eq!(fs::read_to_string(&path).unwrap(), DEFAULT_CONFIG_TOML);
}

#[test]
fn load_invalid_config_falls_back_without_touching_file() {
    let dir = temp_dir("invalid");
    let path = dir.join("config.toml");
    let broken = "font_size = \"big\"\n";
    fs::write(&path, broken).unwrap();

    let (config, error) = config::load(&path);
    assert!(error.is_some());
    assert_eq!(config, Config::default());
    assert_eq!(fs::read_to_string(&path).unwrap(), broken);
}

#[test]
fn load_invalid_config_keeps_valid_data_dir() {
    let dir = temp_dir("keep-data-dir");
    let path = dir.join("config.toml");
    let data_dir = dir.join("memo");
    fs::write(
        &path,
        format!(
            "data_dir = {:?}\nfontsize = 15\n",
            data_dir.to_str().unwrap()
        ),
    )
    .unwrap();

    let (config, error) = config::load(&path);
    assert!(error.is_some());
    assert_eq!(config.data_dir, data_dir.to_str().unwrap());
    assert_eq!(config.entry_types, Config::default().entry_types);
}
