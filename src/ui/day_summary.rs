//! End-of-day ledger: the world pauses at close of business and the night's
//! numbers are tallied, with the nearest goal shown so the next day has a
//! purpose. "Open the doors" starts the next day.

use super::common::{GOLD, LINE, MUTED, SUCCESS, TEXT};
use super::types::{OverlayKind, UiActions};
use crate::data::GameData;
use crate::state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_text_centered_in_box, draw_ui_text, measure_ui_text};

const MEAT_PINK: Color = Color::new(0.93, 0.52, 0.60, 1.0);

pub(super) fn draw_day_summary(game: &GameState, data: &GameData, ui: &mut UiActions) {
    if !game.day_cycle.summary_pending {
        return;
    }

    let width = screen_width();
    let height = screen_height();
    draw_rectangle(
        0.0,
        0.0,
        width,
        height,
        Color::new(0.01, 0.008, 0.012, 0.80),
    );
    ui.modal_open = true;
    ui.overlay = OverlayKind::DaySummary;

    let panel_w = (width - 24.0).clamp(280.0, 540.0);
    let panel_h = (height - 24.0).clamp(220.0, 500.0);
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

    let title = data.text_format(
        "summary_title",
        [("day", game.day_cycle.day.to_string())].as_slice(),
    );
    let title_dim = measure_ui_text(&title, None, 24, 1.0);
    draw_ui_text(
        &title,
        panel.x + (panel.w - title_dim.width) * 0.5,
        panel.y + 40.0,
        24.0,
        GOLD,
    );
    draw_line(
        panel.x + 24.0,
        panel.y + 56.0,
        panel.x + panel.w - 24.0,
        panel.y + 56.0,
        1.5,
        LINE,
    );

    let stats = &game.day_cycle.stats;
    let rows: [(&str, String, Color); 8] = [
        (
            data.text("summary_cash"),
            format!("${}", stats.cash_earned),
            SUCCESS,
        ),
        (
            data.text("summary_renown"),
            stats.renown_earned.to_string(),
            TEXT,
        ),
        (
            data.text("summary_served"),
            stats.guests_served.to_string(),
            TEXT,
        ),
        (
            data.text("summary_lost"),
            stats.guests_lost.to_string(),
            if stats.guests_lost > 0 {
                Color::new(0.94, 0.42, 0.36, 1.0)
            } else {
                MUTED
            },
        ),
        (
            data.text("summary_lounge"),
            stats.guests_processed.to_string(),
            MEAT_PINK,
        ),
        (
            data.text("summary_meat"),
            stats.meat_gained.to_string(),
            MEAT_PINK,
        ),
        (
            data.text("summary_fresh"),
            stats.fresh_dishes.to_string(),
            TEXT,
        ),
        (
            data.text("summary_combo"),
            format!("x{}", stats.best_combo),
            TEXT,
        ),
    ];
    let compact = panel_h < 450.0;
    let mut y = panel.y + 88.0;
    if compact {
        for (index, (label, value, color)) in rows.iter().enumerate() {
            let column = index / 4;
            let row = index % 4;
            let x = panel.x + 26.0 + column as f32 * (panel.w * 0.5 - 12.0);
            let row_y = y + row as f32 * 25.0;
            draw_ui_text(label, x, row_y, 13.0, MUTED);
            let value_dim = measure_ui_text(value, None, 13, 1.0);
            draw_ui_text(
                value,
                x + panel.w * 0.5 - 38.0 - value_dim.width,
                row_y,
                13.0,
                *color,
            );
        }
        y += 4.0 * 25.0 + 10.0;
    } else {
        for (label, value, color) in rows {
            draw_ui_text(label, panel.x + 36.0, y, 16.0, MUTED);
            let value_dim = measure_ui_text(&value, None, 16, 1.0);
            draw_ui_text(
                &value,
                panel.x + panel.w - 36.0 - value_dim.width,
                y,
                16.0,
                color,
            );
            y += 28.0;
        }
    }

    draw_line(panel.x + 24.0, y, panel.x + panel.w - 24.0, y, 1.0, LINE);
    y += 26.0;
    draw_ui_text(data.text("summary_goal"), panel.x + 36.0, y, 15.0, GOLD);
    y += 22.0;
    let goal = data
        .goal_by_id(&game.day_cycle.goal_id)
        .map(|goal| format!("{}: {}", goal.title, goal.description))
        .unwrap_or_else(|| data.text("message_goal_prestige_ready").to_string());
    for line in goal.lines() {
        draw_ui_text(line, panel.x + 36.0, y, 14.0, TEXT);
        y += 20.0;
    }

    let open_rect = Rect::new(
        panel.x + panel.w * 0.5 - 110.0,
        panel.y + panel.h - 56.0,
        220.0,
        38.0,
    );
    let hovered = open_rect.contains(vec2(mouse_position().0, mouse_position().1));
    draw_rectangle(
        open_rect.x,
        open_rect.y,
        open_rect.w,
        open_rect.h,
        if hovered {
            Color::new(0.26, 0.18, 0.10, 1.0)
        } else {
            Color::new(0.16, 0.12, 0.08, 1.0)
        },
    );
    draw_rectangle_lines(
        open_rect.x,
        open_rect.y,
        open_rect.w,
        open_rect.h,
        1.5,
        GOLD,
    );
    draw_text_centered_in_box(
        &data.text_format(
            "ui_open_day",
            [("day", (game.day_cycle.day + 1).to_string())].as_slice(),
        ),
        open_rect.x,
        open_rect.y,
        open_rect.w,
        open_rect.h,
        18.0,
        TEXT,
    );
    ui.day_next_button = Some(open_rect);
}
