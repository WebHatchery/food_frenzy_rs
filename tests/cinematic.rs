use feast_frenzy::state::{
    CinematicPhase, ProcessingCinematic, CURTAIN_MS, ESCORT_MS, QUIET_MS, REVEAL_MS,
};

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
    assert_eq!(sequence.phase().0, CinematicPhase::Escort);
    sequence.advance(ESCORT_MS + 1.0);
    assert_eq!(sequence.phase().0, CinematicPhase::Curtain);
    sequence.advance(CURTAIN_MS);
    assert_eq!(sequence.phase().0, CinematicPhase::Quiet);
    sequence.advance(QUIET_MS);
    assert_eq!(sequence.phase().0, CinematicPhase::Reveal);
    assert!(sequence.can_dismiss());
    assert!(!sequence.finished());
    sequence.advance(REVEAL_MS);
    assert!(sequence.finished());
}

#[test]
fn dismiss_is_blocked_before_the_reveal() {
    let mut sequence = cinematic();
    sequence.advance(ESCORT_MS + CURTAIN_MS * 0.5);
    assert!(!sequence.can_dismiss());
}
