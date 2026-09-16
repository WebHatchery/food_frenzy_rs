//! Transient floating gain numbers ("+12 renown", "+$9") spawned by gameplay
//! and drawn by the UI. Presentation-only state: never saved, cleared on load.

use macroquad::prelude::{vec2, Color};
use macroquad_toolkit::fx::{FloatingText, FloatingTextLayer};
use serde::{Deserialize, Serialize};

pub const FLOATER_LIFETIME_MS: f32 = 1_900.0;
/// Enough for a busy full room without turning into confetti.
pub const MAX_ACTIVE_FLOATERS: usize = 24;
const FLOATER_RISE_PX: f32 = 46.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloaterKind {
    Cash,
    Renown,
    Meat,
    Alert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FloaterAnchor {
    /// Dining-floor world coordinates (same space as `Customer::floor_x/y`).
    Floor { x: f32, y: f32 },
    /// Global gains with no floor position (crafting, upgrades, prestige);
    /// drawn beneath the resource header.
    Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floater {
    pub text: String,
    pub kind: FloaterKind,
    pub anchor: FloaterAnchor,
    pub age_ms: f32,
}

impl Floater {
    /// 0.0 (just spawned) → 1.0 (about to expire).
    pub fn progress(&self) -> f32 {
        (self.age_ms / FLOATER_LIFETIME_MS).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floaters {
    pub active: Vec<Floater>,
    #[serde(skip)]
    runtime: FloatingTextLayer,
}

impl Default for Floaters {
    fn default() -> Self {
        let mut runtime = FloatingTextLayer::new();
        runtime.max_active = MAX_ACTIVE_FLOATERS;
        Self {
            active: Vec::new(),
            runtime,
        }
    }
}

impl Floaters {
    pub fn spawn(&mut self, text: impl Into<String>, kind: FloaterKind, anchor: FloaterAnchor) {
        let text = text.into();
        self.active.push(Floater {
            text: text.clone(),
            kind,
            anchor,
            age_ms: 0.0,
        });
        let lifetime_s = FLOATER_LIFETIME_MS / 1_000.0;
        self.runtime.push(FloatingText::new(
            text,
            vec2(0.0, 0.0),
            floater_color(kind),
            16.0,
            lifetime_s,
            FLOATER_RISE_PX / lifetime_s,
        ));
        self.trim_metadata_to_runtime();
    }

    pub fn spawn_at(&mut self, text: impl Into<String>, kind: FloaterKind, x: f32, y: f32) {
        self.spawn(text, kind, FloaterAnchor::Floor { x, y });
    }

    pub fn update(&mut self, dt_ms: f32) {
        for floater in &mut self.active {
            floater.age_ms += dt_ms;
        }
        self.runtime.update(dt_ms.max(0.0) / 1_000.0);
        self.trim_metadata_to_runtime();
    }

    pub fn runtime_texts(&self) -> &[FloatingText] {
        self.runtime.texts()
    }

    fn trim_metadata_to_runtime(&mut self) {
        let excess = self.active.len().saturating_sub(self.runtime.count());
        if excess > 0 {
            self.active.drain(0..excess);
        }
    }
}

fn floater_color(kind: FloaterKind) -> Color {
    match kind {
        FloaterKind::Cash => Color::new(0.52, 0.84, 0.46, 1.0),
        FloaterKind::Renown => Color::new(0.45, 0.66, 0.96, 1.0),
        FloaterKind::Meat => Color::new(0.93, 0.52, 0.60, 1.0),
        FloaterKind::Alert => Color::new(0.94, 0.42, 0.36, 1.0),
    }
}
