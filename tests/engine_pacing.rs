use feast_frenzy::data::GameBalance;
use feast_frenzy::engine::{
    classify_course_pacing, is_kept_waiting, pacing_score_multiplier, CoursePacing,
};
use feast_frenzy::state::{Course, Customer, Satisfaction};

fn customer(served: usize, total: usize, eating_ms: f32, waiting_ms: f32) -> Customer {
    let order = (0..total)
        .map(|index| Course {
            color: "blue".to_string(),
            label: "Main".to_string(),
            served: index < served,
        })
        .collect();
    Customer {
        id: 1,
        guest_id: "guest-1".to_string(),
        display_name: "Test".to_string(),
        customer_type: "pig".to_string(),
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
        order,
        times_fed: 0,
        trait_alert: None,
        personality: None,
        eating_ms,
        waiting_ms,
    }
}

#[test]
fn pacing_covers_the_full_meal_rhythm() {
    let balance = GameBalance::default();
    assert_eq!(
        classify_course_pacing(&customer(0, 3, 0.0, 0.0), &balance),
        CoursePacing::FirstCourse
    );
    assert_eq!(
        classify_course_pacing(&customer(1, 3, 4_000.0, 0.0), &balance),
        CoursePacing::Rushed
    );
    assert_eq!(
        classify_course_pacing(
            &customer(1, 3, 0.0, balance.course_wait_grace_ms - 1.0),
            &balance
        ),
        CoursePacing::WellPaced
    );
    assert_eq!(
        classify_course_pacing(
            &customer(1, 3, 0.0, balance.course_wait_grace_ms + 1.0),
            &balance
        ),
        CoursePacing::KeptWaiting
    );
}

#[test]
fn multipliers_penalize_rush_and_lateness_but_reward_rhythm() {
    let balance = GameBalance::default();
    assert!(pacing_score_multiplier(CoursePacing::Rushed, &balance) < 1.0);
    assert!(pacing_score_multiplier(CoursePacing::KeptWaiting, &balance) < 1.0);
    assert!(pacing_score_multiplier(CoursePacing::WellPaced, &balance) > 1.0);
    assert_eq!(
        pacing_score_multiplier(CoursePacing::FirstCourse, &balance),
        1.0
    );
}

#[test]
fn hangry_state_requires_expired_grace_and_pending_courses() {
    let balance = GameBalance::default();
    let over_grace = balance.course_wait_grace_ms + 1.0;
    assert!(is_kept_waiting(&customer(1, 3, 0.0, over_grace), &balance));
    // Still eating: not hangry.
    assert!(!is_kept_waiting(
        &customer(1, 3, 500.0, over_grace),
        &balance
    ));
    // Order complete: nothing left to wait for.
    assert!(!is_kept_waiting(&customer(3, 3, 0.0, over_grace), &balance));
    // No course served yet: ordinary patience covers the first wait.
    assert!(!is_kept_waiting(&customer(0, 3, 0.0, over_grace), &balance));
}
