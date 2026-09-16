use feast_frenzy::data::GameData;
use feast_frenzy::engine::{can_process_customer, visits_until_ready_for};
use feast_frenzy::gameplay::serve_customer;
use feast_frenzy::simulation::guests::update_departures;
use feast_frenzy::state::{
    Course, Customer, GameState, GuestState, ProgressionState, Satisfaction,
};

fn seated_customer(id: u32, bill: i64, order: Vec<Course>, times_fed: u32) -> Customer {
    Customer {
        id,
        guest_id: format!("guest-{id}"),
        display_name: "Test".to_string(),
        customer_type: "pig".to_string(),
        satisfaction: Satisfaction::default(),
        max_satisfaction: Satisfaction {
            blue: 40.0,
            green: 40.0,
            yellow: 40.0,
            red: 40.0,
        },
        deliciousness: 1.0,
        total_satisfaction: 0.0,
        overfed: false,
        table_index: 0,
        arrived_at_ms: 0.0,
        floor_x: 0.0,
        floor_y: 0.0,
        target_x: 0.0,
        target_y: 0.0,
        is_seated: true,
        bill,
        depart_timer_ms: 0.0,
        order,
        times_fed,
        trait_alert: None,
        personality: None,
        eating_ms: 0.0,
        waiting_ms: 0.0,
    }
}

fn course(color: &str, served: bool) -> Course {
    Course {
        color: color.to_string(),
        label: "Main".to_string(),
        served,
    }
}

#[test]
fn guest_pays_bill_and_tip_once_order_is_complete() {
    let data = GameData::load();
    let mut game_state = GameState::new(&data);
    let mut progression = ProgressionState::from_game_data(&data);
    // Empty guest list: the satisfied-visit bookkeeping is a no-op here, which
    // keeps the test off macroquad's time API (unavailable headless).
    let mut guest_state = GuestState::new();
    game_state.customers.push(seated_customer(
        1,
        20,
        vec![course("blue", true), course("red", true)],
        0,
    ));

    let dt = data.balance.content_dwell_ms + 100.0;
    update_departures(
        dt,
        &data,
        &mut game_state,
        &mut progression,
        &mut guest_state,
    );

    assert!(game_state.customers.is_empty(), "guest should have left");
    let tip = feast_frenzy::engine::satisfied_tip(&data, 20);
    assert_eq!(progression.currency, 20 + tip);
}

#[test]
fn guest_with_unserved_course_stays_seated() {
    let data = GameData::load();
    let mut game_state = GameState::new(&data);
    let mut progression = ProgressionState::from_game_data(&data);
    let mut guest_state = GuestState::new();
    game_state.customers.push(seated_customer(
        1,
        12,
        vec![course("blue", true), course("red", false)],
        0,
    ));

    let dt = data.balance.content_dwell_ms + 100.0;
    update_departures(
        dt,
        &data,
        &mut game_state,
        &mut progression,
        &mut guest_state,
    );

    assert_eq!(
        game_state.customers.len(),
        1,
        "order not finished, guest stays"
    );
    assert_eq!(progression.currency, 0);
}

#[test]
fn rushing_the_second_course_pays_less_than_pacing_it() {
    let data = GameData::load();
    let mut progression = ProgressionState::from_game_data(&data);
    let mut guest_state = GuestState::new();

    // Scenario A: entree then main served back to back (still eating).
    let mut rushed = GameState::new(&data);
    rushed.customers.push(seated_customer(
        1,
        0,
        vec![course("blue", false), course("blue", false)],
        0,
    ));
    for _ in 0..2 {
        rushed
            .station_mut("blue")
            .unwrap()
            .dishes
            .push(feast_frenzy::state::PlatedDish::new("Toast".to_string()));
    }
    assert!(serve_customer(
        "blue",
        1,
        &data,
        &mut rushed,
        &mut progression,
        &mut guest_state
    ));
    let after_first = rushed.score;
    assert!(rushed.customers[0].eating_ms > 0.0, "eating window armed");
    assert!(serve_customer(
        "blue",
        1,
        &data,
        &mut rushed,
        &mut progression,
        &mut guest_state
    ));
    let rushed_gain = rushed.score - after_first;

    // Scenario B: identical, but the guest finished eating first.
    let mut paced = GameState::new(&data);
    paced.customers.push(seated_customer(
        1,
        0,
        vec![course("blue", false), course("blue", false)],
        0,
    ));
    for _ in 0..2 {
        paced
            .station_mut("blue")
            .unwrap()
            .dishes
            .push(feast_frenzy::state::PlatedDish::new("Toast".to_string()));
    }
    assert!(serve_customer(
        "blue",
        1,
        &data,
        &mut paced,
        &mut progression,
        &mut guest_state
    ));
    let after_first = paced.score;
    paced.customers[0].eating_ms = 0.0;
    paced.customers[0].waiting_ms = 1_000.0;
    assert!(serve_customer(
        "blue",
        1,
        &data,
        &mut paced,
        &mut progression,
        &mut guest_state
    ));
    let paced_gain = paced.score - after_first;

    assert!(
        rushed_gain < paced_gain,
        "rushed second course ({rushed_gain}) must pay less than a paced one ({paced_gain})"
    );
}

#[test]
fn readiness_is_reached_after_enough_fed_visits() {
    let data = GameData::load();
    // The helper builds tier-1 pigs, so the tier ladder's first entry applies.
    let needed = visits_until_ready_for(&data, "pig");
    let below = seated_customer(1, 0, vec![course("blue", false)], needed - 1);
    let ready = seated_customer(2, 0, vec![course("blue", false)], needed);
    assert!(!can_process_customer(&below, &data));
    assert!(can_process_customer(&ready, &data));
}

#[test]
fn tier_one_guests_are_ready_sooner_than_the_flat_fallback() {
    let data = GameData::load();
    let tier_one = visits_until_ready_for(&data, "pig");
    assert!(
        tier_one < data.balance.visits_until_ready,
        "early pacing retune: tier 1 must be faster than the old flat gate"
    );
    let unknown = visits_until_ready_for(&data, "not-a-type");
    assert_eq!(unknown, tier_one, "unknown types fall back to tier 1");
}
