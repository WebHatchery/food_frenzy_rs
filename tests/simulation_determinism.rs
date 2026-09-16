use feast_frenzy::data::{EventEffect, GameData};
use feast_frenzy::engine::is_regular;
use feast_frenzy::simulation::{eligible_event_ids, select_event_id, update_events};
use feast_frenzy::state::{
    ActiveEvent, Customer, GameState, GuestRecord, GuestState, ProgressionState, Satisfaction,
};

#[test]
fn event_eligibility_and_weighted_rolls_are_deterministic() {
    let data = GameData::load();
    let day_one = eligible_event_ids(&data, 1);
    let day_three = eligible_event_ids(&data, 3);
    assert!(day_one.contains(&"dinner-rush".to_string()));
    assert!(!day_one.contains(&"health-inspector".to_string()));
    assert!(day_three.contains(&"health-inspector".to_string()));
    assert_eq!(select_event_id(&data, 1, 0).as_deref(), Some("dinner-rush"));
    assert_eq!(
        select_event_id(&data, 1, 3).as_deref(),
        Some("generous-evening")
    );
    assert!(select_event_id(&data, 1, u32::MAX).is_none());
}

#[test]
fn active_event_applies_its_effect_and_expires_once() {
    let data = GameData::load();
    let mut game = GameState::new(&data);
    let mut progression = ProgressionState::from_game_data(&data);
    game.active_event = Some(ActiveEvent {
        event_id: "food-critic".to_string(),
        remaining_ms: 1_000.0,
    });
    assert_eq!(
        game.active_event_effect(&data),
        Some(&EventEffect::ServeRenownMultiplier { multiplier: 2.0 })
    );

    update_events(1_100.0, &data, &mut game, &mut progression);

    assert!(game.active_event.is_none());
    assert_eq!(progression.events_completed, 1);
    assert!(game
        .messages
        .iter()
        .any(|message| message.contains("passed")));
}

#[test]
fn returning_guest_keeps_identity_until_processing() {
    let data = GameData::load();
    let guest = GuestRecord {
        id: "guest-marnie".to_string(),
        name: "Marnie".to_string(),
        customer_type: "pig".to_string(),
        visits: 3,
        feedings: 3,
        satisfied_visits: 3,
        processed_count: 0,
        last_seen_at: 0,
        personality: Some("warm-host".to_string()),
    };
    let mut guests = GuestState {
        guests: vec![guest.clone()],
    };
    let returning = guests
        .returning_guest_candidates(&["pig".to_string()], &[])
        .first()
        .copied()
        .expect("the one eligible guest should return");
    assert_eq!(returning.id, guest.id);
    assert_eq!(returning.name, "Marnie");
    guests.record_guest_visit(&guest.id);
    guests.record_guest_satisfied_visit(&guest.id);
    assert_eq!(guests.guests[0].visits, 4);
    assert_eq!(guests.guests[0].satisfied_visits, 4);

    let customer = Customer {
        id: 1,
        guest_id: guest.id.clone(),
        display_name: guest.name,
        customer_type: guest.customer_type,
        satisfaction: Satisfaction::default(),
        max_satisfaction: Satisfaction::default(),
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
        bill: 0,
        depart_timer_ms: 0.0,
        order: Vec::new(),
        times_fed: data.balance.regular_visits_threshold,
        trait_alert: None,
        personality: Some("warm-host".to_string()),
        eating_ms: 0.0,
        waiting_ms: 0.0,
    };
    assert!(is_regular(&customer, &data));
    guests.record_guest_processed(&customer.guest_id);
    assert!(guests
        .returning_guest_candidates(&["pig".to_string()], &[])
        .is_empty());
}
