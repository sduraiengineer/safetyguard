use serde::{Serialize, Deserialize};
use crate::safeappdb::config_object::ConfigObject;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub enum AppState {
    #[default]
    NotInitialized,
    Initialized,
    Running,
    Degraded,
    Error,
    Stopped,
    Failed,
}

#[derive(Debug, Default)]
pub struct AppRuntimeContext {
    app_state: AppState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Apps {
    pub name: String,
    #[serde(default = "default_max_retries")]
    pub max_retries: i64,
    #[serde(default)]
    pub critical: bool,
    pub manager: ConfigObject,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub safemonitor: Vec<ConfigObject>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub safeaction: Vec<ConfigObject>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub safeinstrument: Vec<ConfigObject>,
    #[serde(skip_serializing, skip_deserializing)]
    pub runtime_context: AppRuntimeContext,
}

fn default_max_retries() -> i64 {
    -1
}
