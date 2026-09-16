//! Macroquad entry point for Feast Frenzy.

use macroquad::prelude::*;
use macroquad_toolkit::capture;

fn window_conf() -> Conf {
    capture::capture_window_conf("feast_FRENZY", "Feast Frenzy", 1920, 1080)
}

#[macroquad::main(window_conf)]
async fn main() {
    feast_frenzy::app::run().await;
}
