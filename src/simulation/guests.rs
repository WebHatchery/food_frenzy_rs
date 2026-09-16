//! Seated-guest lifecycle: patience walkouts, satisfied departures (paying the
//! tab and banking a fattening visit), and satisfaction decay over time.

use crate::data::{GameData, TutorialTrigger};
use crate::engine::{satisfied_tip, visits_until_ready_for};
use crate::state::{FloaterKind, GameState, GuestState, ProgressionState, Timers};

/// Impatience is checked once per second of accumulated playtime.
const PATIENCE_CHECK_INTERVAL_MS: f32 = 1_000.0;

pub(super) fn update_patience(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    timers: &mut Timers,
    now_ms: f64,
) {
    if game_state.customers.is_empty() {
        timers.patience_timer.reset();
        return;
    }

    timers
        .patience_timer
        .set_interval(PATIENCE_CHECK_INTERVAL_MS);
    for _ in 0..timers.patience_timer.tick(dt_ms) {
        let removed_ids = impatient_customer_ids(data, game_state, progression, now_ms);
        if removed_ids.is_empty() {
            continue;
        }

        for id in &removed_ids {
            let Some(customer) = game_state.customers.iter().find(|item| item.id == *id) else {
                continue;
            };
            let paid = customer.bill.max(0);
            let name = customer.display_name.clone();
            let served = customer.courses_served();
            let ordered = customer.order.len();
            let (floor_x, floor_y) = (customer.floor_x, customer.floor_y);
            progression.add_currency(paid);
            progression.record_customer_lost();
            game_state
                .floaters
                .spawn_at("walked out!", FloaterKind::Alert, floor_x, floor_y);
            if paid > 0 {
                game_state.add_message(format!(
                    "{name} left unhappy ({served}/{ordered} courses), only paid ${paid}."
                ));
            } else {
                game_state.add_message(format!("{name} left hungry and unhappy. A shame."));
            }
        }

        game_state.day_cycle.stats.guests_lost = game_state
            .day_cycle
            .stats
            .guests_lost
            .saturating_add(removed_ids.len() as u32);
        game_state
            .customers
            .retain(|customer| !removed_ids.contains(&customer.id));
        game_state.combo = 0;
    }
}

/// Guests who have eaten every course of their order finish up, settle their
/// tab (plus a tip), and leave — freeing the table and banking a satisfied
/// visit toward the day they are plump enough for the Last Meal Lounge.
pub fn update_departures(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    let dwell = data.balance.content_dwell_ms.max(0.0);
    let mut departed = Vec::new();
    for customer in &mut game_state.customers {
        if !(customer.is_seated && customer.order_complete()) {
            customer.depart_timer_ms = 0.0;
            continue;
        }

        if customer.depart_timer_ms <= 0.0 {
            customer.depart_timer_ms = dwell.max(1.0);
        }
        customer.depart_timer_ms -= dt_ms;
        if customer.depart_timer_ms <= 0.0 {
            departed.push(customer.id);
        }
    }

    if departed.is_empty() {
        return;
    }

    let mut messages = Vec::new();
    let mut floaters = Vec::new();
    let mut any_ready = false;
    let mut cash_earned = 0i64;
    let mut served_count = 0u32;
    let event_tip_multiplier = match game_state.active_event_effect(data) {
        Some(crate::data::EventEffect::TipMultiplier { multiplier }) => *multiplier,
        _ => 1.0,
    };
    for id in &departed {
        let Some(customer) = game_state.customers.iter().find(|item| item.id == *id) else {
            continue;
        };
        let bill = customer.bill.max(0);
        let mut tip = satisfied_tip(data, bill);
        let mut tip_notes = String::new();
        if customer.traits(data).big_tipper {
            tip *= 2;
            tip_notes.push_str(" A big tipper!");
        }
        if event_tip_multiplier > 1.0 {
            tip = ((tip as f64) * event_tip_multiplier).round() as i64;
        }
        let paid = bill.saturating_add(tip);
        progression.add_currency(paid);
        cash_earned += paid;
        served_count += 1;
        guest_state.record_guest_satisfied_visit(&customer.guest_id);
        let fed = customer.times_fed.saturating_add(1);
        let ready = fed >= visits_until_ready_for(data, &customer.customer_type);
        any_ready |= ready;
        let ready_hint = if ready {
            " Plump enough for the Lounge next visit."
        } else {
            ""
        };
        floaters.push((format!("+${paid}"), customer.floor_x, customer.floor_y));
        messages.push(format!(
            "{} left glowing and settled ${paid} (${bill} +${tip} tip).{tip_notes}{ready_hint}",
            customer.display_name
        ));
    }

    game_state
        .customers
        .retain(|customer| !departed.contains(&customer.id));
    game_state.combo = game_state.combo.saturating_add(1);
    game_state.day_cycle.stats.cash_earned += cash_earned;
    game_state.day_cycle.stats.guests_served += served_count;
    if served_count > 0 {
        game_state.queue_sfx(crate::state::SfxCue::Cash);
    }
    for (text, x, y) in floaters {
        game_state.floaters.spawn_at(text, FloaterKind::Cash, x, y);
    }
    for message in messages {
        game_state.add_message(message);
    }
    game_state.tutorial_observe(TutorialTrigger::GuestDepartedFed, data);
    if any_ready {
        game_state.tutorial_observe(TutorialTrigger::GuestReady, data);
    }
}

/// Advance each seated guest's meal rhythm: count down the eating window,
/// then count up the wait for the next course. Guests kept waiting past the
/// grace window bleed satisfaction until the course lands.
pub(super) fn update_course_pacing(dt_ms: f32, data: &GameData, game_state: &mut GameState) {
    let hangry_decay = data.balance.hangry_satisfaction_decay_per_s.max(0.0) * dt_ms / 1_000.0;
    for customer in &mut game_state.customers {
        if !customer.is_seated || customer.order_complete() || customer.courses_served() == 0 {
            continue;
        }
        if customer.eating_ms > 0.0 {
            customer.eating_ms = (customer.eating_ms - dt_ms).max(0.0);
            customer.waiting_ms = 0.0;
            continue;
        }
        customer.waiting_ms += dt_ms;
        if hangry_decay > 0.0 && crate::engine::is_kept_waiting(customer, &data.balance) {
            customer.satisfaction.decay_all(hangry_decay);
            customer.refresh_totals();
        }
    }
}

pub(super) fn update_satisfaction_decay(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    timers: &mut Timers,
) {
    let decay_interval = data.balance.satisfaction_decay_interval;
    if decay_interval <= 0.0 {
        return;
    }
    if game_state.customers.is_empty() {
        timers.decay_timer.reset();
        return;
    }

    let decay_rate = crate::engine::satisfaction_decay_rate(data, progression);
    timers.decay_timer.set_interval(decay_interval);
    for _ in 0..timers.decay_timer.tick(dt_ms) {
        for customer in &mut game_state.customers {
            customer.satisfaction.decay_all(decay_rate);
            customer.refresh_totals();
        }
    }
}

fn impatient_customer_ids(
    data: &GameData,
    game_state: &GameState,
    progression: &ProgressionState,
    now_ms: f64,
) -> Vec<u32> {
    let patience_multiplier = crate::engine::patience_multiplier(progression);
    game_state
        .customers
        .iter()
        .filter_map(|customer| {
            let traits = customer.traits(data);
            let mut patience = data.balance.customer_patience_time * patience_multiplier;
            if traits.fast_spoilage {
                patience *= 0.55;
            }
            (now_ms - customer.arrived_at_ms > f64::from(patience)).then_some(customer.id)
        })
        .collect()
}
