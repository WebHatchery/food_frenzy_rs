//! The Last Meal Lounge processing sequence: a short, fixed timeline that
//! turns the game's signature moment into a staged beat instead of a ticker
//! line. Pure timing/state — drawing lives in `ui::lounge`. Rewards are
//! applied when the invite succeeds; this struct only carries what to show.

use crate::data::CinematicTiming;
use macroquad_toolkit::timing::Timeline;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CinematicPhase {
    /// The guest is led from their table toward the lounge; the room dims.
    Escort,
    /// The lounge curtain draws shut.
    Curtain,
    /// A held beat behind the curtain. The wrong-note moment.
    Quiet,
    /// The payoff: meat gained, renown, the guest's parting.
    Reveal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingCinematic {
    pub guest_name: String,
    pub customer_type: String,
    pub meat_gain: i64,
    pub meat_type: String,
    pub renown_gain: i64,
    pub cash_gain: i64,
    /// Where the guest was seated, in dining-floor world coordinates.
    pub from_floor: (f32, f32),
    /// Personality farewell line shown in the reveal, if the guest had one.
    pub farewell: Option<String>,
    #[serde(default)]
    pub timing: CinematicTiming,
    pub elapsed_ms: f32,
}

impl ProcessingCinematic {
    pub fn new(
        guest_name: String,
        customer_type: String,
        meat_gain: i64,
        meat_type: String,
        renown_gain: i64,
        cash_gain: i64,
        from_floor: (f32, f32),
    ) -> Self {
        Self {
            guest_name,
            customer_type,
            meat_gain,
            meat_type,
            renown_gain,
            cash_gain,
            from_floor,
            farewell: None,
            timing: CinematicTiming::default(),
            elapsed_ms: 0.0,
        }
    }

    pub fn with_timing(mut self, timing: CinematicTiming) -> Self {
        self.timing = timing;
        self
    }

    /// The fixed phase table, in order. Shared by [`total_ms`](Self::total_ms)
    /// and [`phase`](Self::phase) via a fresh [`Timeline`] built from it.
    fn phase_table(&self) -> Vec<(CinematicPhase, f32)> {
        vec![
            (CinematicPhase::Escort, self.timing.escort_ms.max(0.0)),
            (CinematicPhase::Curtain, self.timing.curtain_ms.max(0.0)),
            (CinematicPhase::Quiet, self.timing.quiet_ms.max(0.0)),
            (CinematicPhase::Reveal, self.timing.reveal_ms.max(0.0)),
        ]
    }

    /// A [`Timeline`] advanced to this cinematic's current `elapsed_ms`.
    /// Rebuilt on demand rather than stored, since `elapsed_ms` is the only
    /// field persisted to save data.
    fn timeline_at(&self) -> Timeline<CinematicPhase> {
        let mut timeline = Timeline::new(self.phase_table());
        timeline.advance(self.elapsed_ms);
        timeline
    }

    pub fn total_ms(&self) -> f32 {
        Timeline::new(self.phase_table()).total_duration()
    }

    pub fn advance(&mut self, dt_ms: f32) {
        self.elapsed_ms += dt_ms.max(0.0);
    }

    /// Current phase and 0..1 progress within it.
    pub fn phase(&self) -> (CinematicPhase, f32) {
        match self.timeline_at().current() {
            Some((phase, progress)) => (*phase, progress),
            None => (CinematicPhase::Reveal, 1.0),
        }
    }

    pub fn finished(&self) -> bool {
        self.timeline_at().finished()
    }

    /// The payoff is on screen, so a click may dismiss the sequence early.
    pub fn can_dismiss(&self) -> bool {
        matches!(self.phase().0, CinematicPhase::Reveal)
    }
}
