use figment::{Figment, providers::Format};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    config_version: u64,
    global: Global,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Global {
    log_level: LogLevel,
    check_interval_ms: u64,
    state_file: String,
    log_file: String,
    config_file: String,
    config_file_gen: String,
    mode: Mode,
    endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    DEDUG,
    #[default]
    INFO,
    WARN,
    ERROR,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    DEVELOPMENT,
    PRODUCTION,
}

impl Config {
    pub fn get_config(fp : String ) -> Result<Config, Box<dyn std::error::Error>> {
        let cfg: Config = Figment::from(serde_saphyr::figment::Yaml::file(fp))
            .extract::<Config>()?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use super::*;


    #[test]
    fn test_config() -> Result<(), Box<dyn std::error::Error>> {
        let yml = r#"
config_version: 1
global:
    log_level: info
    check_interval_ms: 1000
    state_file: /tmp/safeappdb.state
    log_file: /tmp/safeappdb.log
    config_file: /tmp/safeappdb.config
    config_file_gen: /tmp/safeappdb.config.gen
    mode: development # Can be development.
    endpoint: /tmp/saftyguard.sock
"#;

        let mut fp = fs::File::create("/tmp/tst.yaml")?;
        fp.write_all(yml.as_bytes())?;
        fp.sync_data()?;

        let config = Config::get_config("/tmp/tst.yaml".to_string())?;
        println!("{config:#?}");

        assert_eq!(config.global.check_interval_ms, 1000);
        assert_eq!(config.global.state_file, "/tmp/safeappdb.state");
        assert_eq!(config.global.log_file, "/tmp/safeappdb.log");
        assert_eq!(config.global.config_file, "/tmp/safeappdb.config");
        assert_eq!(config.global.config_file_gen, "/tmp/safeappdb.config.gen");
        assert_eq!(config.global.endpoint, "/tmp/saftyguard.sock");
        assert_eq!(config.global.log_level, LogLevel::INFO);
        assert_eq!(config.global.mode, Mode::DEVELOPMENT);

        fs::remove_file("/tmp/tst.yaml")?;

        Ok(())
    }
}