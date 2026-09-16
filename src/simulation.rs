//! Deterministic world simulation: cooking, arrivals, guest lifecycle, events,
//! day transitions, and the Lounge cooldown.

mod events;
pub mod guests;
mod spawning;
mod traits;

use crate::data::GameData;
use crate::engine::{max_customer_count, restaurant_table_position};
use crate::gameplay::dish_display_name;
use crate::player::update_player_movement;
use crate::state::{GameState, GuestState, ProgressionState, Timers};

pub fn update_game_world(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
) {
    timers.elapsed_ms += f64::from(dt_ms);
    let now_ms = timers.elapsed_ms;
    game_state.floaters.update(dt_ms);
    update_cooking(dt_ms, data, game_state);
    spawning::update_spawn(
        data,
        game_state,
        progression_state,
        guest_state,
        timers,
        now_ms,
    );
    update_customer_movement(dt_ms, data, progression_state, game_state);
    update_player_movement(dt_ms, data, game_state);
    guests::update_course_pacing(dt_ms, data, game_state);
    guests::update_departures(dt_ms, data, game_state, progression_state, guest_state);
    guests::update_patience(dt_ms, data, game_state, progression_state, timers, now_ms);
    guests::update_satisfaction_decay(dt_ms, data, game_state, progression_state, timers);
    traits::update_traits(dt_ms, data, game_state, timers, progression_state);
    traits::update_trait_alerts(dt_ms, data, game_state, progression_state);
    events::update_events(dt_ms, data, game_state, progression_state);
    update_day_cycle(dt_ms, data, game_state, progression_state);
    update_lounge(dt_ms, game_state);
}

/// Tick the day clock; when a day ends the ledger pauses the world (the app
/// stops simulating until the player opens the next day).
fn update_day_cycle(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    if game_state
        .day_cycle
        .update(dt_ms, data.balance.day_length_ms)
    {
        progression.record_day_completed();
        game_state.queue_sfx(crate::state::SfxCue::DayEnd);
        game_state.add_message(data.text_format(
            "message_day_end",
            [("day", game_state.day_cycle.day.to_string())].as_slice(),
        ));
    }
}

fn update_customer_movement(
    dt_ms: f32,
    data: &GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
) {
    let max_tables = max_customer_count(data, progression);
    let travel = data.balance.customer_walk_speed * (dt_ms / 1000.0);
    let mut newly_seated = false;
    for customer in &mut game_state.customers {
        let (target_x, target_y) = restaurant_table_position(customer.table_index, max_tables);
        customer.target_x = target_x;
        customer.target_y = target_y;

        let dx = target_x - customer.floor_x;
        let dy = target_y - customer.floor_y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= travel || distance <= 1.0 {
            customer.floor_x = target_x;
            customer.floor_y = target_y;
            newly_seated |= !customer.is_seated;
            customer.is_seated = true;
        } else {
            let step = travel / distance;
            customer.floor_x += dx * step;
            customer.floor_y += dy * step;
            customer.is_seated = false;
        }
    }
    if newly_seated {
        game_state.tutorial_observe(crate::data::TutorialTrigger::GuestSeated, data);
    }
}

fn update_cooking(dt_ms: f32, data: &GameData, game_state: &mut GameState) {
    let mut messages = Vec::new();
    let mut spoiled = 0usize;
    let mut plated = false;
    for station in game_state.cooking_stations.values_mut() {
        // Plated dishes age on the pass; spoiled ones are thrown out.
        for dish in &mut station.dishes {
            dish.age_ms += dt_ms;
        }
        let before = station.dishes.len();
        station.dishes.retain(|dish| {
            crate::engine::classify_dish_age(dish.age_ms, &data.balance)
                != crate::engine::Freshness::Spoiled
        });
        spoiled += before - station.dishes.len();

        if !station.is_cooking {
            continue;
        }

        if station.remaining_ms <= dt_ms {
            station.is_cooking = false;
            station.remaining_ms = 0.0;
            if let Some(dish) = data.dish_type_by_color(&station.color) {
                let cooked_name = crate::engine::random_dish_name(dish);
                station
                    .dishes
                    .push(crate::state::PlatedDish::new(cooked_name.clone()));
                plated = true;
                messages.push(
                    data.text_format(
                        "message_dish_ready",
                        [
                            ("dish", dish_display_name(data, &station.color)),
                            ("cooked", cooked_name),
                        ]
                        .as_slice(),
                    ),
                );
            }
        } else {
            station.remaining_ms -= dt_ms;
        }
    }

    if plated {
        game_state.queue_sfx(crate::state::SfxCue::DishReady);
    }
    if spoiled > 0 {
        game_state.floaters.spawn(
            data.text_format(
                "message_spoiled_floater",
                [("count", spoiled.to_string())].as_slice(),
            ),
            crate::state::FloaterKind::Alert,
            crate::state::FloaterAnchor::Header,
        );
        game_state.add_message(data.text_format(
            "message_spoiled",
            [("count", spoiled.to_string())].as_slice(),
        ));
    }
    for message in messages {
        game_state.add_message(message);
    }
}

fn update_lounge(dt_ms: f32, game_state: &mut GameState) {
    if !game_state.special_table_busy {
        return;
    }
    if game_state.special_table_timer > 0.0 {
        game_state.special_table_timer -= dt_ms;
    }
    if game_state.special_table_timer <= 0.0 {
        game_state.special_table_busy = false;
        game_state.special_table_timer = 0.0;
    }
}
