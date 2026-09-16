use feast_frenzy::data::GameData;
use feast_frenzy::state::ProgressionState;

#[test]
fn progression_starts_with_basic_clientele_only() {
    let data = GameData::load();
    let progression = ProgressionState::from_game_data(&data);

    assert!(progression.is_customer_unlocked("pig"));
    assert!(progression.is_customer_unlocked("sheep"));
    assert!(progression.is_customer_unlocked("rabbit"));
    assert!(!progression.is_customer_unlocked("bear"));
}

#[test]
fn customer_type_unlocks_are_persistent_progression() {
    let data = GameData::load();
    let mut progression = ProgressionState::from_game_data(&data);

    assert!(progression.unlock_customer_type("cow"));
    assert!(progression.is_customer_unlocked("cow"));
    assert!(!progression.unlock_customer_type("cow"));
}
