//! Quiet pause, help, and recent-history surfaces reachable during service.

use super::common::{draw_button, GOLD, LINE, MUTED, TEXT};
use super::types::{OverlayKind, UiActions};
use crate::data::GameData;
use crate::state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, wrap_text};

fn draw_overlay_frame(title: &str, width: f32, height: f32) -> Rect {
    draw_rectangle(
        0.0,
        0.0,
        width,
        height,
        Color::new(0.01, 0.008, 0.012, 0.82),
    );
    let panel_w = (width - 32.0).clamp(300.0, 680.0);
    let panel_h = (height - 32.0).clamp(260.0, 620.0);
    let panel = Rect::new(
        width * 0.5 - panel_w * 0.5,
        height * 0.5 - panel_h * 0.5,
        panel_w,
        panel_h,
    );
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.045, 0.038, 0.045, 0.99),
    );
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, GOLD);
    draw_ui_text(title, panel.x + 24.0, panel.y + 38.0, 26.0, GOLD);
    draw_line(
        panel.x + 24.0,
        panel.y + 54.0,
        panel.x + panel.w - 24.0,
        panel.y + 54.0,
        1.0,
        LINE,
    );
    panel
}

pub(super) fn draw_pause_overlay(data: &GameData, ui: &mut UiActions) {
    let panel = draw_overlay_frame(data.text("ui_pause_title"), screen_width(), screen_height());
    ui.modal_open = true;
    ui.overlay = OverlayKind::Pause;
    draw_ui_text(
        data.text("ui_pause_hint"),
        panel.x + 24.0,
        panel.y + 86.0,
        16.0,
        MUTED,
    );
    let button_w = (panel.w - 48.0).min(360.0);
    let x = panel.x + (panel.w - button_w) * 0.5;
    let mut y = panel.y + 116.0;
    let buttons = [
        (data.text("ui_resume"), 0),
        (data.text("ui_help"), 1),
        (data.text("menu_settings"), 2),
        (data.text("ui_return_title"), 3),
    ];
    for (label, index) in buttons {
        let rect = Rect::new(x, y, button_w, 48.0);
        draw_button(rect, label, index == 0, index == 3);
        match index {
            0 => ui.pause_resume = Some(rect),
            1 => ui.pause_help = Some(rect),
            2 => ui.pause_settings = Some(rect),
            _ => ui.pause_title = Some(rect),
        }
        y += 58.0;
    }
}

pub(super) fn draw_help_overlay(data: &GameData, ui: &mut UiActions) {
    let panel = draw_overlay_frame(data.text("ui_help_title"), screen_width(), screen_height());
    ui.modal_open = true;
    ui.overlay = OverlayKind::Help;
    let lines = [
        data.text("help_cook"),
        data.text("help_carry"),
        data.text("help_serve"),
        data.text("help_vip"),
        data.text("help_clear"),
        data.text("help_inspect"),
    ];
    let mut y = panel.y + 88.0;
    for line in lines {
        draw_ui_text("•", panel.x + 28.0, y, 18.0, GOLD);
        draw_ui_text(line, panel.x + 50.0, y, 16.0, TEXT);
        y += 30.0;
    }
    draw_ui_text(
        data.text("help_touch"),
        panel.x + 28.0,
        y + 8.0,
        14.0,
        MUTED,
    );
    let close = Rect::new(
        panel.x + panel.w - 132.0,
        panel.y + panel.h - 62.0,
        108.0,
        44.0,
    );
    draw_button(close, data.text("ui_close"), true, false);
    ui.help_close = Some(close);
}

pub(super) fn draw_history_overlay(game: &GameState, data: &GameData, ui: &mut UiActions) {
    let panel = draw_overlay_frame(
        data.text("ui_history_title"),
        screen_width(),
        screen_height(),
    );
    ui.modal_open = true;
    ui.overlay = OverlayKind::History;
    draw_ui_text(
        data.text("ui_history_hint"),
        panel.x + 24.0,
        panel.y + 82.0,
        14.0,
        MUTED,
    );
    let mut y = panel.y + 112.0;
    let bottom = panel.y + panel.h - 78.0;
    for message in game.messages.iter().rev() {
        let lines = wrap_text(message, panel.w - 64.0, 15.0);
        if y + lines.len() as f32 * 20.0 > bottom {
            break;
        }
        for line in lines {
            draw_ui_text(&line, panel.x + 32.0, y, 15.0, TEXT);
            y += 20.0;
        }
        y += 8.0;
    }
    let close = Rect::new(
        panel.x + panel.w - 132.0,
        panel.y + panel.h - 62.0,
        108.0,
        44.0,
    );
    draw_button(close, data.text("ui_close"), true, false);
    ui.history_close = Some(close);
}
