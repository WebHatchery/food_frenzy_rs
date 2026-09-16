use feast_frenzy::data::GameData;
use feast_frenzy::persistence::{
    load_game, migrate, save_game, validate_version, FoodFrenzySave, SAVE_VERSION,
};

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn can_save_and_load_round_trip() {
    let path = std::env::temp_dir().join(format!(
        "feast_frenzy_save_test_{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    std::env::set_var("feast_FRENZY_TEST_SAVE_PATH", &path);

    let data = GameData::load();
    let game_state = feast_frenzy::state::GameState::new(&data);
    let progression_state = feast_frenzy::state::ProgressionState::from_game_data(&data);
    let guest_state = feast_frenzy::state::GuestState::new();
    let timers = feast_frenzy::state::Timers::new();
    let selected = None;

    let err = save_game(
        &game_state,
        &progression_state,
        &guest_state,
        &timers,
        &selected,
    );
    assert!(err.is_ok(), "save should succeed: {err:?}");

    let loaded = load_game().unwrap_or(None);
    assert!(loaded.is_some());

    std::env::remove_var("feast_FRENZY_TEST_SAVE_PATH");
    let _ = std::fs::remove_file(path);
}

/// A trimmed real v1 save (pre-roadmap schema): no version-2 fields,
/// plated dishes as bare strings, guest records without personalities.
/// Loading it must succeed and migrate rather than wiping the run.
#[test]
fn legacy_v1_save_loads_and_migrates() {
    let legacy = r#"{
        "version": 1,
        "game_state": {
            "score": 420,
            "combo": 2,
            "chain": 3,
            "customers": [],
            "ingredients": { "regular": -1, "pig-meat": 3 },
            "cooking_stations": {
                "blue": {
                    "color": "blue",
                    "is_cooking": false,
                    "remaining_ms": 0.0,
                    "dishes": ["Pickled Clover Tart"]
                }
            },
            "special_table_busy": false,
            "special_table_timer": 0.0,
            "messages": ["Service started."],
            "next_customer_id": 7
        },
        "progression_state": {
            "currency": 55,
            "upgrades": [],
            "recipes": [],
            "achievements": [],
            "prestige_level": 0,
            "prestige_points": 0,
            "total_score": 420,
            "processed_customer_counts": { "pig": 1 },
            "processed_customer_types": ["pig"],
            "feeding_capacity_bonus": 0,
            "crafted_recipe_counts": {},
            "total_dishes_served": 9,
            "preferred_dishes_served": 4,
            "overfed_customer_count": 0,
            "customers_lost": 1,
            "unlocked_customer_types": ["pig", "sheep", "rabbit"]
        },
        "guest_state": {
            "guests": [{
                "id": "guest-1",
                "name": "Marnie",
                "customer_type": "pig",
                "visits": 4,
                "feedings": 6,
                "satisfied_visits": 3,
                "processed_count": 0,
                "last_seen_at": 0
            }]
        },
        "timers": {
            "elapsed_ms": 90000.0,
            "next_spawn_ms": 95000.0,
            "spawn_step": 2,
            "decay_accum_ms": 0.0,
            "patience_accum_ms": 0.0,
            "trait_accum_ms": 0.0,
            "save_accum_ms": 0.0
        },
        "selected_station": null
    }"#;

    let mut save: FoodFrenzySave =
        serde_json::from_str(legacy).expect("legacy v1 save must still deserialize");
    validate_version(&save).expect("legacy version is accepted before migration or import");
    migrate(&mut save);

    assert_eq!(save.version, SAVE_VERSION);
    let station = &save.game_state.cooking_stations["blue"];
    assert_eq!(station.dishes[0].name, "Pickled Clover Tart");
    assert_eq!(station.dishes[0].age_ms, 0.0, "legacy dishes start fresh");
    assert!(save.game_state.tutorial.step_index == 0 && !save.game_state.tutorial.complete);
    assert_eq!(save.game_state.day_cycle.day, 1);
    assert!(!save.game_state.day_cycle.summary_pending);
    assert!(save.guest_state.guests[0].personality.is_none());
    assert!(save.progression_state.specialization.is_none());
    assert_eq!(save.progression_state.currency, 55);
    save.version = SAVE_VERSION + 1;
    assert!(
        validate_version(&save).is_err(),
        "future saves must not be relabeled or imported"
    );
}
