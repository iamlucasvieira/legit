use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde::Serialize;

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
    /// Create a new Settings instance
    pub fn new() -> Result<Settings, ConfigError> {
        let default_config = include_str!("config/default.ini");
        Config::builder()
            .add_source(File::from_str(default_config, config::FileFormat::Ini))
            .add_source(File::new(".git/config", config::FileFormat::Ini).required(false))
            .add_source(Environment::with_prefix("LEGIT"))
            .build()?
            .try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_new() {
        let settings = Settings::new().unwrap();
        assert_eq!(settings.core.repositoryformatversion, 0);
    }
}
