//! Shared balance subtypes that keep the main catalog module cohesive.

use serde::{Deserialize, Serialize};

/// Configurable timing for the Last Meal Lounge presentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CinematicTiming {
    #[serde(default)]
    pub escort_ms: f32,
    #[serde(default)]
    pub curtain_ms: f32,
    #[serde(default)]
    pub quiet_ms: f32,
    #[serde(default)]
    pub reveal_ms: f32,
}

impl Default for CinematicTiming {
    fn default() -> Self {
        Self {
            escort_ms: 1_400.0,
            curtain_ms: 900.0,
            quiet_ms: 800.0,
            reveal_ms: 2_600.0,
        }
    }
}
