//! Kitchen stations, cooking controls, and the carried-dish affordance.

use super::actors::draw_player_actor_scaled;
use super::common::{
    dish_label, draw_bar, draw_button, draw_centered_section_title, draw_panel, draw_row_value,
    draw_station_dots, kitchen_to_screen, station_draw_color, ACCENT, CARD, GOLD, LINE, MUTED,
    SUCCESS, TEXT,
};
use super::sprites::{self, Region};
use super::types::UiActions;

/// Interim-art appliance backing each cooking station, themed to its course.
fn station_region(color: &str) -> Region {
    match color {
        "green" => Region::PotStove, // Hearth Broth — simmering pot
        "yellow" => Region::Stove,   // Butcher's Roast — range
        "red" => Region::Mixer,      // Velvet Sweets — mixer bench
        _ => Region::PrepVeg,        // Forager Toasts — prep counter
    }
}
use crate::data::{GameData, STATION_COLORS};
use crate::engine::kitchen_pass_position;
use crate::state::{CookingStation, GameState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw_kitchen(
    panel: Rect,
    game: &GameState,
    data: &GameData,
    selected_station: &Option<String>,
    interior_sheet: Option<&Texture2D>,
    ui: &mut UiActions,
) {
    draw_panel(panel);
    draw_centered_section_title(data.text("ui_kitchen"), panel);

    let compact = panel.h < 500.0 || panel.w < 240.0;
    let hero_h = if compact {
        (panel.h * 0.20).clamp(64.0, 92.0)
    } else {
        (panel.h * 0.22).clamp(124.0, 150.0)
    };
    let hero = Rect::new(panel.x + 14.0, panel.y + 44.0, panel.w - 28.0, hero_h);
    draw_kitchen_hero(hero, game, data, interior_sheet);

    let ready = game
        .cooking_stations
        .values()
        .map(|station| station.dishes.len())
        .sum::<usize>();
    let cooking = game
        .cooking_stations
        .values()
        .filter(|station| station.is_cooking)
        .count();
    draw_row_value(
        data.text("ui_station_load"),
        &data.text_format(
            "ui_station_load_value",
            [
                ("cooking", cooking.to_string()),
                ("ready", ready.to_string()),
            ]
            .as_slice(),
        ),
        Rect::new(hero.x + 12.0, hero.y + hero.h + 10.0, hero.w - 24.0, 22.0),
        SUCCESS,
    );

    let title_y = hero.y + hero.h + 46.0;
    draw_centered_section_title(
        &data.text_format(
            "ui_ready_count",
            [("ready", ready.min(4).to_string())].as_slice(),
        ),
        Rect::new(panel.x, title_y - 28.0, panel.w, 36.0),
    );

    let row_gap = if compact { 4.0 } else { 8.0 };
    let clear_h = if compact { 30.0 } else { 40.0 };
    let start_y = title_y + 14.0;
    let available_h = panel.y + panel.h - clear_h - 18.0 - start_y;
    let row_h = if compact {
        ((available_h - row_gap * 3.0) / 4.0).clamp(34.0, 68.0)
    } else {
        ((available_h - row_gap * 3.0) / 4.0).clamp(60.0, 96.0)
    };
    let mut y = start_y;
    for color in STATION_COLORS {
        if let Some(station) = game.cooking_stations.get(color) {
            let row = Rect::new(panel.x + 14.0, y, panel.w - 28.0, row_h);
            let is_selected = selected_station.as_deref() == Some(color);
            draw_recipe_station(row, color, station, is_selected, data, interior_sheet, ui);
        }
        y += row_h + row_gap;
    }

    let clear_rect = Rect::new(
        panel.x + 18.0,
        panel.y + panel.h - clear_h - 10.0,
        panel.w - 36.0,
        clear_h,
    );
    draw_button(clear_rect, data.text("ui_clear_carried"), false, false);
    ui.clear_selection = Some(clear_rect);
}

fn draw_kitchen_hero(rect: Rect, game: &GameState, data: &GameData, sheet: Option<&Texture2D>) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.08, 0.065, 0.052, 1.0),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE);
    draw_rectangle(
        rect.x + 12.0,
        rect.y + rect.h - 44.0,
        rect.w - 24.0,
        24.0,
        Color::new(0.16, 0.11, 0.07, 1.0),
    );

    let shelf_y = rect.y + rect.h * 0.18;
    let utensil_h = rect.h * 0.30;
    for index in 0..9 {
        let x = rect.x + 26.0 + index as f32 * (rect.w - 52.0) / 8.0;
        draw_line(
            x,
            shelf_y,
            x,
            shelf_y + utensil_h,
            2.0,
            Color::new(0.30, 0.20, 0.12, 1.0),
        );
        draw_circle(
            x,
            shelf_y + utensil_h + 8.0,
            8.0,
            Color::new(0.16, 0.13, 0.11, 1.0),
        );
        draw_circle_lines(x, shelf_y + utensil_h + 8.0, 8.0, 1.0, LINE);
    }

    if let Some(sheet) = sheet {
        sprites::blit_grounded(
            sheet,
            Region::Sink,
            rect.x + rect.w * 0.5,
            rect.y + rect.h - 12.0,
            rect.h * 0.74,
        );
    } else {
        let pot = vec2(rect.x + rect.w * 0.50, rect.y + rect.h * 0.58);
        let pot_r = (rect.h * 0.22).clamp(26.0, 36.0);
        draw_circle(pot.x, pot.y + 8.0, pot_r, Color::new(0.09, 0.08, 0.08, 1.0));
        draw_circle_lines(pot.x, pot.y + 8.0, pot_r + 1.0, 2.0, GOLD);
        draw_rectangle(
            pot.x - pot_r - 10.0,
            pot.y + 4.0,
            (pot_r + 10.0) * 2.0,
            pot_r + 6.0,
            Color::new(0.12, 0.10, 0.09, 1.0),
        );
        draw_circle(pot.x, pot.y + pot_r + 10.0, pot_r * 0.45, ORANGE);
    }

    let pass = kitchen_pass_position();
    let pass_pos = kitchen_to_screen(rect, pass.0, pass.1);
    draw_circle(pass_pos.x, pass_pos.y, 10.0, GOLD);
    if game.player.x < 0.0 {
        draw_player_actor_scaled(
            kitchen_to_screen(rect, game.player.x, game.player.y),
            &game.player,
            0.60,
            data,
            sheet,
        );
    }

    let station_count = data.dish_types.len().max(1) as f32;
    for (index, dish) in data.dish_types.iter().enumerate() {
        let x = rect.x + 26.0 + index as f32 * ((rect.w - 52.0) / station_count);
        draw_circle(
            x,
            rect.y + rect.h - 31.0,
            7.0,
            station_draw_color(&dish.color),
        );
    }
}

