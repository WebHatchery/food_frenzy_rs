//! Chef movement and proximity interactions for stations and seated guests.

use crate::data::{GameData, STATION_COLORS};
use crate::engine::{
    kitchen_pass_position, kitchen_station_position, RESTAURANT_FLOOR_HEIGHT,
    RESTAURANT_FLOOR_WIDTH,
};
use crate::gameplay::{dish_display_name, serve_customer, start_cooking};
use crate::state::{GameState, GuestState, ProgressionState};
use macroquad::prelude::*;

pub fn set_player_target(
    game_state: &mut GameState,
    x: f32,
    y: f32,
    task_label: &str,
    carried_station: Option<String>,
    clear_carry_on_arrival: bool,
) {
    game_state.player.target_x = x;
    game_state.player.target_y = y;
    game_state.player.task_label = task_label.to_string();
    game_state.player.carried_station = carried_station;
    game_state.player.clear_carry_on_arrival = clear_carry_on_arrival;
    game_state.player.lock_on_arrival_ms = 0.0;
}

pub fn start_station_with_player(
    station_color: &str,
    data: &GameData,
    progression: &ProgressionState,
    selected_station: &Option<String>,
    game_state: &mut GameState,
) -> bool {
    if game_state.player.action_lock_ms > 0.0 {
        game_state.add_message(data.text("message_cooking_lock"));
        return false;
    }

    let started = start_cooking(station_color, data, progression, game_state);
    if started {
        let carried_station = if game_state.player.clear_carry_on_arrival {
            None
        } else {
            selected_station.clone()
        };
        send_player_to_station(
            station_color,
            data.text("task_cooking"),
            carried_station,
            false,
            game_state,
        );
        game_state.player.lock_on_arrival_ms = data.balance.player_cooking_start_lock_ms;
        game_state.add_message(data.text_format(
            "message_started_cooking",
            [("dish", dish_display_name(data, station_color))].as_slice(),
        ));
    }
    started
}

pub fn select_station_with_player(
    station_color: String,
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
) {
    if game_state.player.action_lock_ms > 0.0 {
        game_state.add_message(data.text("message_cooking_lock"));
        return;
    }

    if selected_station.as_deref() == Some(station_color.as_str()) {
        *selected_station = None;
        clear_player_carry(data, game_state);
    } else {
        *selected_station = Some(station_color.clone());
        send_player_to_station(
            &station_color,
            data.text("task_carrying"),
            Some(station_color.clone()),
            false,
            game_state,
        );
        game_state.tutorial_observe(crate::data::TutorialTrigger::DishCarried, data);
    }
}

pub fn player_target_for_customer(customer_id: u32, game_state: &GameState) -> Option<(f32, f32)> {
    game_state
        .customers
        .iter()
        .find(|customer| customer.id == customer_id)
        .map(|customer| {
            (
                (customer.floor_x - 26.0).clamp(0.0, RESTAURANT_FLOOR_WIDTH),
                (customer.floor_y + 54.0).clamp(0.0, RESTAURANT_FLOOR_HEIGHT),
            )
        })
}

pub fn send_player_to_customer(
    customer_id: u32,
    station_color: &str,
    data: &GameData,
    game_state: &mut GameState,
) {
    if let Some((x, y)) = player_target_for_customer(customer_id, game_state) {
        set_player_target(
            game_state,
            x,
            y,
            data.text("task_serving"),
            Some(station_color.to_string()),
            true,
        );
    }
}

pub fn clear_player_carry(data: &GameData, game_state: &mut GameState) {
    let (x, y) = kitchen_pass_position();
    game_state.player.target_x = x;
    game_state.player.target_y = y;
    game_state.player.carried_station = None;
    game_state.player.clear_carry_on_arrival = false;
    if game_state.player.task_label == data.text("task_carrying")
        || game_state.player.task_label == data.text("task_serving")
    {
        game_state.player.task_label = data.text("task_prep").to_string();
    }
}

pub fn handle_player_keyboard_movement(dt_ms: f32, data: &GameData, game_state: &mut GameState) {
    if game_state.player.action_lock_ms > 0.0 {
        return;
    }

    let movement = keyboard_movement_axis();
    if movement == Vec2::ZERO {
        return;
    }

    let travel = data.balance.player_walk_speed * (dt_ms / 1000.0);
    let player = &mut game_state.player;
    let next = clamp_player_position(
        vec2(player.x, player.y) + movement.normalize() * travel,
        data,
    );

    player.x = next.x;
    player.y = next.y;
    player.target_x = next.x;
    player.target_y = next.y;
    player.lock_on_arrival_ms = 0.0;
    player.clear_carry_on_arrival = false;
    if player.carried_station.is_none() {
        player.task_label = if next.x < 0.0 {
            data.text("task_prep")
        } else {
            data.text("task_floor")
        }
        .to_string();
    }
}

