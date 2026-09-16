use feast_frenzy::data::CinematicTiming;
use feast_frenzy::state::{CinematicPhase, ProcessingCinematic};

fn cinematic() -> ProcessingCinematic {
    ProcessingCinematic::new(
        "Marnie".to_string(),
        "pig".to_string(),
        4,
        "pig-meat".to_string(),
        320,
        64,
        (400.0, 300.0),
    )
}

#[test]
fn phases_advance_in_order() {
    let mut sequence = cinematic();
    let timing = CinematicTiming::default();
    assert_eq!(sequence.phase().0, CinematicPhase::Escort);
    sequence.advance(timing.escort_ms + 1.0);
    assert_eq!(sequence.phase().0, CinematicPhase::Curtain);
    sequence.advance(timing.curtain_ms);
    assert_eq!(sequence.phase().0, CinematicPhase::Quiet);
    sequence.advance(timing.quiet_ms);
    assert_eq!(sequence.phase().0, CinematicPhase::Reveal);
    assert!(sequence.can_dismiss());
    assert!(!sequence.finished());
    sequence.advance(timing.reveal_ms);
    assert!(sequence.finished());
}

#[test]
fn dismiss_is_blocked_before_the_reveal() {
    let mut sequence = cinematic();
    let timing = CinematicTiming::default();
    sequence.advance(timing.escort_ms + timing.curtain_ms * 0.5);
    assert!(!sequence.can_dismiss());
}
