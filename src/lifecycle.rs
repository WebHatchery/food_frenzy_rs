//! Start and restore active runs, including compatibility repair for older
//! saves and newly added content.

use crate::data::GameData;
use crate::engine::{kitchen_pass_position, max_customer_count, restaurant_table_position};
use crate::persistence::{load_game, FoodFrenzySave};
use crate::state::{
    CookingStation, GameState, GuestState, ProgressionState, Timers, INFINITE_INGREDIENTS,
};

pub fn start_new_game(
    data: &GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    selected_station: &mut Option<String>,
) {
    *game_state = GameState::new(data);
    *progression_state = ProgressionState::from_game_data(data);
    *guest_state = GuestState::new();
    *timers = Timers::new();
    *selected_station = None;
    initialize_active_game(
        data,
        progression_state,
        game_state,
        selected_station,
        timers,
        data.text("message_new_game"),
    );
}

pub fn load_saved_game(
    data: &GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    selected_station: &mut Option<String>,
) -> Result<(), String> {
    let Some(saved) = load_game().map_err(|error| format!("Load failed: {error}"))? else {
        return Err(data.text("message_no_save").to_string());
    };

    restore_save(
        saved,
        data,
        game_state,
        progression_state,
        guest_state,
        timers,
        selected_station,
    );
    Ok(())
}

fn restore_save(
    saved: FoodFrenzySave,
    data: &GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    selected_station: &mut Option<String>,
) {
    *game_state = saved.game_state;
    *progression_state = saved.progression_state;
    *guest_state = saved.guest_state;
    *timers = saved.timers;
    *selected_station = saved.selected_station;
    initialize_active_game(
        data,
        progression_state,
        game_state,
        selected_station,
        timers,
        data.text("message_loaded_game"),
    );
}

fn initialize_active_game(
    data: &GameData,
    progression: &mut ProgressionState,
    game_state: &mut GameState,
    selected_station: &mut Option<String>,
    timers: &mut Timers,
    startup_message: &str,
) {
    progression.ensure_customer_unlocks(data);
    ensure_compatibility(data, progression, game_state, selected_station);
    progression.set_upgrade_costs();
    if game_state.day_cycle.goal_id.is_empty()
        || data.goal_by_id(&game_state.day_cycle.goal_id).is_none()
    {
        game_state.day_cycle.goal_id =
            crate::engine::select_next_day_goal(data, game_state.day_cycle.day, progression).id;
    }
    if !startup_message.is_empty()
        && !game_state
            .messages
            .iter()
            .any(|message| message == startup_message)
    {
        game_state.add_message(startup_message.to_string());
    }
    if timers.next_spawn_ms <= 0.0 {
        timers.next_spawn_ms = f64::from(data.balance.customer_spawn_interval);
    }
    timers.save_timer.reset();
}

fn ensure_compatibility(
    data: &GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
    selected_station: &mut Option<String>,
) {
    for dish in &data.dish_types {
        if !game_state.cooking_stations.contains_key(&dish.color) {
            game_state
                .cooking_stations
                .insert(dish.color.clone(), CookingStation::new(dish.color.clone()));
        }
    }

    if !game_state
        .ingredients
        .contains_key(&data.balance.regular_ingredient_name)
    {
        game_state.ingredients.insert(
            data.balance.regular_ingredient_name.clone(),
            INFINITE_INGREDIENTS,
        );
    }

    if let Some(station) = selected_station {
        if !game_state.cooking_stations.contains_key(station) {
            *selected_station = None;
        }
    }

    let max_tables = max_customer_count(data, progression);
    for customer in &mut game_state.customers {
        let (target_x, target_y) = restaurant_table_position(customer.table_index, max_tables);
        customer.target_x = target_x;
        customer.target_y = target_y;
        if customer.floor_x == 0.0 && customer.floor_y == 0.0 {
            customer.floor_x = target_x;
            customer.floor_y = target_y;
            customer.is_seated = true;
        }
        // Saves from before the course system have no order; give them one so
        // they can be served and leave instead of getting stuck.
        if customer.order.is_empty() {
            customer.order = crate::engine::roll_order(data, &customer.customer_type);
        }
    }

    if game_state.player.carried_station.is_none()
        && (game_state.player.x + 165.0).abs() <= 1.0
        && (game_state.player.y - 72.0).abs() <= 1.0
    {
        let (x, y) = kitchen_pass_position();
        game_state.player.x = x;
        game_state.player.y = y;
        game_state.player.target_x = x;
        game_state.player.target_y = y;
    }
}
