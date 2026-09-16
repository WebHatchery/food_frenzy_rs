//! Responsive playing-screen layout and top-level UI composition.

use super::clientele_board::draw_clientele_board;
use super::common::{draw_resource_tile, draw_tooltip, BACKGROUND, GOLD, LINE};
use super::day_summary::draw_day_summary;
use super::dining::draw_dining_room;
use super::floaters::draw_floaters;
use super::growth::{draw_event_feed, draw_growth_panel};
use super::kitchen::draw_kitchen;
use super::lounge::draw_processing_overlay;
use super::prestige_modal::draw_prestige_modal;
use super::specialization::draw_specialization_modal;
use super::tutorial_panel::draw_tutorial_panel;
use super::types::UiActions;
use crate::data::GameData;
use crate::engine::max_customer_count;
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;
use std::collections::HashMap;

fn draw_top_header(data: &GameData, game: &GameState, progression: &ProgressionState) {
    let width = screen_width();
    let bar = Rect::new(8.0, 8.0, width - 16.0, 70.0);
    draw_header_frame(bar);
    draw_header_tiles(bar, data, game, progression);
}

fn draw_header_frame(bar: Rect) {
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w,
        bar.h,
        Color::new(0.045, 0.038, 0.044, 1.0),
    );
    draw_rectangle_lines(bar.x, bar.y, bar.w, bar.h, 1.0, LINE);
    draw_rectangle_lines(
        bar.x + 3.0,
        bar.y + 3.0,
        bar.w - 6.0,
        bar.h - 6.0,
        1.0,
        GOLD,
    );
}

fn draw_header_tiles(bar: Rect, data: &GameData, game: &GameState, progression: &ProgressionState) {
    let vip = if game.special_table_busy {
        format!("{:.0}s", (game.special_table_timer / 1000.0).max(0.0))
    } else {
        data.text("ui_ready").to_string()
    };
    let meat_total: i64 = game
        .ingredients
        .iter()
        .filter(|(name, amount)| {
            name.as_str() != data.balance.regular_ingredient_name && **amount > 0
        })
        .map(|(_, amount)| *amount)
        .sum();
    // Each tile carries a hover explanation: the three currencies confused
    // the original playtest, so every number says what it is for.
    let tiles = [
        (
            data.text("label_cash"),
            format!("${}", progression.currency),
            Color::new(0.52, 0.84, 0.46, 1.0),
            data.text("tip_cash"),
        ),
        (
            data.text("label_renown"),
            game.score.to_string(),
            Color::new(0.44, 0.66, 0.96, 1.0),
            data.text("tip_renown"),
        ),
        (
            data.text("label_larder"),
            meat_total.to_string(),
            Color::new(0.93, 0.52, 0.60, 1.0),
            data.text("tip_larder"),
        ),
        (
            data.text("label_guests"),
            format!(
                "{}/{}",
                game.customers.len(),
                max_customer_count(data, progression)
            ),
            Color::new(0.90, 0.70, 0.40, 1.0),
            data.text("tip_guests"),
        ),
        (
            data.text("label_lounge"),
            vip,
            Color::new(0.74, 0.52, 0.92, 1.0),
            data.text("tip_lounge"),
        ),
        (
            data.text("label_prestige"),
            progression.prestige_level.to_string(),
            Color::new(0.82, 0.45, 0.88, 1.0),
            data.text("tip_prestige"),
        ),
    ];
    let gap = 8.0;
    let tile_w = (bar.w - 24.0 - gap * (tiles.len() as f32 - 1.0)) / tiles.len() as f32;
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let mut hovered_tip: Option<(f32, &str)> = None;
    let mut x = bar.x + 12.0;
    for (index, (label, value, accent, tip)) in tiles.iter().enumerate() {
        let rect = Rect::new(x, bar.y + 8.0, tile_w, 54.0);
        draw_resource_tile(rect, label, value, Some(*accent));
        if index == 1 {
            // Renown tile doubles as the visible prestige progress bar.
            let progress = (progression.total_score as f32
                / data.balance.prestige_score_requirement.max(1) as f32)
                .clamp(0.0, 1.0);
            draw_rectangle(
                rect.x + 38.0,
                rect.y + rect.h - 9.0,
                rect.w - 50.0,
                4.0,
                Color::new(0.16, 0.13, 0.15, 1.0),
            );
            draw_rectangle(
                rect.x + 38.0,
                rect.y + rect.h - 9.0,
                (rect.w - 50.0) * progress,
                4.0,
                Color::new(0.74, 0.35, 0.88, 1.0),
            );
        }
        if rect.contains(mouse) {
            hovered_tip = Some((rect.x + rect.w * 0.5, tip));
        }
        x += tile_w + gap;
    }
    if let Some((center_x, tip)) = hovered_tip {
        draw_tooltip(tip, center_x, bar.y + bar.h + 6.0);
    }
}

fn layout_rects(width: f32, height: f32) -> (Rect, Rect, Rect, Rect) {
    let margin = 8.0;
    let header_h = 78.0;
    let footer_h = 56.0;
    let gap = 8.0;
    let content_w = (width - margin * 2.0 - gap * 2.0).max(0.0);
    let side_w = if width < 972.0 {
        (content_w * 0.28).clamp(150.0, 240.0)
    } else {
        (width * 0.225).clamp(320.0, 420.0)
    };
    let left_w = side_w;
    let right_w = side_w;
    let main_y = margin + header_h;
    let main_h = (height - main_y - footer_h - margin).max(0.0);
    let left = Rect::new(margin, main_y, left_w, main_h);
    let right_x = (width - margin - right_w).max(left.x + left.w + gap);
    let right = Rect::new(right_x, main_y, right_w, main_h);
    let floor = Rect::new(
        left.x + left.w + gap,
        main_y,
        (right.x - left.x - left.w - gap * 2.0).max(1.0),
        main_h,
    );
    let feed_y = (height - footer_h + 8.0).max(main_y + main_h + 4.0);
    let feed = Rect::new(
        margin,
        feed_y,
        width - margin * 2.0,
        (height - feed_y).max(0.0),
    );
    (left, floor, right, feed)
}

pub fn draw_and_collect_hitboxes(
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    now_ms: f64,
    selected_station: &Option<String>,
    character_textures: &HashMap<String, Texture2D>,
    interior_sheet: Option<&Texture2D>,
) -> UiActions {
    let mut ui = UiActions::default();
    let (left, floor, right, feed) = layout_rects(screen_width(), screen_height());

    clear_background(BACKGROUND);
    draw_top_header(data, game, progression);
    draw_kitchen(left, game, data, selected_station, interior_sheet, &mut ui);
    draw_dining_room(
        floor,
        game,
        progression,
        data,
        now_ms,
        selected_station,
        character_textures,
        interior_sheet,
        &mut ui,
    );
    draw_growth_panel(right, game, progression, data, &mut ui);
    draw_event_feed(feed, game, data);
    draw_floaters(floor, game);
    draw_tutorial_panel(floor, game, data, &mut ui);
    // Full-screen overlays, back to front; the processing sequence owns the
    // whole screen while active.
    draw_clientele_board(game, progression, data, &mut ui);
    draw_specialization_modal(game, progression, data, &mut ui);
    draw_prestige_modal(game, data, &mut ui);
    draw_day_summary(game, data, &mut ui);
    draw_processing_overlay(game, data);

    ui
}
