//! Special-trait situations with counterplay. Telegraphed traits (fox steal,
//! monkey tantrum, wandering) arm a visible warning and only fire if the
//! player doesn't answer within the window; passive traits (fast spoilage)
//! tick as before. First encounters surface a one-time hint from
//! `assets/data/trait_behaviors.json`.

use crate::data::GameData;
use crate::engine::{chance, max_customer_count};
use crate::gameplay::dish_display_name;
use crate::state::{FloaterKind, GameState, ProgressionState, Timers, TraitAlert};
use std::collections::HashSet;

pub(super) fn update_traits(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    timers: &mut Timers,
    progression: &mut ProgressionState,
) {
    let tick = data.balance.trait_tick_interval;
    if tick <= 0.0 || game_state.customers.is_empty() {
        timers.trait_timer.reset();
        return;
    }

    timers.trait_timer.set_interval(tick);
    for _ in 0..timers.trait_timer.tick(dt_ms) {
        for index in 0..game_state.customers.len() {
            tick_customer_traits(index, data, game_state, progression);
        }
    }
}

/// Roll each guest's traits once per tick: passive effects apply immediately;
/// telegraphed ones arm a warning instead of firing.
fn tick_customer_traits(
    index: usize,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let traits = game_state.customers[index].traits(data);

    if traits.fast_spoilage {
        game_state.customers[index]
            .satisfaction
            .decay_all(data.balance.fast_spoilage_decay);
        game_state.customers[index].refresh_totals();
        surface_first_encounter_hint("fast_spoilage", index, data, game_state, progression);
    }

    if !game_state.customers[index].is_seated || game_state.customers[index].trait_alert.is_some() {
        return;
    }

    let armed_key = if traits.can_steal_food
        && chance(data.balance.fox_steal_chance)
        && any_plated_dish(game_state)
    {
        Some("can_steal_food")
    } else if traits.throws_food
        && chance(data.balance.monkey_throw_chance)
        && game_state.customers[index].total_satisfaction < data.balance.monkey_cranky_threshold
        && game_state.customers[index].total_satisfaction > 0.0
    {
        Some("throws_food")
    } else if traits.can_wander && chance(data.balance.can_wander_chance) {
        Some("can_wander")
    } else {
        None
    };

    let Some(trait_key) = armed_key else {
        return;
    };
    let courses_served = game_state.customers[index].courses_served();
    game_state.customers[index].trait_alert = Some(TraitAlert {
        trait_key: trait_key.to_string(),
        remaining_ms: data.balance.trait_telegraph_ms.max(500.0),
        courses_served_at_arm: courses_served,
    });
    if let Some(behavior) = data.trait_behavior(trait_key) {
        let name = game_state.customers[index].display_name.clone();
        game_state.add_message(data.text_format(
            "message_trait_telegraph",
            [("name", name), ("telegraph", behavior.telegraph.clone())].as_slice(),
        ));
    }
    surface_first_encounter_hint(trait_key, index, data, game_state, progression);
}

fn surface_first_encounter_hint(
    trait_key: &str,
    index: usize,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    if !progression.note_trait_encounter(trait_key) {
        return;
    }
    let Some(behavior) = data.trait_behavior(trait_key) else {
        return;
    };
    let (x, y) = {
        let customer = &game_state.customers[index];
        (customer.floor_x, customer.floor_y)
    };
    game_state.floaters.spawn_at(
        data.text_format(
            "message_trait_new",
            [("trait", behavior.name.clone())].as_slice(),
        ),
        FloaterKind::Alert,
        x,
        y,
    );
    game_state.add_message(
        data.text_format(
            "message_trait_hint",
            [
                ("trait", behavior.name.clone()),
                ("hint", behavior.hint.clone()),
            ]
            .as_slice(),
        ),
    );
}

/// Count down armed warnings every frame and resolve the ones whose window
/// closed — either the consequence fires or the counterplay averted it.
pub(super) fn update_trait_alerts(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
) {
    let mut expired = Vec::new();
    for (index, customer) in game_state.customers.iter_mut().enumerate() {
        if let Some(alert) = &mut customer.trait_alert {
            alert.remaining_ms -= dt_ms;
            if alert.remaining_ms <= 0.0 {
                expired.push(index);
            }
        }
    }

    let max_tables = max_customer_count(data, progression);
    for index in expired {
        let Some(alert) = game_state.customers[index].trait_alert.take() else {
            continue;
        };
        resolve_alert(index, &alert, data, game_state, max_tables);
    }
}

