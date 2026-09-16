//! The Last Meal Lounge processing overlay: a staged, full-screen beat that
//! dims the room, draws the curtain, holds a quiet moment, then reveals the
//! meat payoff. Timing lives in `state::cinematic`; this file only draws.

use super::common::{GOLD, LINE, MUTED, TEXT};
use crate::data::GameData;
use crate::state::{CinematicPhase, GameState, ProcessingCinematic};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const CURTAIN_RED: Color = Color::new(0.30, 0.05, 0.07, 1.0);
const CURTAIN_TRIM: Color = Color::new(0.62, 0.42, 0.16, 1.0);
const MEAT_PINK: Color = Color::new(0.93, 0.52, 0.60, 1.0);

pub(super) fn draw_processing_overlay(game: &GameState, data: &GameData) {
    let Some(cinematic) = &game.processing_cinematic else {
        return;
    };
    let (phase, progress) = cinematic.phase();
    let width = screen_width();
    let height = screen_height();

    let dim = match phase {
        CinematicPhase::Escort => 0.30 + 0.30 * progress,
        CinematicPhase::Curtain | CinematicPhase::Quiet => 0.62,
        CinematicPhase::Reveal => 0.70,
    };
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.01, 0.008, 0.012, dim));

    let stage_w = (width - 24.0).clamp(300.0, 580.0);
    let stage_h = (height - 24.0).clamp(240.0, 360.0);
    let stage = Rect::new(
        width * 0.5 - stage_w * 0.5,
        height * 0.5 - stage_h * 0.5,
        stage_w,
        stage_h,
    );
    match phase {
        CinematicPhase::Escort => draw_escort(cinematic, progress, width, height, data),
        CinematicPhase::Curtain => {
            draw_stage(stage, data);
            draw_curtains(stage, progress);
        }
        CinematicPhase::Quiet => {
            draw_stage(stage, data);
            draw_curtains(stage, 1.0);
            draw_quiet_beat(stage, progress, data);
        }
        CinematicPhase::Reveal => draw_reveal(cinematic, stage, progress, data),
    }
}

fn draw_escort(
    cinematic: &ProcessingCinematic,
    progress: f32,
    width: f32,
    height: f32,
    data: &GameData,
) {
    let text = data.text_format(
        "ui_lounge_escort",
        [("name", cinematic.guest_name.clone())].as_slice(),
    );
    let dots = ".".repeat(1 + ((progress * 6.0) as usize % 3));
    let line = format!("{text}{dots}");
    let dim = measure_ui_text(&line, None, 22, 1.0);
    draw_ui_text(
        &line,
        width * 0.5 - dim.width * 0.5,
        height * 0.80,
        22.0,
        TEXT,
    );
}

fn draw_stage(stage: Rect, data: &GameData) {
    draw_rectangle(
        stage.x,
        stage.y,
        stage.w,
        stage.h,
        Color::new(0.045, 0.030, 0.040, 0.98),
    );
    draw_rectangle_lines(stage.x, stage.y, stage.w, stage.h, 2.0, GOLD);
    draw_ui_text(
        data.text("ui_lounge_title"),
        stage.x + 18.0,
        stage.y + 30.0,
        18.0,
        GOLD,
    );
    draw_line(
        stage.x + 16.0,
        stage.y + 42.0,
        stage.x + stage.w - 16.0,
        stage.y + 42.0,
        1.5,
        LINE,
    );
}

/// Two velvet halves sliding shut across the stage interior.
fn draw_curtains(stage: Rect, progress: f32) {
    let inner = Rect::new(
        stage.x + 8.0,
        stage.y + 50.0,
        stage.w - 16.0,
        stage.h - 60.0,
    );
    let half = inner.w * 0.5 * progress.clamp(0.0, 1.0);
    draw_rectangle(inner.x, inner.y, half, inner.h, CURTAIN_RED);
    draw_rectangle(
        inner.x + inner.w - half,
        inner.y,
        half,
        inner.h,
        CURTAIN_RED,
    );
    if half > 4.0 {
        draw_line(
            inner.x + half,
            inner.y,
            inner.x + half,
            inner.y + inner.h,
            2.0,
            CURTAIN_TRIM,
        );
        draw_line(
            inner.x + inner.w - half,
            inner.y,
            inner.x + inner.w - half,
            inner.y + inner.h,
            2.0,
            CURTAIN_TRIM,
        );
    }
}

fn draw_quiet_beat(stage: Rect, progress: f32, data: &GameData) {
    // A held breath; the shake is a deterministic wobble, no RNG.
    let wobble = if progress > 0.45 && progress < 0.75 {
        ((progress * 80.0).sin() * 3.0).abs()
    } else {
        0.0
    };
    let text = if progress > 0.45 {
        data.text("ui_lounge_thump")
    } else {
        data.text("ui_lounge_quiet")
    };
    let dim = measure_ui_text(text, None, 20, 1.0);
    draw_ui_text(
        text,
        stage.x + stage.w * 0.5 - dim.width * 0.5 + wobble,
        stage.y + stage.h * 0.55,
        20.0,
        MUTED,
    );
}

fn draw_reveal(cinematic: &ProcessingCinematic, stage: Rect, progress: f32, data: &GameData) {
    draw_stage(stage, data);
    let center_x = stage.x + stage.w * 0.5;
    let appear = (progress * 3.0).clamp(0.0, 1.0);

    let headline = data.text("ui_lounge_headline");
    let headline_dim = measure_ui_text(headline, None, 24, 1.0);
    draw_ui_text(
        headline,
        center_x - headline_dim.width * 0.5,
        stage.y + 92.0,
        24.0,
        Color::new(GOLD.r, GOLD.g, GOLD.b, appear),
    );

    let meat_line = format!("+{}  {}", cinematic.meat_gain, cinematic.meat_type);
    let meat_dim = measure_ui_text(&meat_line, None, 34, 1.0);
    draw_ui_text(
        &meat_line,
        center_x - meat_dim.width * 0.5,
        stage.y + 150.0,
        34.0,
        Color::new(MEAT_PINK.r, MEAT_PINK.g, MEAT_PINK.b, appear),
    );

    let gains_line = format!(
        "+{} renown    +${} house cut",
        cinematic.renown_gain, cinematic.cash_gain
    );
    let gains_dim = measure_ui_text(&gains_line, None, 18, 1.0);
    draw_ui_text(
        &gains_line,
        center_x - gains_dim.width * 0.5,
        stage.y + 192.0,
        18.0,
        Color::new(TEXT.r, TEXT.g, TEXT.b, appear),
    );

    let farewell = cinematic
        .farewell
        .clone()
        .unwrap_or_else(|| format!("{} has joined the menu.", cinematic.guest_name));
    let farewell_dim = measure_ui_text(&farewell, None, 16, 1.0);
    draw_ui_text(
        &farewell,
        center_x - farewell_dim.width * 0.5,
        stage.y + 244.0,
        16.0,
        Color::new(MUTED.r, MUTED.g, MUTED.b, appear),
    );

    if cinematic.can_dismiss() {
        let hint = data.text("ui_return_service");
        let hint_dim = measure_ui_text(hint, None, 14, 1.0);
        let pulse = ((macroquad::time::get_time() * 3.0).sin() * 0.5 + 0.5) as f32;
        draw_ui_text(
            hint,
            center_x - hint_dim.width * 0.5,
            stage.y + stage.h - 24.0,
            14.0,
            Color::new(MUTED.r, MUTED.g, MUTED.b, 0.4 + 0.6 * pulse),
        );
    }
}
