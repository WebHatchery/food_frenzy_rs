//! House-style choice modal: shown once per run, after the first processing,
//! offering three specializations with real trade-offs. Clicking a card
//! commits; prestige resets the choice.

use super::common::{GOLD, LINE, MUTED, SUCCESS, TEXT};
use super::types::{OverlayKind, UiActions};
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
    ui.overlay = OverlayKind::Specialization;

    let narrow = width < 900.0;
    let page_size = if narrow { 1 } else { 3 };
    let page_count = data.specializations.len().div_ceil(page_size).max(1);
    let page = game.specialization_page.min(page_count.saturating_sub(1));
    let visible = data
        .specializations
        .iter()
        .skip(page * page_size)
        .take(page_size);
    let count = visible.len();
    let columns = count.max(1);
    let card_w =
        ((width - 32.0 - CARD_GAP * (columns as f32 - 1.0)) / columns as f32).clamp(180.0, CARD_W);
    let card_h = if narrow {
        (height - 150.0).clamp(190.0, CARD_H)
    } else {
        (height - 96.0).clamp(190.0, CARD_H)
    };
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

    for (index, spec) in visible.enumerate() {
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
    if narrow && page_count > 1 {
        let previous = Rect::new(width * 0.5 - 156.0, height - 52.0, 120.0, 42.0);
        let next = Rect::new(width * 0.5 + 36.0, height - 52.0, 120.0, 42.0);
        super::common::draw_button(previous, data.text("ui_previous"), page > 0, page == 0);
        super::common::draw_button(
            next,
            data.text("ui_next"),
            page + 1 < page_count,
            page + 1 >= page_count,
        );
        ui.specialization_previous = Some(previous);
        ui.specialization_next = Some(next);
        draw_ui_text(
            &format!("{} / {}", page + 1, page_count),
            width * 0.5 - 18.0,
            height - 22.0,
            14.0,
            MUTED,
        );
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
        let text = describe_effect(data, &key, value);
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
        data.text("ui_choose"),
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
    let template = effect_text_key(key, value);
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

fn effect_text_key(key: &str, value: f64) -> &'static str {
    match (key, value.is_sign_positive()) {
        ("cook_time_multiplier", true) => "effect_cook_time_longer",
        ("cook_time_multiplier", false) => "effect_cook_time_shorter",
        ("spawn_interval_multiplier", true) => "effect_spawn_interval_slower",
        ("spawn_interval_multiplier", false) => "effect_spawn_interval_faster",
        ("satisfaction_decay_multiplier", true) => "effect_decay_faster",
        ("satisfaction_decay_multiplier", false) => "effect_decay_slower",
        ("patience_multiplier", true) => "effect_patience_longer",
        ("patience_multiplier", false) => "effect_patience_shorter",
        ("meat_yield_multiplier", true) => "effect_meat_yield_more",
        ("meat_yield_multiplier", false) => "effect_meat_yield_less",
        ("combo_multiplier", true) => "effect_combo_more",
        ("combo_multiplier", false) => "effect_combo_less",
        ("recipe_value_multiplier", true) => "effect_recipe_value_more",
        ("recipe_value_multiplier", false) => "effect_recipe_value_less",
        ("capacity_gain_multiplier", true) => "effect_capacity_more",
        ("capacity_gain_multiplier", false) => "effect_capacity_less",
        ("max_customers_bonus", true) => "effect_tables_more",
        ("max_customers_bonus", false) => "effect_tables_less",
        _ => "effect_unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::effect_text_key;

    #[test]
    fn effect_direction_uses_mechanical_meaning() {
        assert_eq!(
            effect_text_key("cook_time_multiplier", 0.10),
            "effect_cook_time_longer"
        );
        assert_eq!(
            effect_text_key("cook_time_multiplier", -0.12),
            "effect_cook_time_shorter"
        );
        assert_eq!(
            effect_text_key("satisfaction_decay_multiplier", 0.10),
            "effect_decay_faster"
        );
        assert_eq!(
            effect_text_key("satisfaction_decay_multiplier", -0.10),
            "effect_decay_slower"
        );
    }
}
