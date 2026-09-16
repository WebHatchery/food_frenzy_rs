//! Shared drawing primitives, colors, labels, hit-target sizing, and meters.

use crate::data::GameData;
use crate::engine::{
    patience_multiplier, KITCHEN_SERVICE_LEFT, RESTAURANT_FLOOR_HEIGHT, RESTAURANT_FLOOR_WIDTH,
};
use crate::state::{Customer, GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_text_centered_in_box, progress_bar};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use std::collections::HashMap;

pub(super) const BACKGROUND: Color = Color::new(0.035, 0.032, 0.038, 1.0);
pub(super) const PANEL: Color = Color::new(0.060, 0.055, 0.065, 0.96);
pub(super) const CARD: Color = Color::new(0.075, 0.068, 0.075, 0.98);
pub(super) const TEXT: Color = Color::new(0.93, 0.89, 0.80, 1.0);
pub(super) const MUTED: Color = Color::new(0.61, 0.58, 0.55, 1.0);
pub(super) const LINE: Color = Color::new(0.25, 0.20, 0.16, 1.0);
pub(super) const GOLD: Color = Color::new(0.84, 0.60, 0.31, 1.0);
pub(super) const ACCENT: Color = Color::new(0.40, 0.64, 0.92, 1.0);
pub(super) const SUCCESS: Color = Color::new(0.48, 0.78, 0.43, 1.0);

pub(super) fn station_draw_color(color: &str) -> Color {
    match color {
        "blue" => SKYBLUE,
        "green" => LIME,
        "yellow" => YELLOW,
        "red" => ORANGE,
        _ => LIGHTGRAY,
    }
}

pub(super) fn dish_label(data: &GameData, color: &str) -> String {
    data.dish_type_by_color(color)
        .map(|dish| dish.name.clone())
        .unwrap_or_else(|| data.text("ui_dish_unknown").to_string())
}

pub(super) fn ellipsize(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut trimmed: String = text.chars().take(max_chars.saturating_sub(3)).collect();
    trimmed.push_str("...");
    trimmed
}

pub(super) fn customer_fallback_color(customer_type: &str) -> Color {
    match customer_type {
        "pig" => PINK,
        "cow" => BEIGE,
        "sheep" => LIGHTGRAY,
        "rabbit" => WHITE,
        "cat" => GOLD,
        "deer" => BROWN,
        "duck" => YELLOW,
        "chicken" => ORANGE,
        "fish" => SKYBLUE,
        "fox" => RED,
        "goat" => LIME,
        "bear" => DARKBROWN,
        "monkey" => PURPLE,
        _ => MAGENTA,
    }
}

pub(super) fn draw_panel(rect: Rect) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(PANEL).with_border(1.0, LINE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_rectangle_lines(
        rect.x + 3.0,
        rect.y + 3.0,
        rect.w - 6.0,
        rect.h - 6.0,
        1.0,
        Color::new(0.60, 0.40, 0.23, 0.42),
    );
}

pub(super) fn draw_section_title(text: &str, x: f32, y: f32) {
    let title = text.to_uppercase();
    let dim = measure_ui_text(&title, None, 17, 1.0);
    draw_ui_text(&title, x, y, 17.0, GOLD);
    draw_line(x - 36.0, y - 7.0, x - 8.0, y - 7.0, 1.0, LINE);
    draw_line(
        x + dim.width + 8.0,
        y - 7.0,
        x + dim.width + 36.0,
        y - 7.0,
        1.0,
        LINE,
    );
}

pub(super) fn draw_centered_section_title(text: &str, rect: Rect) {
    let title = text.to_uppercase();
    let dim = measure_ui_text(&title, None, 17, 1.0);
    let x = rect.x + (rect.w - dim.width) * 0.5;
    draw_section_title(&title, x, rect.y + 29.0);
}

pub(super) fn draw_button(rect: Rect, text: &str, active: bool, disabled: bool) {
    let color = if disabled {
        Color::new(0.12, 0.11, 0.12, 1.0)
    } else if active {
        Color::new(0.24, 0.16, 0.10, 1.0)
    } else {
        Color::new(0.13, 0.11, 0.10, 1.0)
    };
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(color)
        .with_border(1.0, if disabled { LINE } else { GOLD });
    macroquad_toolkit::ui::draw_surface(rect, &surface);

    let font_size = if text.len() > 13 { 14.0 } else { 16.0 };
    draw_text_centered_in_box(
        text,
        rect.x + 6.0,
        rect.y,
        rect.w - 12.0,
        rect.h,
        font_size,
        if disabled { MUTED } else { TEXT },
    );
}

pub(super) fn draw_menu_button(rect: Rect, text: &str) {
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let hovered = rect.contains(mouse);
    let color = if hovered {
        ACCENT
    } else {
        Color::new(0.16, 0.14, 0.13, 0.96)
    };
    let border = if hovered {
        WHITE
    } else {
        Color::new(0.78, 0.52, 0.30, 1.0)
    };
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(color).with_border(1.5, border);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_text_centered_in_box(
        text,
        rect.x + 8.0,
        rect.y,
        rect.w - 16.0,
        rect.h,
        (rect.h * 0.42).clamp(18.0, 24.0),
        WHITE,
    );
}

