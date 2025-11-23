//! Configuration file loading and parsing.

use super::schema::MegamaidConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Loads configuration from a YAML file.
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<MegamaidConfig> {
    let path = path.as_ref();

    if !path.exists() {
        anyhow::bail!("Configuration file not found: {}", path.display());
    }

    let content = fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path.display()))?;

    parse_config(&content)
        .context(format!("Failed to parse config file: {}", path.display()))
}

/// Parses configuration from a YAML string.
pub fn parse_config(yaml: &str) -> Result<MegamaidConfig> {
    let config: MegamaidConfig = serde_yaml::from_str(yaml)
        .context("Invalid YAML syntax or structure")?;

    Ok(config)
}

/// Attempts to load config from default locations.
/// Returns None if no config file is found.
pub fn load_default_config() -> Result<Option<MegamaidConfig>> {
    let default_paths = vec![
        "megamaid.yaml",
        "megamaid.yml",
        ".megamaid.yaml",
        ".megamaid.yml",
    ];

    for path in default_paths {
        if Path::new(path).exists() {
            return load_config(path).map(Some);
        }
    }

    Ok(None)
}

/// Writes a config to a YAML file.
pub fn write_config<P: AsRef<Path>>(config: &MegamaidConfig, path: P) -> Result<()> {
    let path = path.as_ref();

    let yaml = serde_yaml::to_string(config)
        .context("Failed to serialize configuration")?;

    fs::write(path, yaml)
        .context(format!("Failed to write config to: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
scanner:
  max_depth: 5
  skip_hidden: true
detector:
  rules:
    size_threshold:
      threshold_mb: 200
"#;

        let config = parse_config(yaml).unwrap();
        assert_eq!(config.scanner.max_depth, Some(5));
        assert_eq!(config.detector.rules.size_threshold.threshold_mb, 200);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "scanner: [[[invalid";
        let result = parse_config(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_config() {
        let yaml = "";
        let config = parse_config(yaml).unwrap();
        // Should use all defaults
        assert_eq!(config.scanner.max_depth, None);
        assert!(config.scanner.skip_hidden);
    }

    #[test]
    fn test_load_config_file() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.yaml");

        let yaml = r#"
scanner:
  max_depth: 10
executor:
  parallel: true
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.scanner.max_depth, Some(10));
        assert!(config.executor.parallel);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_config("/nonexistent/config.yaml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_write_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("output.yaml");

        let mut config = MegamaidConfig::default();
        config.scanner.max_depth = Some(15);
        config.executor.parallel = true;

        write_config(&config, &config_path).unwrap();

        // Read it back
        let loaded = load_config(&config_path).unwrap();
        assert_eq!(loaded.scanner.max_depth, Some(15));
        assert!(loaded.executor.parallel);
    }

    #[test]
    fn test_load_default_config_not_found() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = load_default_config().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_default_config_found() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("megamaid.yaml");

        let yaml = r#"
scanner:
  max_depth: 20
"#;
        fs::write(&config_path, yaml).unwrap();

        std::env::set_current_dir(temp.path()).unwrap();

        let result = load_default_config().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().scanner.max_depth, Some(20));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let original = MegamaidConfig::default();
        let yaml = serde_yaml::to_string(&original).unwrap();
        let parsed = parse_config(&yaml).unwrap();
        assert_eq!(original, parsed);
    }
}
