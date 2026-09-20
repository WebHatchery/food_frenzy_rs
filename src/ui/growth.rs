//! Quiet current-service feedback; detailed progression lives in management.

use super::common::{draw_button, ellipsize, LINE, MUTED, TEXT};
use super::types::UiActions;
use crate::data::GameData;
use crate::state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw_event_feed(rect: Rect, game: &GameState, data: &GameData, ui: &mut UiActions) {
    if rect.h <= 0.0 {
        return;
    }
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.045, 0.040, 0.048, 0.98),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE);
    draw_ui_text(
        data.text("ui_recent_service"),
        rect.x + 16.0,
        rect.y + 26.0,
        16.0,
        MUTED,
    );
    let recent = game
        .messages
        .last()
        .map(|message| ellipsize(message, 56))
        .unwrap_or_else(|| data.text("ui_no_recent_service").to_string());
    draw_ui_text(&recent, rect.x + 160.0, rect.y + 26.0, 15.0, TEXT);
    let history = Rect::new(rect.x + rect.w - 132.0, rect.y + 4.0, 116.0, 34.0);
    draw_button(
        history,
        &data.text_format(
            "ui_history_button",
            [("count", game.messages.len().to_string())].as_slice(),
        ),
        false,
        false,
    );
    ui.history_button = Some(history);
}
