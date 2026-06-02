use serde::{ Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigObject {
    #[serde(rename = "type")]
    pub object_type: String,

    #[serde(rename = "sub-type", skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub sub_type: String,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub config: HashMap<String, ConfigValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl ConfigObject {
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.config.get(key)
    }

    pub fn string(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            ConfigValue::String(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            ConfigValue::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn int(&self, key: &str) -> Option<i64> {
        match self.get(key)? {
            ConfigValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn float(&self, key: &str) -> Option<f64> {
        match self.get(key)? {
            ConfigValue::Float(value) => Some(*value),
            ConfigValue::Int(value) => Some(*value as f64),
            _ => None,
        }
    }
}