use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Core {
    pub repositoryformatversion: i32,
    pub filemode: bool,
    pub bare: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub core: Core,
}

impl Settings {
    // Create a new settings, receives a file path
    pub fn new(use_config_path: Option<&Path>) -> Result<Settings, ConfigError> {
        let mut builder = Config::builder()
            .add_source(File::with_name("src/config/default"))
            .add_source(Environment::with_prefix("LEGIT"));

        if let Some(path) = use_config_path {
            builder = builder.add_source(File::from(path));
        }

        builder.build()?.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_settings_new() {
        let settings = Settings::new(None).unwrap();
        assert_eq!(settings.core.repositoryformatversion, 0);
    }

    #[test]
    fn test_settings_new_user_file() {
        let mut file = NamedTempFile::with_suffix(".ini").expect("Failed to create temp file");
        let content = r#"
            [core]
            repositoryformatversion = 100
        "#;
        write!(file, "{}", content).unwrap();
        assert_eq!(
            Settings::new(Some(file.path()))
                .unwrap()
                .core
                .repositoryformatversion,
            100
        );
    }
}
