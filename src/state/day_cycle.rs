//! Soft day/shift framing: a real-time day clock, per-day stats for the
//! end-of-day ledger, and the pause-the-world summary gate between days.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DayStats {
    pub cash_earned: i64,
    pub renown_earned: i64,
    pub guests_served: u32,
    pub guests_lost: u32,
    pub meat_gained: i64,
    pub guests_processed: u32,
    pub fresh_dishes: u32,
    pub best_combo: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayCycle {
    pub day: u32,
    pub elapsed_ms: f32,
    /// True between the day ending and the player opening the next one; the
    /// world pauses and the ledger overlay is up.
    pub summary_pending: bool,
    /// Whether this day's dining event has already fired.
    pub event_fired: bool,
    /// Data-driven objective shown in the closing ledger for the next shift.
    #[serde(default)]
    pub goal_id: String,
    pub stats: DayStats,
}

impl Default for DayCycle {
    fn default() -> Self {
        Self {
            day: 1,
            elapsed_ms: 0.0,
            summary_pending: false,
            event_fired: false,
            goal_id: String::new(),
            stats: DayStats::default(),
        }
    }
}

impl DayCycle {
    /// Advance the clock; returns true on the tick the day ends.
    pub fn update(&mut self, dt_ms: f32, day_length_ms: f32) -> bool {
        if self.summary_pending {
            return false;
        }
        self.elapsed_ms += dt_ms.max(0.0);
        if self.elapsed_ms >= day_length_ms.max(1_000.0) {
            self.summary_pending = true;
            return true;
        }
        false
    }

    pub fn start_next_day(&mut self, goal_id: String) {
        self.day = self.day.saturating_add(1);
        self.elapsed_ms = 0.0;
        self.summary_pending = false;
        self.event_fired = false;
        self.goal_id = goal_id;
        self.stats = DayStats::default();
    }

    /// 0..1 progress through the current day (for the clock UI).
    pub fn day_progress(&self, day_length_ms: f32) -> f32 {
        (self.elapsed_ms / day_length_ms.max(1.0)).clamp(0.0, 1.0)
    }

    pub fn record_combo(&mut self, combo: u32) {
        self.stats.best_combo = self.stats.best_combo.max(combo);
    }
}
