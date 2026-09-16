//! Guest arrival: initial-delay and interval spawns, returning-guest selection,
//! and seating new arrivals at the first empty table.

use crate::data::GameData;
use crate::engine::{
    chance, max_customer_count, max_satisfaction_for_customer, restaurant_entrance_position,
    restaurant_table_position, spawn_interval_ms,
};
use crate::state::{Customer, GameState, GuestState, ProgressionState, Satisfaction, Timers};

pub(super) fn update_spawn(
    data: &GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    now_ms: f64,
) {
    while timers.spawn_step < data.balance.initial_spawn_delays.len() {
        let initial_delay = data.balance.initial_spawn_delays[timers.spawn_step];
        if now_ms >= f64::from(initial_delay) {
            let _ = try_spawn_customer(data, game_state, progression, guest_state, now_ms);
            timers.spawn_step = timers.spawn_step.saturating_add(1);
        } else {
            break;
        }
    }

    let mut interval = f64::from(spawn_interval_ms(data, progression));
    if let Some(crate::data::EventEffect::SpawnRush { multiplier }) =
        game_state.active_event_effect(data)
    {
        interval = (interval * f64::from(*multiplier))
            .max(f64::from(data.balance.min_customer_spawn_interval));
    }
    if now_ms >= timers.next_spawn_ms {
        let _ = try_spawn_customer(data, game_state, progression, guest_state, now_ms);
        timers.next_spawn_ms = now_ms + interval;
    }
}

fn try_spawn_customer(
    data: &GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    guest_state: &mut GuestState,
    now_ms: f64,
) -> bool {
    let max_customers = max_customer_count(data, progression);
    if game_state.customers.len() >= max_customers {
        return false;
    }

    let active_guest_ids: Vec<String> = game_state
        .customers
        .iter()
        .map(|customer| customer.guest_id.clone())
        .collect();
    let returning = if chance(data.balance.returning_guest_chance) {
        guest_state
            .get_returning_unlocked_guest(&progression.unlocked_customer_types, &active_guest_ids)
    } else {
        None
    };

    let selected_type_id = returning
        .as_ref()
        .and_then(|guest| data.customer_type_by_id(&guest.customer_type))
        .cloned()
        .or_else(|| {
            let unlocked_customer_types: Vec<_> = data
                .customer_types
                .iter()
                .filter(|customer_type| progression.is_customer_unlocked(&customer_type.id))
                .collect();
            macroquad_toolkit::rng::choose(&unlocked_customer_types)
                .map(|customer_type| (**customer_type).clone())
        });
    let Some(customer_type) = selected_type_id else {
        return false;
    };

    let guest_record = if let Some(guest) = returning {
        guest
    } else {
        let guest_name = macroquad_toolkit::rng::choose(&data.regulars.names)
            .cloned()
            .unwrap_or_else(|| "Guest".to_string());
        let personality = macroquad_toolkit::rng::choose(&data.regulars.personalities)
            .map(|personality| personality.id.clone());
        guest_state.create_guest(&guest_name, &customer_type.id, personality)
    };

    let Some(table_index) = first_empty_table(game_state, max_customers) else {
        return false;
    };

    let traits = customer_type.special_traits.clone().unwrap_or_default();
    let max_satisfaction =
        max_satisfaction_for_customer(&data.balance, &traits, progression.feeding_capacity_bonus);
    let customer_id = game_state.next_customer_id();
    let (floor_x, floor_y) = restaurant_entrance_position();
    let (target_x, target_y) = restaurant_table_position(table_index, max_customers);
    let order = crate::engine::roll_order(data, &customer_type.id);
    let times_fed = guest_record.satisfied_visits;
    game_state.customers.push(Customer {
        id: customer_id,
        guest_id: guest_record.id.clone(),
        display_name: guest_record.name.clone(),
        customer_type: customer_type.id.clone(),
        satisfaction: Satisfaction::default(),
        max_satisfaction,
        deliciousness: customer_type.base_deliciousness,
        total_satisfaction: 0.0,
        overfed: false,
        table_index,
        arrived_at_ms: now_ms,
        floor_x,
        floor_y,
        target_x,
        target_y,
        is_seated: false,
        bill: 0,
        depart_timer_ms: 0.0,
        order,
        times_fed,
        trait_alert: None,
        personality: guest_record.personality.clone(),
        eating_ms: 0.0,
        waiting_ms: 0.0,
    });
    guest_state.record_guest_visit(&guest_record.id);
    game_state.full_room_bonus_armed = true;
    let arrival_line = guest_record
        .personality
        .as_deref()
        .and_then(|personality| data.personality_by_id(personality))
        .map(|personality| personality.arrival.clone());
    let regular_prefix = if times_fed >= data.balance.regular_visits_threshold {
        "Your regular "
    } else {
        ""
    };
    if times_fed >= crate::engine::visits_until_ready_for(data, &customer_type.id) {
        game_state.add_message(format!(
            "{regular_prefix}{} waddles back in, plump and ready. The Lounge awaits.",
            guest_record.name
        ));
    } else if times_fed == 0 {
        let flavor = arrival_line
            .map(|line| format!(" {} {line}.", guest_record.name))
            .unwrap_or_default();
        game_state.add_message(format!(
            "Welcome in, {}! Table {}.{flavor}",
            guest_record.name,
            table_index + 1
        ));
    } else {
        game_state.add_message(format!(
            "{regular_prefix}{} is back! Table {} (visit {}).",
            guest_record.name,
            table_index + 1,
            times_fed + 1
        ));
    }

    true
}

fn first_empty_table(game_state: &GameState, max_customers: usize) -> Option<usize> {
    (0..max_customers).find(|index| {
        game_state
            .customers
            .iter()
            .all(|customer| customer.table_index != *index)
    })
}
