//! Player-facing copy loaded from the same typed asset catalog as game data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextCatalog {
    pub entries: HashMap<String, String>,
    #[serde(default)]
    pub fallback_chatter: Vec<String>,
}

impl TextCatalog {
    pub fn get(&self, key: &str) -> &str {
        self.entries.get(key).map(String::as_str).unwrap_or("")
    }

    pub fn format(&self, key: &str, replacements: &[(&str, String)]) -> String {
        let mut value = self.get(key).to_string();
        for (name, replacement) in replacements {
            value = value.replace(&format!("{{{name}}}"), replacement);
        }
        value
    }

    pub fn fallback_chatter_line(&self, index: usize) -> Option<&str> {
        self.fallback_chatter
            .get(index % self.fallback_chatter.len().max(1))
            .map(String::as_str)
    }
}