pub(super) fn draw_card(rect: Rect, title: &str) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(CARD).with_border(1.0, LINE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_ui_text(title, rect.x + 12.0, rect.y + 25.0, 18.0, GOLD);
}

pub(super) fn draw_tooltip(text: &str, center_x: f32, y: f32) {
    let text_dim = measure_ui_text(text, None, 14, 1.0);
    let rect = Rect::new(
        center_x - text_dim.width * 0.5 - 10.0,
        y,
        text_dim.width + 20.0,
        24.0,
    );
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.02, 0.02, 0.025, 0.86))
        .with_border(1.0, SKYBLUE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_ui_text(text, rect.x + 10.0, rect.y + 17.0, 14.0, TEXT);
}

pub(super) fn draw_bar(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    value: f32,
    max_value: f32,
    color: Color,
) {
    progress_bar(x, y, width, height, value, max_value, color);
}

pub(super) fn draw_resource_tile(rect: Rect, label: &str, value: &str, accent: Option<Color>) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.055, 0.048, 0.055, 1.0))
        .with_border(1.0, LINE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    let accent = accent.unwrap_or(GOLD);
    draw_circle(rect.x + 19.0, rect.y + rect.h * 0.5, 7.0, accent);
    draw_circle_lines(rect.x + 19.0, rect.y + rect.h * 0.5, 10.0, 1.0, GOLD);
    draw_ui_text(label, rect.x + 38.0, rect.y + 19.0, 13.0, MUTED);
    draw_ui_text(value, rect.x + 38.0, rect.y + 42.0, 22.0, TEXT);
}

pub(super) fn draw_row_value(label: &str, value: &str, rect: Rect, color: Color) {
    draw_ui_text(label, rect.x, rect.y + 18.0, 15.0, MUTED);
    let value_dim = measure_ui_text(value, None, 16, 1.0);
    draw_ui_text(
        value,
        rect.x + rect.w - value_dim.width,
        rect.y + 18.0,
        16.0,
        color,
    );
}

pub(super) fn draw_station_dots(x: f32, y: f32, count: usize, filled: usize, color: Color) {
    for index in 0..count {
        draw_circle(
            x + index as f32 * 16.0,
            y,
            4.5,
            if index < filled {
                color
            } else {
                Color::new(0.21, 0.18, 0.15, 1.0)
            },
        );
    }
}

pub(super) fn sorted_ingredient_lines(game: &GameState) -> Vec<String> {
    let mut ingredients: Vec<_> = game
        .ingredients
        .iter()
        .filter(|(name, amount)| name.as_str() != "regular" && **amount > 0)
        .map(|(name, amount)| format!("{name}: {amount}"))
        .collect();
    ingredients.sort();
    ingredients
}

pub(super) fn format_unlock_cost(data: &GameData, cost: &HashMap<String, i64>) -> String {
    if cost.is_empty() {
        return data.text("ui_open").to_string();
    }

    let mut parts: Vec<_> = cost
        .iter()
        .map(|(ingredient, amount)| format!("{amount} {ingredient}"))
        .collect();
    parts.sort();
    parts.join(", ")
}

pub(super) fn can_afford_cost(game: &GameState, cost: &HashMap<String, i64>) -> bool {
    cost.iter()
        .all(|(ingredient, amount)| game.has_ing(ingredient, *amount))
}

pub(super) fn player_near_customer(game: &GameState, customer: &Customer, range: f32) -> bool {
    if game.player.x < 0.0 || !customer.is_seated {
        return false;
    }
    let dx = game.player.x - customer.floor_x;
    let dy = game.player.y - customer.floor_y;
    (dx * dx + dy * dy).sqrt() <= range
}

fn patience_limit_ms(customer: &Customer, data: &GameData, progression: &ProgressionState) -> f32 {
    let traits = customer.traits(data);
    let mut patience = data.balance.customer_patience_time * patience_multiplier(progression);
    if traits.fast_spoilage {
        patience *= 0.55;
    }
    patience.max(1.0)
}

pub(super) fn patience_remaining_ratio(
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
) -> f32 {
    let elapsed = (now_ms - customer.arrived_at_ms).max(0.0) as f32;
    (1.0 - elapsed / patience_limit_ms(customer, data, progression)).clamp(0.0, 1.0)
}

pub(super) fn patience_color(ratio: f32) -> Color {
    if ratio < 0.24 {
        RED
    } else if ratio < 0.48 {
        ORANGE
    } else {
        LIME
    }
}

pub(super) fn floor_to_screen(floor: Rect, world_x: f32, world_y: f32) -> Vec2 {
    vec2(
        floor.x + (world_x / RESTAURANT_FLOOR_WIDTH) * floor.w,
        floor.y + (world_y / RESTAURANT_FLOOR_HEIGHT) * floor.h,
    )
}

pub(super) fn kitchen_to_screen(panel: Rect, world_x: f32, world_y: f32) -> Vec2 {
    let x_t = ((world_x - KITCHEN_SERVICE_LEFT) / -KITCHEN_SERVICE_LEFT).clamp(0.0, 1.0);
    let y_t = (world_y / RESTAURANT_FLOOR_HEIGHT).clamp(0.0, 1.0);
    vec2(
        panel.x + 22.0 + x_t * (panel.w - 44.0),
        panel.y + y_t * panel.h,
    )
}
