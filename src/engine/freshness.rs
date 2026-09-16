//! Dish freshness on the pass: fresh dishes pay a bill bonus, stale ones pay
//! normally, spoiled ones are thrown out. This is the kitchen's clock — it
//! turns fire-and-forget cooking into a timing decision.

use crate::data::GameBalance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Spoiled,
}

pub fn classify_dish_age(age_ms: f32, balance: &GameBalance) -> Freshness {
    if age_ms >= balance.dish_spoil_ms.max(1.0) {
        Freshness::Spoiled
    } else if age_ms >= balance.dish_fresh_window_ms.max(1.0) {
        Freshness::Stale
    } else {
        Freshness::Fresh
    }
}

/// Bill multiplier for serving a dish in this state. Spoiled dishes never
/// reach a table (the pass discards them), so only Fresh pays extra.
pub fn freshness_bill_multiplier(freshness: Freshness, balance: &GameBalance) -> f64 {
    match freshness {
        Freshness::Fresh => balance.fresh_bill_bonus_multiplier.max(1.0),
        Freshness::Stale | Freshness::Spoiled => 1.0,
    }
}

/// Seconds until the dish stops being fresh (for the pass UI countdown).
pub fn seconds_until_stale(age_ms: f32, balance: &GameBalance) -> f32 {
    ((balance.dish_fresh_window_ms - age_ms) / 1000.0).max(0.0)
}
