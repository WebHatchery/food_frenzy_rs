//! Onboarding progress: which data-driven tutorial step (see
//! `assets/data/tutorial.json`) the player is on. Steps advance when gameplay
//! reports the matching trigger; the definitions live in `GameData`.

use crate::data::{TutorialStep, TutorialTrigger};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TutorialProgress {
    pub step_index: usize,
    pub complete: bool,
}

impl TutorialProgress {
    pub fn current_step<'data>(&self, steps: &'data [TutorialStep]) -> Option<&'data TutorialStep> {
        if self.complete {
            return None;
        }
        steps.get(self.step_index)
    }

    /// Called by gameplay whenever something tutorial-worthy happens; advances
    /// the tutorial if the event matches the current step's trigger.
    pub fn observe(&mut self, trigger: TutorialTrigger, steps: &[TutorialStep]) {
        if self
            .current_step(steps)
            .is_some_and(|step| step.trigger == trigger)
        {
            self.advance(steps);
        }
    }

    /// Advance unconditionally (the "Got it" button on acknowledged steps).
    pub fn advance(&mut self, steps: &[TutorialStep]) {
        self.step_index = self.step_index.saturating_add(1);
        if self.step_index >= steps.len() {
            self.complete = true;
        }
    }

    pub fn skip(&mut self) {
        self.complete = true;
    }
}
