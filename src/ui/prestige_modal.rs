//! Prestige perk choice: resetting the house is a milestone, so it comes with
//! a decision — pick the one thing that survives the reset.

use super::common::{GOLD, LINE, MUTED, TEXT};
use super::types::UiActions;
use crate::data::GameData;
use crate::state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, wrap_text};

const CARD_W: f32 = 250.0;
const CARD_H: f32 = 210.0;
const CARD_GAP: f32 = 16.0;

pub(super) fn draw_prestige_modal(game: &GameState, data: &GameData, ui: &mut UiActions) {
    if !game.pending_prestige || data.prestige_perks.is_empty() {
        return;
    }

    let width = screen_width();
    let height = screen_height();
    draw_rectangle(
        0.0,
        0.0,
        width,
        height,
        Color::new(0.01, 0.008, 0.012, 0.75),
    );
    ui.modal_open = true;

    let count = data.prestige_perks.len().min(4);
    let columns = count.min(if width < 1100.0 { 2 } else { 4 }).max(1);
    let card_w = ((width - 32.0 - CARD_GAP * (columns as f32 - 1.0)) / columns as f32)
        .min(CARD_W)
        .max(180.0);
    let card_h = (height - 96.0).clamp(160.0, CARD_H);
    let total_w = card_w * columns as f32 + CARD_GAP * (columns as f32 - 1.0);
    let start_x = width * 0.5 - total_w * 0.5;
    let rows = count.div_ceil(columns);
    let total_h = card_h * rows as f32 + CARD_GAP * (rows as f32 - 1.0);
    let top = height * 0.5 - total_h * 0.5;

    let headline = data.text("ui_prestige_question");
    let headline_dim = measure_ui_text(headline, None, 24, 1.0);
    draw_ui_text(
        headline,
        width * 0.5 - headline_dim.width * 0.5,
        top - 40.0,
        24.0,
        GOLD,
    );

    for (index, perk) in data.prestige_perks.iter().take(count).enumerate() {
        let column = index % columns;
        let row = index / columns;
        let card = Rect::new(
            start_x + column as f32 * (card_w + CARD_GAP),
            top + row as f32 * (card_h + CARD_GAP),
            card_w,
            card_h,
        );
        let hovered = card.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(
            card.x,
            card.y,
            card.w,
            card.h,
            if hovered {
                Color::new(0.10, 0.075, 0.085, 0.99)
            } else {
                Color::new(0.05, 0.042, 0.050, 0.98)
            },
        );
        draw_rectangle_lines(
            card.x,
            card.y,
            card.w,
            card.h,
            if hovered { 2.5 } else { 1.5 },
            if hovered { GOLD } else { LINE },
        );
        draw_ui_text(&perk.name, card.x + 14.0, card.y + 28.0, 17.0, GOLD);
        let mut y = card.y + 52.0;
        for line in wrap_text(&perk.description, card.w - 28.0, 13.0) {
            draw_ui_text(&line, card.x + 14.0, y, 13.0, TEXT);
            y += 17.0;
        }
        draw_ui_text(
            data.text("ui_click_prestige"),
            card.x + 14.0,
            card.y + card.h - 12.0,
            12.0,
            if hovered { GOLD } else { MUTED },
        );
        ui.prestige_perk_buttons.insert(perk.id.clone(), card);
    }
}
