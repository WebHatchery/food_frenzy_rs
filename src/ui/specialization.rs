//! House-style choice modal: shown once per run, after the first processing,
//! offering three specializations with real trade-offs. Clicking a card
//! commits; prestige resets the choice.

use super::common::{GOLD, LINE, MUTED, SUCCESS, TEXT};
use super::types::UiActions;
use crate::data::{GameData, SpecializationDef};
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, wrap_text};

const CARD_W: f32 = 300.0;
const CARD_H: f32 = 330.0;
const CARD_GAP: f32 = 20.0;

pub(super) fn specialization_choice_pending(
    game: &GameState,
    progression: &ProgressionState,
) -> bool {
    progression.specialization.is_none()
        && !progression.processed_customer_counts.is_empty()
        && game.tutorial.complete
        && game.processing_cinematic.is_none()
}

pub(super) fn draw_specialization_modal(
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    if !specialization_choice_pending(game, progression) || data.specializations.is_empty() {
        return;
    }

    let width = screen_width();
    let height = screen_height();
    draw_rectangle(
        0.0,
        0.0,
        width,
        height,
        Color::new(0.01, 0.008, 0.012, 0.72),
    );
    ui.modal_open = true;

    let count = data.specializations.len().min(3);
    let columns = count.min(if width < 900.0 { 2 } else { 3 }).max(1);
    let card_w = ((width - 32.0 - CARD_GAP * (columns as f32 - 1.0)) / columns as f32)
        .min(CARD_W)
        .max(180.0);
    let card_h = (height - 96.0).clamp(190.0, CARD_H);
    let total_w = card_w * columns as f32 + CARD_GAP * (columns as f32 - 1.0);
    let start_x = width * 0.5 - total_w * 0.5;
    let rows = count.div_ceil(columns);
    let total_h = card_h * rows as f32 + CARD_GAP * (rows as f32 - 1.0);
    let top = height * 0.5 - total_h * 0.5;

    let headline = data.text("ui_house_style");
    let headline_dim = measure_ui_text(headline, None, 26, 1.0);
    draw_ui_text(
        headline,
        width * 0.5 - headline_dim.width * 0.5,
        top - 46.0,
        26.0,
        GOLD,
    );
    let sub = data.text("ui_house_style_subtitle");
    let sub_dim = measure_ui_text(sub, None, 16, 1.0);
    draw_ui_text(
        sub,
        width * 0.5 - sub_dim.width * 0.5,
        top - 20.0,
        16.0,
        MUTED,
    );

    for (index, spec) in data.specializations.iter().take(count).enumerate() {
        let column = index % columns;
        let row = index / columns;
        let card = Rect::new(
            start_x + column as f32 * (card_w + CARD_GAP),
            top + row as f32 * (card_h + CARD_GAP),
            card_w,
            card_h,
        );
        draw_specialization_card(card, spec, data, ui);
    }
}

fn draw_specialization_card(
    card: Rect,
    spec: &SpecializationDef,
    data: &GameData,
    ui: &mut UiActions,
) {
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let hovered = card.contains(mouse);
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

    draw_ui_text(&spec.name, card.x + 16.0, card.y + 32.0, 20.0, GOLD);
    let mut y = card.y + 60.0;
    for line in wrap_text(&spec.description, card.w - 32.0, 14.0) {
        draw_ui_text(&line, card.x + 16.0, y, 14.0, TEXT);
        y += 19.0;
    }

    y += 10.0;
    for (key, value) in sorted_effects(spec) {
        let good = effect_reads_as_buff(&key, value);
        let text = format!(
            "{} {}",
            if good { "+" } else { "-" },
            describe_effect(data, &key, value)
        );
        draw_ui_text(
            &text,
            card.x + 16.0,
            y,
            13.0,
            if good {
                SUCCESS
            } else {
                Color::new(0.90, 0.48, 0.42, 1.0)
            },
        );
        y += 18.0;
    }

    let mut flavor_y = card.y + card.h - 58.0;
    for line in wrap_text(&spec.flavor, card.w - 32.0, 12.0) {
        draw_ui_text(&line, card.x + 16.0, flavor_y, 12.0, MUTED);
        flavor_y += 15.0;
    }

    draw_ui_text(
        data.text("ui_click_commit"),
        card.x + 16.0,
        card.y + card.h - 12.0,
        12.0,
        if hovered { GOLD } else { MUTED },
    );
    ui.specialization_buttons.insert(spec.id.clone(), card);
}

fn sorted_effects(spec: &SpecializationDef) -> Vec<(String, f64)> {
    let mut effects: Vec<_> = spec
        .effects
        .iter()
        .map(|(key, value)| (key.clone(), *value))
        .collect();
    effects.sort_by(|left, right| left.0.cmp(&right.0));
    effects
}

/// Whether a signed effect delta helps the player, per key semantics
/// (for `*_multiplier` keys where lower is better, negative deltas are buffs).
fn effect_reads_as_buff(key: &str, value: f64) -> bool {
    let lower_is_better = matches!(
        key,
        "cook_time_multiplier" | "spawn_interval_multiplier" | "satisfaction_decay_multiplier"
    );
    if lower_is_better {
        value < 0.0
    } else {
        value > 0.0
    }
}

fn describe_effect(data: &GameData, key: &str, value: f64) -> String {
    let percent = (value.abs() * 100.0).round() as i64;
    let text_key = format!("effect_{key}");
    let template = if data.text(&text_key).is_empty() {
        "effect_unknown"
    } else {
        &text_key
    };
    data.text_format(
        template,
        [
            ("percent", percent.to_string()),
            ("count", value.abs().round().to_string()),
            ("key", key.to_string()),
        ]
        .as_slice(),
    )
}
