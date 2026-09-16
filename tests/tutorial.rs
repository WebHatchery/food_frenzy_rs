use feast_frenzy::data::{TutorialStep, TutorialTrigger};
use feast_frenzy::state::TutorialProgress;

fn steps() -> Vec<TutorialStep> {
    vec![
        TutorialStep {
            id: "a".to_string(),
            title: "A".to_string(),
            body: "a".to_string(),
            trigger: TutorialTrigger::CookingStarted,
        },
        TutorialStep {
            id: "b".to_string(),
            title: "B".to_string(),
            body: "b".to_string(),
            trigger: TutorialTrigger::Acknowledged,
        },
    ]
}

#[test]
fn only_the_matching_trigger_advances() {
    let steps = steps();
    let mut progress = TutorialProgress::default();
    progress.observe(TutorialTrigger::CourseServed, &steps);
    assert_eq!(progress.step_index, 0);
    progress.observe(TutorialTrigger::CookingStarted, &steps);
    assert_eq!(progress.step_index, 1);
}

#[test]
fn completing_the_last_step_finishes_the_tutorial() {
    let steps = steps();
    let mut progress = TutorialProgress {
        step_index: 1,
        complete: false,
    };
    progress.advance(&steps);
    assert!(progress.complete);
    assert!(progress.current_step(&steps).is_none());
}

#[test]
fn skip_completes_immediately() {
    let steps = steps();
    let mut progress = TutorialProgress::default();
    progress.skip();
    assert!(progress.complete);
    assert!(progress.current_step(&steps).is_none());
}
