//! Onboarding card: shows the current tutorial step, waits for the matching
//! gameplay event (or a "Got it" click), and always offers a skip.

use super::common::{draw_button, GOLD, LINE, MUTED, TEXT};
use super::types::UiActions;
use crate::data::{GameData, TutorialTrigger};
use crate::state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, wrap_text};

const PANEL_W: f32 = 430.0;
const BODY_FONT: f32 = 15.0;

pub(super) fn draw_tutorial_panel(
    floor: Rect,
    game: &GameState,
    data: &GameData,
    ui: &mut UiActions,
) {
    let Some(step) = game.tutorial.current_step(&data.tutorial_steps) else {
        return;
    };

    let panel_w = PANEL_W.min((floor.w - 16.0).max(180.0));
    let lines = wrap_text(&step.body, panel_w - 28.0, BODY_FONT);
    let needs_ack = step.trigger == TutorialTrigger::Acknowledged;
    let body_h = lines.len() as f32 * 20.0;
    let panel_h = 64.0 + body_h + if needs_ack { 40.0 } else { 12.0 };
    let panel = Rect::new(
        floor.x + (floor.w - panel_w) * 0.5,
        floor.y + 10.0,
        panel_w,
        panel_h,
    );

    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.03, 0.025, 0.032, 0.93),
    );
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.5, GOLD);
    draw_rectangle_lines(
        panel.x + 3.0,
        panel.y + 3.0,
        panel.w - 6.0,
        panel.h - 6.0,
        1.0,
        LINE,
    );

    let step_count = format!(
        "{}/{}",
        game.tutorial.step_index + 1,
        data.tutorial_steps.len()
    );
    draw_ui_text(&step.title, panel.x + 14.0, panel.y + 26.0, 18.0, GOLD);
    let count_dim = measure_ui_text(&step_count, None, 13, 1.0);
    draw_ui_text(
        &step_count,
        panel.x + panel.w - count_dim.width - 60.0,
        panel.y + 24.0,
        13.0,
        MUTED,
    );

    for (index, line) in lines.iter().enumerate() {
        draw_ui_text(
            line,
            panel.x + 14.0,
            panel.y + 50.0 + index as f32 * 20.0,
            BODY_FONT,
            TEXT,
        );
    }

    let skip_rect = Rect::new(panel.x + panel.w - 52.0, panel.y + 8.0, 44.0, 22.0);
    draw_button(skip_rect, "Skip", false, false);
    ui.tutorial_skip = Some(skip_rect);

    if needs_ack {
        let next_rect = Rect::new(
            panel.x + panel.w - 92.0,
            panel.y + panel.h - 36.0,
            80.0,
            28.0,
        );
        draw_button(next_rect, "Got it", true, false);
        ui.tutorial_next = Some(next_rect);
    }
}
