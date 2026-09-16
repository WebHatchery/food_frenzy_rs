use feast_frenzy::data::GameBalance;
use feast_frenzy::engine::{classify_dish_age, freshness_bill_multiplier, Freshness};

fn balance() -> GameBalance {
    GameBalance::default()
}

#[test]
fn dishes_progress_fresh_to_stale_to_spoiled() {
    let balance = balance();
    assert_eq!(classify_dish_age(0.0, &balance), Freshness::Fresh);
    assert_eq!(
        classify_dish_age(balance.dish_fresh_window_ms, &balance),
        Freshness::Stale
    );
    assert_eq!(
        classify_dish_age(balance.dish_spoil_ms, &balance),
        Freshness::Spoiled
    );
}

#[test]
fn only_fresh_dishes_pay_the_bonus() {
    let balance = balance();
    assert!(freshness_bill_multiplier(Freshness::Fresh, &balance) > 1.0);
    assert_eq!(freshness_bill_multiplier(Freshness::Stale, &balance), 1.0);
}
