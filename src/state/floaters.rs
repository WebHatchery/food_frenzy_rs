//! Transient floating gain numbers ("+12 renown", "+$9") spawned by gameplay
//! and drawn by the UI. Presentation-only state: never saved, cleared on load.

use serde::{Deserialize, Serialize};

pub const FLOATER_LIFETIME_MS: f32 = 1_900.0;
/// Enough for a busy full room without turning into confetti.
pub const MAX_ACTIVE_FLOATERS: usize = 24;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Floaters {
    pub active: Vec<Floater>,
}

impl Floaters {
    pub fn spawn(&mut self, text: impl Into<String>, kind: FloaterKind, anchor: FloaterAnchor) {
        self.active.push(Floater {
            text: text.into(),
            kind,
            anchor,
            age_ms: 0.0,
        });
        if self.active.len() > MAX_ACTIVE_FLOATERS {
            self.active
                .drain(0..(self.active.len() - MAX_ACTIVE_FLOATERS));
        }
    }

    pub fn spawn_at(&mut self, text: impl Into<String>, kind: FloaterKind, x: f32, y: f32) {
        self.spawn(text, kind, FloaterAnchor::Floor { x, y });
    }

    pub fn update(&mut self, dt_ms: f32) {
        for floater in &mut self.active {
            floater.age_ms += dt_ms;
        }
        self.active
            .retain(|floater| floater.age_ms < FLOATER_LIFETIME_MS);
    }
}
