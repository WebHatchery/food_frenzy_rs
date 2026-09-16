use feast_frenzy::state::{
    FloaterAnchor, FloaterKind, Floaters, FLOATER_LIFETIME_MS, MAX_ACTIVE_FLOATERS,
};

#[test]
fn floaters_expire_after_lifetime() {
    let mut floaters = Floaters::default();
    floaters.spawn_at("+$5", FloaterKind::Cash, 10.0, 10.0);
    floaters.update(FLOATER_LIFETIME_MS - 1.0);
    assert_eq!(floaters.active.len(), 1);
    floaters.update(2.0);
    assert!(floaters.active.is_empty());
}

#[test]
fn floater_count_is_capped() {
    let mut floaters = Floaters::default();
    for index in 0..40 {
        floaters.spawn("+1", FloaterKind::Renown, FloaterAnchor::Header);
        let _ = index;
    }
    assert_eq!(floaters.active.len(), MAX_ACTIVE_FLOATERS);
}
