//! Dining events: one weighted-random situation fires partway through each
//! day (rush, inspector, critic, generous mood), lasts a fixed duration, and
//! changes how the floor plays while it runs.

use crate::data::GameData;
use crate::state::{ActiveEvent, FloaterAnchor, FloaterKind, GameState, ProgressionState};

/// Return the events that can appear on a given service day in catalog order.
/// Keeping eligibility pure makes the content gate easy to test without a
/// Macroquad clock or global random source.
pub fn eligible_event_ids(data: &GameData, day: u32) -> Vec<String> {
    data.dining_events
        .iter()
        .filter(|event| event.min_day <= day)
        .map(|event| event.id.clone())
        .collect()
}

/// Resolve a weighted event roll without touching global randomness.
pub fn select_event_id(data: &GameData, day: u32, roll: u32) -> Option<String> {
    let candidates: Vec<_> = data
        .dining_events
        .iter()
        .filter(|event| event.min_day <= day)
        .collect();
    let total_weight: u32 = candidates.iter().map(|event| event.weight).sum();
    if total_weight == 0 || roll >= total_weight {
        return None;
    }
    let mut remaining = roll;
    candidates.into_iter().find_map(|event| {
        if remaining < event.weight {
            Some(event.id.clone())
        } else {
            remaining -= event.weight;
            None
        }
    })
}

pub fn update_events(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    if let Some(active) = &mut game_state.active_event {
        active.remaining_ms -= dt_ms;
        if active.remaining_ms <= 0.0 {
            let name = data
                .dining_event_by_id(&active.event_id)
                .map(|event| event.name.clone())
                .unwrap_or_else(|| data.text("message_event_unknown").to_string());
            game_state.active_event = None;
            progression.record_event_completed();
            game_state.add_message(
                data.text_format("message_event_passed", [("event", name)].as_slice()),
            );
        }
        return;
    }

    // One event per day, firing once the day is far enough along.
    let day = &game_state.day_cycle;
    if day.event_fired
        || day.summary_pending
        || day.elapsed_ms < data.balance.day_length_ms * data.balance.event_day_fraction
    {
        return;
    }

    let current_day = game_state.day_cycle.day;
    let total_weight: u32 = data
        .dining_events
        .iter()
        .filter(|event| event.min_day <= current_day)
        .map(|event| event.weight)
        .sum();
    game_state.day_cycle.event_fired = true;
    if total_weight == 0 {
        return;
    }

    let roll = macroquad_toolkit::rng::gen_range(0i32, total_weight as i32) as u32;
    let Some(event_id) = select_event_id(data, current_day, roll) else {
        return;
    };
    let Some(event) = data.dining_event_by_id(&event_id) else {
        return;
    };
    game_state.active_event = Some(ActiveEvent {
        event_id: event.id.clone(),
        remaining_ms: event.duration_ms.max(1_000.0),
    });
    game_state.add_message(event.announcement.clone());
    game_state.queue_sfx(crate::state::SfxCue::Event);
    game_state.floaters.spawn(
        event.name.clone(),
        FloaterKind::Alert,
        FloaterAnchor::Header,
    );
}
