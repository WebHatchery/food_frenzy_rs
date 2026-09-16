//! End-of-day ledger: the world pauses at close of business and the night's
//! numbers are tallied, with the nearest goal shown so the next day has a
//! purpose. "Open the doors" starts the next day.

use super::common::{can_afford_cost, format_unlock_cost, GOLD, LINE, MUTED, SUCCESS, TEXT};
use super::types::UiActions;
use crate::data::GameData;
use crate::engine::prestige_requirement;
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_text_centered_in_box, draw_ui_text, measure_ui_text};

const MEAT_PINK: Color = Color::new(0.93, 0.52, 0.60, 1.0);

pub(super) fn draw_day_summary(
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
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

    let title = format!("DAY {} - CLOSING LEDGER", game.day_cycle.day);
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
        ("Cash taken", format!("${}", stats.cash_earned), SUCCESS),
        ("Renown earned", stats.renown_earned.to_string(), TEXT),
        ("Guests served", stats.guests_served.to_string(), TEXT),
        (
            "Guests lost",
            stats.guests_lost.to_string(),
            if stats.guests_lost > 0 {
                Color::new(0.94, 0.42, 0.36, 1.0)
            } else {
                MUTED
            },
        ),
        (
            "Sent to the Lounge",
            stats.guests_processed.to_string(),
            MEAT_PINK,
        ),
        ("Meat stocked", stats.meat_gained.to_string(), MEAT_PINK),
        ("Fresh dishes", stats.fresh_dishes.to_string(), TEXT),
        ("Best combo", format!("x{}", stats.best_combo), TEXT),
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
    draw_ui_text("Tomorrow's goal", panel.x + 36.0, y, 15.0, GOLD);
    y += 22.0;
    let goal = nearest_goal(game, progression, data);
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
        &format!("Open Day {}", game.day_cycle.day + 1),
        open_rect.x,
        open_rect.y,
        open_rect.w,
        open_rect.h,
        18.0,
        TEXT,
    );
    ui.day_next_button = Some(open_rect);
}

/// The nearest thing worth chasing, so every ledger ends with a pull forward.
fn nearest_goal(game: &GameState, progression: &ProgressionState, data: &GameData) -> String {
    // 1. An affordable clientele unlock beats everything.
    let mut locked: Vec<_> = data
        .customer_types
        .iter()
        .filter(|customer_type| !progression.is_customer_unlocked(&customer_type.id))
        .collect();
    locked.sort_by_key(|customer_type| customer_type.profile_tier);
    if let Some(next) = locked.first() {
        let cost = format_unlock_cost(&next.unlock_cost);
        if can_afford_cost(game, &next.unlock_cost) {
            return format!(
                "The larder can already attract {} - open the ladder and do it.",
                next.name
            );
        }
        return format!("Attract {} ({}).", next.name, cost);
    }
    // 2. Otherwise, the prestige wall.
    let requirement = prestige_requirement(data, progression);
    if progression.total_score < requirement {
        return format!(
            "Prestige at {} renown ({} to go).",
            requirement,
            requirement - progression.total_score
        );
    }
    "Prestige is ready - cash in whenever it suits the house.".to_string()
}
