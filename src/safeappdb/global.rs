use figment::{Figment, providers::Format};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_config_version")]
    config_version: u64,
    #[serde(default)]
    global: Global,
    #[serde(default)]
    watchdog: Watchdog,
    #[serde(default)]
    recover: Recover,
}
impl Config {
    fn default_config_version() -> u64 {
        1
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
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

impl Default for Global {
    fn default() -> Self {
        Self {
            log_level: LogLevel::INFO,
            check_interval_ms: 500,
            state_file: "/tmp/safeappdb.state".to_string(),
            log_file: "/tmp/safeappdb.log".to_string(),
            config_file: "/tmp/safeappdb.config".to_string(),
            config_file_gen: "/tmp/safeappdb.config.gen".to_string(),
            mode: Mode::DEVELOPMENT,
            endpoint: "/tmp/saftyguard.sock".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    DEDUG,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Watchdog {
    device: String,
    timeout_sec: u64,
    pretimeout_sec: u64,
    kick_interval_ms: u64,
}

impl Default for Watchdog {
    fn default() -> Self {
        Self {
            device: "/dev/watchdog".to_string(),
            timeout_sec: 10,
            pretimeout_sec: 5,
            kick_interval_ms: 3000,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Recover {
    system_reset_on_failure: bool,
    max_global_failures: u64,
}
impl Default for Recover {
    fn default() -> Self {
        Self {
            system_reset_on_failure: true,
            max_global_failures: 5,
        }
    }
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

watchdog:
  device: /dev/watchdog2
  timeout_sec: 100
  pretimeout_sec: 15
  kick_interval_ms: 2000

recover:
  system_reset_on_failure: false
  max_global_failures: 6

"#;

        let mut fp = fs::File::create("/tmp/tst.yaml")?;
        fp.write_all(yml.as_bytes())?;
        fp.sync_all()?;

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

        assert_eq!(config.recover.system_reset_on_failure, false);
        assert_eq!(config.recover.max_global_failures, 6);

        assert_eq!(config.watchdog.device, "/dev/watchdog2");
        assert_eq!(config.watchdog.timeout_sec, 100);
        assert_eq!(config.watchdog.pretimeout_sec, 15);
        assert_eq!(config.watchdog.kick_interval_ms, 2000);

        fs::remove_file("/tmp/tst.yaml")?;

        Ok(())
    }
    #[test]
    fn test_config_default() -> Result<(), Box<dyn std::error::Error>> {
        let yml = r#"
"#;

        let mut fp = fs::File::create("/tmp/tst2.yaml")?;
        fp.write_all(yml.as_bytes())?;
        fp.sync_all()?;

        let config = Config::get_config("/tmp/tst2.yaml".to_string())?;
        println!("{config:#?}");

        assert_eq!(config.global.check_interval_ms, 500);
        assert_eq!(config.global.state_file, "/tmp/safeappdb.state");
        assert_eq!(config.global.log_file, "/tmp/safeappdb.log");
        assert_eq!(config.global.config_file, "/tmp/safeappdb.config");
        assert_eq!(config.global.config_file_gen, "/tmp/safeappdb.config.gen");
        assert_eq!(config.global.endpoint, "/tmp/saftyguard.sock");
        assert_eq!(config.global.log_level, LogLevel::INFO);
        assert_eq!(config.global.mode, Mode::DEVELOPMENT);

        assert_eq!(config.recover.system_reset_on_failure, true);
        assert_eq!(config.recover.max_global_failures, 5);
        
        assert_eq!(config.watchdog.device, "/dev/watchdog");
        assert_eq!(config.watchdog.timeout_sec, 10);
        assert_eq!(config.watchdog.pretimeout_sec, 5);
        assert_eq!(config.watchdog.kick_interval_ms, 3000);

        fs::remove_file("/tmp/tst2.yaml")?;

        Ok(())
    }
}