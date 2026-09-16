//! Dining-floor composition: tables, guests, service prompts, and clock.

mod room;

use super::actors::{draw_customer_sprite, draw_player_actor};
use super::common::{
    dish_label, draw_button, floor_to_screen, station_draw_color, GOLD, LINE, MUTED, TEXT,
};
use super::sprites::{self, Region};
use super::types::UiActions;
use crate::data::GameData;
use crate::engine::{max_customer_count, restaurant_entrance_position, restaurant_table_position};
use crate::state::{Customer, GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use std::collections::HashMap;

fn draw_table(
    center: Vec2,
    table_index: usize,
    customer: Option<&Customer>,
    data: &GameData,
    selected_station: &Option<String>,
    game: &GameState,
    interior_sheet: Option<&Texture2D>,
    ui: &mut UiActions,
) {
    let table_radius = 48.0;
    let occupied = customer.is_some();
    let ready_for_lounge =
        customer.is_some_and(|customer| crate::engine::can_process_customer(customer, data));
    let rect = Rect::new(center.x - 74.0, center.y - 58.0, 148.0, 116.0);

    if let Some(sheet) = interior_sheet {
        // Chairs sit behind the table so an occupant reads as seated at it.
        sprites::blit_grounded(
            sheet,
            Region::ChairWood,
            center.x - 44.0,
            center.y + 18.0,
            74.0,
        );
        sprites::blit_grounded(
            sheet,
            Region::ChairWood,
            center.x + 44.0,
            center.y + 18.0,
            74.0,
        );
        sprites::blit_grounded(sheet, Region::RoundTable, center.x, center.y + 34.0, 96.0);
        if ready_for_lounge {
            draw_circle_lines(center.x, center.y, table_radius + 4.0, 2.5, SKYBLUE);
        }
    } else {
        let table_color = if occupied {
            Color::new(0.23, 0.15, 0.09, 1.0)
        } else {
            Color::new(0.14, 0.11, 0.09, 1.0)
        };
        let outline = if ready_for_lounge {
            SKYBLUE
        } else if occupied {
            Color::new(0.70, 0.50, 0.30, 1.0)
        } else {
            Color::new(0.42, 0.34, 0.25, 1.0)
        };
        draw_circle(
            center.x - 45.0,
            center.y,
            18.0,
            Color::new(0.10, 0.075, 0.06, 1.0),
        );
        draw_circle(
            center.x + 45.0,
            center.y,
            18.0,
            Color::new(0.10, 0.075, 0.06, 1.0),
        );
        draw_circle(center.x, center.y, table_radius, table_color);
        draw_circle_lines(
            center.x,
            center.y,
            table_radius,
            if ready_for_lounge { 2.5 } else { 1.5 },
            outline,
        );
        draw_circle(center.x, center.y, 14.0, Color::new(0.50, 0.27, 0.15, 1.0));
    }
    draw_ui_text(
        &format!("T{}", table_index + 1),
        center.x - 10.0,
        center.y + 5.0,
        17.0,
        GOLD,
    );

    let Some(customer) = customer else {
        draw_ui_text(
            data.text("ui_open"),
            center.x - 16.0,
            center.y + 27.0,
            13.0,
            MUTED,
        );
        return;
    };

    let mut chip_x = center.x - 26.0;
    if let Some(customer_type) = data.customer_type_by_id(&customer.customer_type) {
        for dish_color in customer_type.preferred_dishes.iter().take(3) {
            draw_circle(
                chip_x + 6.0,
                center.y - 30.0,
                5.0,
                station_draw_color(dish_color),
            );
            chip_x += 18.0;
        }
    }

    let can_serve = customer.is_seated
        && selected_station.as_ref().is_some_and(|color| {
            game.cooking_stations
                .get(color)
                .is_some_and(|station| !station.dishes.is_empty())
                && customer.next_course_for(color).is_some()
        });
    if can_serve {
        let serve_rect = Rect::new(rect.x + rect.w - 62.0, rect.y + 12.0, 50.0, 24.0);
        draw_button(serve_rect, data.text("ui_serve"), true, false);
        draw_circle_lines(center.x, center.y, table_radius + 5.0, 2.0, SKYBLUE);
        ui.serve_customer.insert(customer.id, serve_rect);
    }
}

pub(super) fn draw_dining_room(
    floor: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    now_ms: f64,
    selected_station: &Option<String>,
    textures: &HashMap<String, Texture2D>,
    interior_sheet: Option<&Texture2D>,
    ui: &mut UiActions,
) {
    if let Some(sheet) = interior_sheet {
        sprites::tile(sheet, Region::FloorWoodDark, floor, 76.0);
        // Mute the bright wood into the game's dark palette, then darken the top
        // strip into a "wall" so the floor reads with depth, not a flat repeat.
        draw_rectangle(
            floor.x,
            floor.y,
            floor.w,
            floor.h,
            Color::new(0.04, 0.03, 0.03, 0.30),
        );
        draw_rectangle(
            floor.x,
            floor.y,
            floor.w,
            96.0,
            Color::new(0.05, 0.04, 0.04, 0.45),
        );
    } else {
        room::draw_floor_pattern(floor);
    }
    super::ambience::draw_tier_tone(floor, progression, data);
    room::draw_room_fixtures(floor, data);
    if let Some(sheet) = interior_sheet {
        room::draw_room_decor(floor, sheet, progression, data);
    }
    draw_rectangle_lines(floor.x, floor.y, floor.w, floor.h, 1.5, LINE);

    let selected_text = selected_station.as_deref().map(|color| {
        let ready = game
            .cooking_stations
            .get(color)
            .map(|station| station.dishes.len())
            .unwrap_or_default();
        if ready > 0 {
            data.text_format(
                "message_serve_prompt",
                [("dish", dish_label(data, color))].as_slice(),
            )
        } else {
            dish_label(data, color)
        }
    });
    let plaque = Rect::new(
        floor.x + 18.0,
        floor.y + floor.h - 48.0,
        floor.w * 0.48,
        34.0,
    );
    draw_rectangle(
        plaque.x,
        plaque.y,
        plaque.w,
        plaque.h,
        Color::new(0.04, 0.035, 0.04, 0.86),
    );
    draw_rectangle_lines(plaque.x, plaque.y, plaque.w, plaque.h, 1.0, LINE);
    draw_ui_text(
        &data.text_format(
            "ui_serving",
            [(
                "selection",
                selected_text.unwrap_or_else(|| data.text("message_cook_carry_serve").to_string()),
            )]
            .as_slice(),
        ),
        plaque.x + 12.0,
        plaque.y + 23.0,
        15.0,
        TEXT,
    );

    let entrance_world = restaurant_entrance_position();
    let entrance = floor_to_screen(floor, entrance_world.0, entrance_world.1);
    draw_rectangle(
        entrance.x - 44.0,
        entrance.y - 28.0,
        88.0,
        56.0,
        Color::new(0.055, 0.045, 0.040, 1.0),
    );
    draw_rectangle_lines(entrance.x - 44.0, entrance.y - 28.0, 88.0, 56.0, 1.5, GOLD);
    draw_ui_text(
        data.text("ui_front_door"),
        entrance.x - 34.0,
        entrance.y + 6.0,
        15.0,
        GOLD,
    );

    room::draw_last_meal_lounge(floor, game, data, interior_sheet);

    let max_tables = max_customer_count(data, progression);
    for table_index in 0..max_tables {
        let (x, y) = restaurant_table_position(table_index, max_tables);
        let table_customer = game
            .customers
            .iter()
            .find(|customer| customer.table_index == table_index);
        draw_table(
            floor_to_screen(floor, x, y),
            table_index,
            table_customer,
            data,
            selected_station,
            game,
            interior_sheet,
            ui,
        );
    }

    if game.player.x >= 0.0 {
        draw_player_actor(
            floor_to_screen(floor, game.player.x, game.player.y),
            &game.player,
            data,
            interior_sheet,
        );
    }

    let mut customers: Vec<_> = game.customers.iter().collect();
    customers.sort_by(|left, right| {
        left.floor_y
            .partial_cmp(&right.floor_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for customer in customers {
        draw_customer_sprite(
            floor,
            customer,
            data,
            progression,
            now_ms,
            selected_station,
            textures,
            game,
            ui,
        );
    }

    if game.customers.is_empty() {
        let text = data.text("message_waiting_guests");
        let dim = measure_ui_text(text, None, 18, 1.0);
        draw_ui_text(
            text,
            floor.x + (floor.w - dim.width) * 0.5,
            floor.y + 116.0,
            18.0,
            MUTED,
        );
    }

    super::ambience::draw_ambient_chatter(floor, game, data, now_ms);
    draw_combo_meter(floor, game, progression, data);
    draw_day_clock(floor, game, data);
    draw_event_banner(floor, game, data);
}

/// The service-day clock: which day it is and how much of it is left.
fn draw_day_clock(floor: Rect, game: &GameState, data: &GameData) {
    let rect = Rect::new(floor.x + 12.0, floor.y + 12.0, 128.0, 40.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.04, 0.035, 0.04, 0.88),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE);
    draw_ui_text(
        &data.text_format(
            "ui_day",
            [("day", game.day_cycle.day.to_string())].as_slice(),
        ),
        rect.x + 10.0,
        rect.y + 17.0,
        14.0,
        GOLD,
    );
    let progress = game.day_cycle.day_progress(data.balance.day_length_ms);
    draw_rectangle(
        rect.x + 10.0,
        rect.y + 26.0,
        rect.w - 20.0,
        6.0,
        Color::new(0.16, 0.13, 0.15, 1.0),
    );
    draw_rectangle(
        rect.x + 10.0,
        rect.y + 26.0,
        (rect.w - 20.0) * (1.0 - progress),
        6.0,
        Color::new(0.90, 0.70, 0.40, 1.0),
    );
}

/// Banner for the dining event currently shaping the floor.
fn draw_event_banner(floor: Rect, game: &GameState, data: &GameData) {
    let Some(active) = &game.active_event else {
        return;
    };
    let Some(event) = data.dining_event_by_id(&active.event_id) else {
        return;
    };
    let text = data.text_format(
        "ui_event_banner",
        [
            ("name", event.name.clone()),
            (
                "seconds",
                format!("{:.0}", (active.remaining_ms / 1000.0).max(0.0)),
            ),
            ("description", event.description.clone()),
        ]
        .as_slice(),
    );
    let dim = measure_ui_text(&text, None, 14, 1.0);
    let rect = Rect::new(floor.x + 12.0, floor.y + 58.0, dim.width + 24.0, 26.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.16, 0.07, 0.05, 0.92),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.5,
        Color::new(0.94, 0.60, 0.36, 1.0),
    );
    draw_ui_text(
        &text,
        rect.x + 12.0,
        rect.y + 18.0,
        14.0,
        Color::new(0.96, 0.82, 0.66, 1.0),
    );
}

/// Visible service-streak meter: the combo multiplier the score math already
/// applies, plus progress toward the next streak cash bonus.
fn draw_combo_meter(
    floor: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
) {
    if game.combo == 0 {
        return;
    }
    let boost = progression.get_effect("combo_multiplier", 1.0);
    let multiplier = 1.0 + f64::from(game.combo) * data.balance.combo_score_step * boost;
    let rect = Rect::new(floor.x + floor.w - 178.0, floor.y + 12.0, 164.0, 44.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.04, 0.035, 0.04, 0.88),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, GOLD);
    draw_ui_text(
        &data.text_format(
            "ui_combo",
            [
                ("combo", game.combo.to_string()),
                ("multiplier", format!("{:.1}", multiplier)),
            ]
            .as_slice(),
        ),
        rect.x + 10.0,
        rect.y + 18.0,
        14.0,
        TEXT,
    );
    let interval = data.balance.combo_milestone_interval.max(2);
    let toward_next = game.combo % interval;
    draw_rectangle(
        rect.x + 10.0,
        rect.y + 28.0,
        rect.w - 20.0,
        6.0,
        Color::new(0.16, 0.13, 0.15, 1.0),
    );
    draw_rectangle(
        rect.x + 10.0,
        rect.y + 28.0,
        (rect.w - 20.0) * (toward_next as f32 / interval as f32),
        6.0,
        GOLD,
    );
}
