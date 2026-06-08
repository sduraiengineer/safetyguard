use serde::{Serialize, Deserialize};
use crate::safeappdb::config_object::ConfigObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    name: String,
    #[serde(default = "default_max_retries")]
    max_retries: i64,
    #[serde(default)]
    critical: bool,
    manager: ConfigObject,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub safemonitor: Vec<ConfigObject>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub safeaction: Vec<ConfigObject>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub safeinstrument: Vec<ConfigObject>,
}

fn default_max_retries() -> i64 {
    -1
}
