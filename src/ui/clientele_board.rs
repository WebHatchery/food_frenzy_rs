//! The clientele goal board: every customer type on the 4-tier ladder with
//! its unlock cost, meat yield, and trait identity — the aspirational "menu
//! of guests" that gives the meat economy a visible long-term shape.

use super::common::{
    can_afford_cost, draw_button, format_unlock_cost, station_draw_color, GOLD, LINE, MUTED,
    SUCCESS, TEXT,
};
use super::types::UiActions;
use crate::data::{CustomerType, GameData};
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const ROW_H: f32 = 44.0;

pub(super) fn draw_clientele_board(
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    if !game.show_clientele_board {
        return;
    }

    let width = screen_width();
    let height = screen_height();
    draw_rectangle(
        0.0,
        0.0,
        width,
        height,
        Color::new(0.01, 0.008, 0.012, 0.78),
    );

    let mut types: Vec<&CustomerType> = data.customer_types.iter().collect();
    types.sort_by(|left, right| {
        left.profile_tier
            .cmp(&right.profile_tier)
            .then_with(|| left.name.cmp(&right.name))
    });

    let board_w = (width - 24.0).clamp(280.0, 860.0);
    let board_h = (types.len() as f32 * ROW_H + 96.0).min((height - 24.0).max(160.0));
    let board = Rect::new(
        width * 0.5 - board_w * 0.5,
        height * 0.5 - board_h * 0.5,
        board_w,
        board_h,
    );
    draw_rectangle(
        board.x,
        board.y,
        board.w,
        board.h,
        Color::new(0.045, 0.038, 0.045, 0.99),
    );
    draw_rectangle_lines(board.x, board.y, board.w, board.h, 2.0, GOLD);
    draw_ui_text(
        "CLIENTELE LADDER",
        board.x + 18.0,
        board.y + 32.0,
        22.0,
        GOLD,
    );
    draw_ui_text(
        "Every guest the house can attract. Meat from each tier unlocks the next.",
        board.x + 18.0,
        board.y + 54.0,
        14.0,
        MUTED,
    );

    let close = Rect::new(board.x + board.w - 76.0, board.y + 14.0, 60.0, 26.0);
    draw_button(close, "Close", true, false);
    ui.clientele_board_toggle = Some(close);
    ui.modal_open = true;
    // The board owns the attract buttons while it is open.
    ui.attract_buttons.clear();

    let visible_rows = ((board.h - 96.0) / ROW_H).floor() as usize;
    let mut y = board.y + 72.0;
    let mut last_tier = 0;
    for customer_type in types.into_iter().take(visible_rows) {
        if customer_type.profile_tier != last_tier {
            last_tier = customer_type.profile_tier;
            draw_ui_text(
                &format!("TIER {}", last_tier.max(1)),
                board.x + 18.0,
                y + 12.0,
                13.0,
                GOLD,
            );
        }
        draw_clientele_row(
            Rect::new(board.x + 78.0, y, board.w - 96.0, ROW_H - 6.0),
            customer_type,
            game,
            progression,
            data,
            ui,
        );
        y += ROW_H;
    }
}

fn draw_clientele_row(
    row: Rect,
    customer_type: &CustomerType,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    let unlocked = progression.is_customer_unlocked(&customer_type.id);
    draw_rectangle(
        row.x,
        row.y,
        row.w,
        row.h,
        if unlocked {
            Color::new(0.07, 0.075, 0.062, 0.98)
        } else {
            Color::new(0.055, 0.048, 0.055, 0.98)
        },
    );
    draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, LINE);

    // Preferred-dish identity dots double as the "silhouette" color cue.
    let mut dot_x = row.x + 12.0;
    for dish_color in customer_type.preferred_dishes.iter().take(2) {
        draw_circle(
            dot_x,
            row.y + row.h * 0.5,
            6.0,
            if unlocked {
                station_draw_color(dish_color)
            } else {
                Color::new(0.25, 0.22, 0.22, 1.0)
            },
        );
        dot_x += 16.0;
    }

    let name = if unlocked {
        customer_type.name.clone()
    } else {
        format!("{} (locked)", customer_type.name.replace(" Girl", ""))
    };
    draw_ui_text(
        &name,
        row.x + 48.0,
        row.y + 17.0,
        15.0,
        if unlocked { TEXT } else { MUTED },
    );

    let identity = trait_identity_line(customer_type, data);
    draw_ui_text(&identity, row.x + 48.0, row.y + 33.0, 12.0, MUTED);

    let yield_text = format!("yields {}-meat", customer_type.id);
    let yield_dim = measure_ui_text(&yield_text, None, 12, 1.0);
    draw_ui_text(
        &yield_text,
        row.x + row.w - yield_dim.width - 200.0,
        row.y + 17.0,
        12.0,
        Color::new(0.93, 0.52, 0.60, 1.0),
    );

    if unlocked {
        draw_ui_text(
            "on the floor",
            row.x + row.w - 92.0,
            row.y + row.h * 0.5 + 5.0,
            13.0,
            SUCCESS,
        );
    } else {
        let cost_text = format_unlock_cost(&customer_type.unlock_cost);
        let cost_dim = measure_ui_text(&cost_text, None, 12, 1.0);
        draw_ui_text(
            &cost_text,
            row.x + row.w - cost_dim.width - 200.0,
            row.y + 33.0,
            12.0,
            MUTED,
        );
        let can_attract = can_afford_cost(game, &customer_type.unlock_cost);
        let button = Rect::new(row.x + row.w - 92.0, row.y + row.h * 0.5 - 13.0, 80.0, 26.0);
        draw_button(button, "Attract", can_attract, !can_attract);
        if can_attract {
            ui.attract_buttons.insert(customer_type.id.clone(), button);
        }
    }
}

fn trait_identity_line(customer_type: &CustomerType, data: &GameData) -> String {
    let Some(traits) = &customer_type.special_traits else {
        return "easygoing".to_string();
    };
    let flags = [
        ("low_appetite", traits.low_appetite),
        ("can_wander", traits.can_wander),
        ("multiplies_on_process", traits.multiplies_on_process),
        ("fast_spoilage", traits.fast_spoilage),
        ("can_steal_food", traits.can_steal_food),
        ("can_eat_waste", traits.can_eat_waste),
        ("high_yield", traits.high_yield),
        ("throws_food", traits.throws_food),
    ];
    let names: Vec<String> = flags
        .iter()
        .filter(|(_, active)| *active)
        .filter_map(|(key, _)| {
            data.trait_behavior(key)
                .map(|behavior| behavior.name.clone())
        })
        .collect();
    if names.is_empty() {
        "easygoing".to_string()
    } else {
        names.join(" · ")
    }
}