fn resolve_alert(
    index: usize,
    alert: &TraitAlert,
    data: &GameData,
    game_state: &mut GameState,
    max_tables: usize,
) {
    let name = game_state.customers[index].display_name.clone();
    let (x, y) = {
        let customer = &game_state.customers[index];
        (customer.floor_x, customer.floor_y)
    };
    match alert.trait_key.as_str() {
        "can_steal_food" => {
            // Counterplay: their order got finished, or the pass was cleared.
            if game_state.customers[index].order_complete() || !any_plated_dish(game_state) {
                game_state.add_message(
                    data.text_format("message_trait_nothing", [("name", name.clone())].as_slice()),
                );
                return;
            }
            if let Some((station_color, dish_name)) = steal_dish(game_state) {
                let station_name = dish_display_name(data, &station_color);
                game_state
                    .floaters
                    .spawn_at(data.text("ui_stole_dish"), FloaterKind::Alert, x, y);
                game_state.add_message(
                    data.text_format(
                        "message_trait_stole",
                        [
                            ("name", name),
                            ("dish", dish_name),
                            ("station", station_name),
                        ]
                        .as_slice(),
                    ),
                );
            }
        }
        "throws_food" => {
            // Counterplay: satisfaction raised above the cranky threshold.
            if game_state.customers[index].total_satisfaction
                >= data.balance.monkey_cranky_threshold
            {
                game_state.add_message(
                    data.text_format("message_trait_settled", [("name", name.clone())].as_slice()),
                );
                return;
            }
            if let Some((station_color, dish_name)) = steal_dish(game_state) {
                let station_name = dish_display_name(data, &station_color);
                game_state.combo = 0;
                game_state
                    .floaters
                    .spawn_at("tantrum! combo lost", FloaterKind::Alert, x, y);
                game_state.add_message(
                    data.text_format(
                        "message_trait_threw",
                        [
                            ("name", name),
                            ("dish", dish_name),
                            ("station", station_name),
                        ]
                        .as_slice(),
                    ),
                );
            }
        }
        "can_wander" => {
            // Counterplay: any course served during the window settles them.
            if game_state.customers[index].courses_served() > alert.courses_served_at_arm {
                game_state.add_message(
                    data.text_format("message_trait_course_settled", [("name", name)].as_slice()),
                );
                return;
            }
            move_customer_to_empty_table(index, data, game_state, max_tables);
        }
        _ => {}
    }
}

fn any_plated_dish(game_state: &GameState) -> bool {
    game_state
        .cooking_stations
        .values()
        .any(|station| !station.dishes.is_empty())
}

fn move_customer_to_empty_table(
    index: usize,
    data: &GameData,
    game_state: &mut GameState,
    max_tables: usize,
) {
    let occupied = occupied_tables(game_state, max_tables);
    let empty_tables: Vec<usize> = (0..max_tables)
        .filter(|table| !occupied.contains(table))
        .collect();
    let Some(next) = macroquad_toolkit::rng::choose(&empty_tables).copied() else {
        return;
    };

    let display_name = {
        let customer = &mut game_state.customers[index];
        customer.table_index = next;
        customer.is_seated = false;
        customer.display_name.clone()
    };
    game_state.add_message(data.text_format(
        "message_trait_wandered",
        [("name", display_name), ("table", (next + 1).to_string())].as_slice(),
    ));
}

fn occupied_tables(game_state: &GameState, max_tables: usize) -> HashSet<usize> {
    game_state
        .customers
        .iter()
        .filter_map(|customer| (customer.table_index < max_tables).then_some(customer.table_index))
        .collect()
}

fn steal_dish(game_state: &mut GameState) -> Option<(String, String)> {
    let candidates: Vec<String> = game_state
        .cooking_stations
        .iter()
        .filter_map(|(color, station)| {
            if station.dishes.is_empty() {
                None
            } else {
                Some(color.clone())
            }
        })
        .collect();

    let station_color = macroquad_toolkit::rng::choose(&candidates).cloned()?;

    let station = game_state.cooking_stations.get_mut(&station_color)?;
    if station.dishes.is_empty() {
        return None;
    }
    let dish = station.dishes.remove(0);
    Some((station_color, dish.name))
}
