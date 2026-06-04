use std::{fs, path::Path};

use super::{GuardConfigFile, DEFAULT_CONFIG_FILENAME};

pub(super) const CONFIG_FILENAMES: &[&str] = &[
    "flavor.toml",
    "flavor.yaml",
    "flavor.yml",
    DEFAULT_CONFIG_FILENAME,
];

pub(super) fn parse_config_file(config_path: &Path) -> Result<GuardConfigFile, String> {
    let source = fs::read_to_string(config_path)
        .map_err(|error| format!("failed to read config {}: {error}", config_path.display()))?;
    let extension = config_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    match extension {
        "json" => serde_json::from_str(&source).map_err(|error| {
            format!(
                "failed to parse JSON config {}: {error}",
                config_path.display()
            )
        }),
        "toml" => toml::from_str(&source).map_err(|error| {
            format!(
                "failed to parse TOML config {}: {error}",
                config_path.display()
            )
        }),
        "yaml" | "yml" => serde_yaml::from_str(&source).map_err(|error| {
            format!(
                "failed to parse YAML config {}: {error}",
                config_path.display()
            )
        }),
        _ => Err(format!(
            "unsupported config format for {}: expected .json, .toml, .yaml, or .yml",
            config_path.display()
        )),
    }
}