fn draw_recipe_station(
    row: Rect,
    color: &str,
    station: &CookingStation,
    is_selected: bool,
    data: &GameData,
    sheet: Option<&Texture2D>,
    ui: &mut UiActions,
) {
    draw_rectangle(
        row.x,
        row.y,
        row.w,
        row.h,
        if is_selected {
            Color::new(0.12, 0.14, 0.18, 1.0)
        } else {
            CARD
        },
    );
    draw_rectangle_lines(
        row.x,
        row.y,
        row.w,
        row.h,
        if is_selected { 2.0 } else { 1.0 },
        if is_selected { ACCENT } else { LINE },
    );

    let plate_r = (row.h * 0.31).clamp(21.0, 30.0);
    let text_x = row.x + 88.0;
    if let Some(sheet) = sheet {
        let icon_h = (row.h * 0.62).clamp(40.0, 56.0);
        sprites::blit_grounded(
            sheet,
            station_region(color),
            row.x + 42.0,
            row.y + row.h - 8.0,
            icon_h,
        );
    } else {
        draw_dish_plate(vec2(row.x + 48.0, row.y + row.h * 0.50), color, plate_r);
    }
    draw_circle(row.x + 16.0, row.y + 18.0, 7.0, station_draw_color(color));
    draw_ui_text(
        &dish_label(data, color),
        text_x,
        row.y + row.h * 0.36,
        16.0,
        TEXT,
    );

    let ready = station.dishes.len();
    // The pass has a clock now: show the oldest plated dish's freshness.
    let (status, status_color) = if let Some(oldest) = station.dishes.first() {
        match crate::engine::classify_dish_age(oldest.age_ms, &data.balance) {
            crate::engine::Freshness::Fresh => (
                data.text_format(
                    "ui_fresh",
                    [(
                        "seconds",
                        format!(
                            "{:.0}",
                            crate::engine::seconds_until_stale(oldest.age_ms, &data.balance)
                        ),
                    )]
                    .as_slice(),
                ),
                SUCCESS,
            ),
            _ => (data.text("ui_going_stale").to_string(), ORANGE),
        }
    } else if station.is_cooking {
        (data.text("ui_cooking").to_string(), MUTED)
    } else {
        (data.text("ui_ready_to_cook").to_string(), MUTED)
    };
    draw_ui_text(&status, text_x, row.y + row.h * 0.63, 14.0, status_color);
    draw_station_dots(
        text_x,
        row.y + row.h - 14.0,
        data.balance.cooking_slots_limit,
        ready,
        station_draw_color(color),
    );

    if station.is_cooking {
        draw_cooking_bar(
            Rect::new(text_x, row.y + row.h - 8.0, row.w - 186.0, 4.0),
            color,
            station.remaining_ms,
            data,
        );
        // Ambient steam puffs so a working stove reads alive at a glance.
        let clock = macroquad::time::get_time() as f32;
        for puff in 0..3 {
            let t = (clock * 0.8 + puff as f32 * 0.33) % 1.0;
            draw_circle(
                row.x + 42.0 + (puff as f32 - 1.0) * 7.0 + (clock * 2.0 + puff as f32).sin() * 3.0,
                row.y + row.h - 14.0 - t * 26.0,
                3.5 * (1.0 - t) + 1.0,
                Color::new(0.85, 0.85, 0.88, 0.30 * (1.0 - t)),
            );
        }
    }

    let can_cook = station.can_cook(data.balance.cooking_slots_limit);
    let button = Rect::new(row.x + row.w - 78.0, row.y + row.h * 0.5 - 14.0, 64.0, 28.0);
    if ready > 0 {
        draw_button(button, data.text("ui_carry"), true, false);
        ui.station_select.push((color.to_string(), row));
    } else if can_cook {
        draw_button(button, data.text("ui_cook"), false, false);
        ui.station_cook.push((color.to_string(), row));
    } else {
        draw_button(button, data.text("ui_busy"), false, true);
    }
}

fn draw_dish_plate(center: Vec2, color: &str, radius: f32) {
    draw_circle(
        center.x,
        center.y,
        radius,
        Color::new(0.20, 0.18, 0.15, 1.0),
    );
    draw_circle(
        center.x,
        center.y - radius * 0.09,
        radius * 0.74,
        Color::new(0.82, 0.78, 0.66, 1.0),
    );
    draw_circle(
        center.x,
        center.y - radius * 0.15,
        radius * 0.50,
        station_draw_color(color),
    );
    draw_circle(
        center.x + radius * 0.20,
        center.y - radius * 0.29,
        radius * 0.18,
        Color::new(0.95, 0.90, 0.70, 1.0),
    );
}

fn draw_cooking_bar(rect: Rect, color: &str, remaining_ms: f32, data: &GameData) {
    let cook_time = data
        .dish_type_by_color(color)
        .map(|dish| dish.cook_time_ms)
        .unwrap_or(1.0);
    let progress = 1.0 - (remaining_ms / cook_time).clamp(0.0, 1.0);
    draw_bar(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        progress,
        1.0,
        station_draw_color(color),
    );
}