pub fn update_player_movement(dt_ms: f32, data: &GameData, game_state: &mut GameState) {
    let travel = data.balance.player_walk_speed * (dt_ms / 1000.0);
    let player = &mut game_state.player;
    if player.action_lock_ms > 0.0 {
        player.action_lock_ms = (player.action_lock_ms - dt_ms).max(0.0);
        if player.action_lock_ms <= 0.0 && player.task_label == data.text("task_cooking") {
            player.task_label = if player.x < 0.0 {
                data.text("task_prep")
            } else {
                data.text("task_floor")
            }
            .to_string();
        }
        return;
    }

    let dx = player.target_x - player.x;
    let dy = player.target_y - player.y;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance <= travel || distance <= 1.0 {
        player.x = player.target_x;
        player.y = player.target_y;
        if player.lock_on_arrival_ms > 0.0 {
            player.action_lock_ms = player.lock_on_arrival_ms;
            player.lock_on_arrival_ms = 0.0;
            player.task_label = data.text("task_cooking").to_string();
            return;
        }
        if player.clear_carry_on_arrival {
            player.carried_station = None;
            player.clear_carry_on_arrival = false;
            player.task_label = data.text("task_floor").to_string();
        }
    } else if distance > 0.0 {
        let step = travel / distance;
        player.x += dx * step;
        player.y += dy * step;
    }
}

pub fn interact_with_nearest_station(
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression: &ProgressionState,
) {
    let Some(station_color) = nearest_player_station(game_state) else {
        return;
    };

    let has_ready_dish = game_state
        .cooking_stations
        .get(station_color)
        .is_some_and(|station| !station.dishes.is_empty());
    if has_ready_dish {
        select_station_with_player(
            station_color.to_string(),
            data,
            selected_station,
            game_state,
        );
    } else {
        start_station_with_player(
            station_color,
            data,
            progression,
            selected_station,
            game_state,
        );
    }
}

pub fn interact_with_nearest_customer(
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    let Some(station_color) = selected_station.clone() else {
        return false;
    };
    let has_ready_dish = game_state
        .cooking_stations
        .get(&station_color)
        .is_some_and(|station| !station.dishes.is_empty());
    if !has_ready_dish {
        return false;
    }

    let Some(customer_id) = nearest_servable_customer(data, game_state, &station_color) else {
        return false;
    };
    if serve_customer(
        &station_color,
        customer_id,
        data,
        game_state,
        progression,
        guest_state,
    ) {
        send_player_to_customer(customer_id, &station_color, data, game_state);
        *selected_station = None;
        true
    } else {
        false
    }
}

fn send_player_to_station(
    station_color: &str,
    task_label: &str,
    carried_station: Option<String>,
    clear_carry_on_arrival: bool,
    game_state: &mut GameState,
) {
    let (x, y) = kitchen_station_position(station_color);
    set_player_target(
        game_state,
        x,
        y,
        task_label,
        carried_station,
        clear_carry_on_arrival,
    );
}

fn keyboard_movement_axis() -> Vec2 {
    let mut dx = 0.0;
    let mut dy = 0.0;
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        dx -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        dx += 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        dy -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        dy += 1.0;
    }
    vec2(dx, dy)
}

fn clamp_player_position(next: Vec2, data: &GameData) -> Vec2 {
    if next.x < 0.0 {
        vec2(
            next.x.clamp(data.balance.kitchen_service_left, -1.0),
            next.y.clamp(32.0, 160.0),
        )
    } else {
        vec2(
            next.x.clamp(0.0, data.balance.restaurant_floor_width),
            next.y
                .clamp(52.0, data.balance.restaurant_floor_height - 24.0),
        )
    }
}

fn nearest_player_station(game_state: &GameState) -> Option<&'static str> {
    if game_state.player.x >= 0.0 {
        return None;
    }

    STATION_COLORS
        .iter()
        .filter_map(|color| {
            let (x, y) = kitchen_station_position(color);
            let dx = game_state.player.x - x;
            let dy = game_state.player.y - y;
            let distance = (dx * dx + dy * dy).sqrt();
            (distance <= 74.0).then_some((*color, distance))
        })
        .min_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(color, _)| color)
}

fn nearest_servable_customer(
    data: &GameData,
    game_state: &GameState,
    station_color: &str,
) -> Option<u32> {
    if game_state.player.x < 0.0 {
        return None;
    }

    game_state
        .customers
        .iter()
        .filter(|customer| customer.is_seated && customer.next_course_for(station_color).is_some())
        .filter_map(|customer| {
            let dx = game_state.player.x - customer.floor_x;
            let dy = game_state.player.y - customer.floor_y;
            let distance = (dx * dx + dy * dy).sqrt();
            (distance <= data.balance.player_interaction_range).then_some((customer.id, distance))
        })
        .min_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(customer_id, _)| customer_id)
}
