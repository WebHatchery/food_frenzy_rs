//! Static dining-room dressing: floor, fixtures, decor, and the Last Meal
//! Lounge alcove. Tables and guests are drawn by the parent module.

use super::super::common::{draw_bar, ellipsize, station_draw_color, GOLD, LINE, MUTED, TEXT};
use super::super::sprites::{self, Region};
use crate::data::GameData;
use crate::state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw_floor_pattern(floor: Rect) {
    draw_rectangle(
        floor.x,
        floor.y,
        floor.w,
        floor.h,
        Color::new(0.080, 0.070, 0.062, 1.0),
    );
    draw_rectangle(
        floor.x,
        floor.y,
        floor.w,
        96.0,
        Color::new(0.070, 0.050, 0.040, 1.0),
    );
    let tile = 54.0;
    let mut x = floor.x;
    while x < floor.x + floor.w {
        draw_line(
            x,
            floor.y,
            x,
            floor.y + floor.h,
            1.0,
            Color::new(0.12, 0.105, 0.095, 1.0),
        );
        x += tile;
    }
    let mut y = floor.y;
    while y < floor.y + floor.h {
        draw_line(
            floor.x,
            y,
            floor.x + floor.w,
            y,
            1.0,
            Color::new(0.12, 0.105, 0.095, 1.0),
        );
        y += tile;
    }
    draw_rectangle_lines(
        floor.x + 4.0,
        floor.y + 4.0,
        floor.w - 8.0,
        floor.h - 8.0,
        1.0,
        LINE,
    );
}

pub(super) fn draw_room_fixtures(floor: Rect, data: &GameData) {
    let counter = Rect::new(floor.x + 18.0, floor.y + 18.0, floor.w * 0.34, 64.0);
    draw_rectangle(
        counter.x,
        counter.y,
        counter.w,
        counter.h,
        Color::new(0.11, 0.075, 0.045, 1.0),
    );
    draw_rectangle_lines(counter.x, counter.y, counter.w, counter.h, 1.0, GOLD);
    draw_ui_text(
        data.text("ui_kitchen"),
        counter.x + 16.0,
        counter.y + 24.0,
        16.0,
        GOLD,
    );
    let station_gap = ((counter.w - 72.0) / 3.0).clamp(36.0, 58.0);
    for index in 0..4 {
        draw_circle(
            counter.x + 36.0 + index as f32 * station_gap,
            counter.y + 45.0,
            11.0,
            station_draw_color(crate::data::STATION_COLORS[index]),
        );
    }

    let runner = Rect::new(
        floor.x + floor.w * 0.24,
        floor.y + 118.0,
        floor.w * 0.10,
        floor.h - 188.0,
    );
    draw_rectangle(
        runner.x,
        runner.y,
        runner.w,
        runner.h,
        Color::new(0.19, 0.055, 0.045, 0.88),
    );
    draw_rectangle_lines(
        runner.x,
        runner.y,
        runner.w,
        runner.h,
        1.0,
        Color::new(0.44, 0.26, 0.15, 1.0),
    );
}

/// Dress the room with interim-art decor, kept to the top wall and right edge
/// so it stays clear of tables, the door, the plaque, and the lounge.
pub(super) fn draw_room_decor(floor: Rect, sheet: &Texture2D) {
    // Garland strung across the top wall.
    sprites::blit(
        sheet,
        Region::StringLights,
        Rect::new(floor.x + 16.0, floor.y + 2.0, floor.w - 32.0, 38.0),
    );
    // House sign hung centre-top; menu board and a framed dish to the right.
    sprites::blit_grounded(
        sheet,
        Region::SignFeast,
        floor.x + floor.w * 0.52,
        floor.y + 78.0,
        50.0,
    );
    sprites::blit_grounded(
        sheet,
        Region::FramedPic,
        floor.x + floor.w * 0.70,
        floor.y + 84.0,
        58.0,
    );
    sprites::blit_grounded(
        sheet,
        Region::MenuBoard,
        floor.x + floor.w - 54.0,
        floor.y + 104.0,
        80.0,
    );
    // Potted plant softening the right edge.
    sprites::blit_grounded(
        sheet,
        Region::Plant,
        floor.x + floor.w - 46.0,
        floor.y + floor.h * 0.52,
        84.0,
    );
}

pub(super) fn draw_last_meal_lounge(
    floor: Rect,
    game: &GameState,
    data: &GameData,
    interior_sheet: Option<&Texture2D>,
) {
    let lounge = Rect::new(
        floor.x + floor.w - 250.0,
        floor.y + floor.h - 154.0,
        220.0,
        122.0,
    );
    let ready_guest = game
        .customers
        .iter()
        .find(|customer| customer.is_seated && crate::engine::can_process_customer(customer, data));
    let is_active = game.special_table_busy || ready_guest.is_some();
    draw_rectangle(
        lounge.x,
        lounge.y,
        lounge.w,
        lounge.h,
        if is_active {
            Color::new(0.12, 0.08, 0.12, 1.0)
        } else {
            Color::new(0.065, 0.052, 0.062, 1.0)
        },
    );
    draw_rectangle_lines(
        lounge.x,
        lounge.y,
        lounge.w,
        lounge.h,
        if is_active { 2.0 } else { 1.0 },
        if is_active { SKYBLUE } else { LINE },
    );
    draw_line(
        lounge.x + 20.0,
        lounge.y + 30.0,
        lounge.x + lounge.w - 20.0,
        lounge.y + 30.0,
        2.0,
        GOLD,
    );
    draw_circle(
        lounge.x + lounge.w * 0.50,
        lounge.y + 70.0,
        35.0,
        Color::new(0.04, 0.035, 0.04, 1.0),
    );
    draw_circle_lines(lounge.x + lounge.w * 0.50, lounge.y + 70.0, 36.0, 1.5, GOLD);
    if let Some(sheet) = interior_sheet {
        sprites::blit_grounded(
            sheet,
            Region::Candles,
            lounge.x + lounge.w * 0.5,
            lounge.y + 88.0,
            44.0,
        );
    }
    draw_ui_text(
        data.text("ui_lounge_title"),
        lounge.x + 14.0,
        lounge.y + 23.0,
        17.0,
        TEXT,
    );
    let status = if game.special_table_busy {
        data.text_format(
            "ui_lounge_processing",
            [(
                "seconds",
                format!("{:.0}", (game.special_table_timer / 1000.0).max(0.0)),
            )]
            .as_slice(),
        )
    } else if let Some(customer) = ready_guest {
        data.text_format(
            "ui_lounge_status_ready",
            [("name", ellipsize(&customer.display_name, 12))].as_slice(),
        )
    } else {
        data.text("ui_lounge_locked").to_string()
    };
    draw_ui_text(
        &status,
        lounge.x + 14.0,
        lounge.y + lounge.h - 39.0,
        14.0,
        if is_active { LIGHTGRAY } else { MUTED },
    );
    if game.special_table_busy {
        draw_bar(
            lounge.x + 14.0,
            lounge.y + lounge.h - 21.0,
            lounge.w - 28.0,
            6.0,
            data.balance.special_table_process_time - game.special_table_timer,
            data.balance.special_table_process_time.max(1.0),
            SKYBLUE,
        );
    } else {
        draw_ui_text(
            data.text("ui_lounge_opens"),
            lounge.x + 14.0,
            lounge.y + lounge.h - 18.0,
            12.0,
            MUTED,
        );
    }
}
