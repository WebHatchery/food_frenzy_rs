use feast_frenzy::state::DayCycle;

#[test]
fn day_ends_once_and_waits_for_the_summary() {
    let mut cycle = DayCycle::default();
    assert!(!cycle.update(1_000.0, 10_000.0));
    assert!(cycle.update(9_500.0, 10_000.0), "day should end");
    assert!(cycle.summary_pending);
    assert!(
        !cycle.update(60_000.0, 10_000.0),
        "clock holds while the ledger is up"
    );
    assert_eq!(cycle.day, 1);
}

#[test]
fn next_day_resets_the_ledger() {
    let mut cycle = DayCycle::default();
    cycle.stats.cash_earned = 120;
    cycle.record_combo(7);
    cycle.update(20_000.0, 10_000.0);
    cycle.start_next_day();
    assert_eq!(cycle.day, 2);
    assert_eq!(cycle.stats.cash_earned, 0);
    assert_eq!(cycle.stats.best_combo, 0);
    assert!(!cycle.summary_pending);
    assert!(!cycle.event_fired);
}
