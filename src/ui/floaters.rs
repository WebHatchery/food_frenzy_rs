//! Draws the transient floating gain numbers spawned by gameplay: they rise
//! and fade above the spot where the value was earned, so every transaction
//! has visible cause and effect.

use super::common::floor_to_screen;
use crate::state::{FloaterAnchor, GameState};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const BACKING: Color = Color::new(0.02, 0.02, 0.025, 1.0);

pub(super) fn draw_floaters(floor: Rect, game: &GameState) {
    let mut header_row = 0usize;
    for (floater, text) in game
        .floaters
        .active
        .iter()
        .zip(game.floaters.runtime_texts())
    {
        let alpha = text.life_fraction();
        let font = text.font_size;
        let dim = measure_ui_text(&text.text, None, font as u16, 1.0);

        let (x, y) = match floater.anchor {
            FloaterAnchor::Floor { x, y } => {
                let screen = floor_to_screen(floor, x, y);
                (
                    screen.x - dim.width * 0.5,
                    screen.y - 96.0 + text.position.y,
                )
            }
            FloaterAnchor::Header => {
                let row = header_row;
                header_row += 1;
                (
                    screen_width() * 0.5 - dim.width * 0.5,
                    96.0 + row as f32 * 22.0 + text.position.y * 0.26,
                )
            }
        };

        let color = with_alpha(text.color, alpha);
        // Soft backing so the number reads over any floor art.
        draw_rectangle(
            x - 6.0,
            y - font + 2.0,
            dim.width + 12.0,
            font + 6.0,
            with_alpha(BACKING, 0.55 * alpha),
        );
        draw_ui_text(&text.text, x, y, font, color);
    }
}
