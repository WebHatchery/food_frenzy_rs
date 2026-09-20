//! Responsive playing-screen layout and top-level UI composition.

use super::common::{draw_button, draw_resource_tile, draw_tooltip, BACKGROUND, GOLD, LINE};
use super::day_summary::draw_day_summary;
use super::dining::draw_dining_room;
use super::floaters::draw_floaters;
use super::growth::draw_event_feed;
use super::kitchen::draw_kitchen;
use super::lounge::draw_processing_overlay;
use super::management::draw_management_screen;
use super::prestige_modal::draw_prestige_modal;
use super::specialization::draw_specialization_modal;
use super::tutorial_panel::draw_tutorial_panel;
use super::types::UiActions;
use super::utility::{draw_help_overlay, draw_history_overlay, draw_pause_overlay};
use crate::data::{EventEffect, GameData};
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;
use std::collections::HashMap;

fn draw_top_header(
    data: &GameData,
    game: &GameState,
    progression: &ProgressionState,
    ui: &mut UiActions,
) {
    let width = screen_width();
    let bar = Rect::new(8.0, 8.0, (width - 16.0).max(1.0), 60.0);
    draw_header_frame(bar);
    draw_header_tiles(bar, data, game, progression, ui);
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

fn draw_header_tiles(
    bar: Rect,
    data: &GameData,
    game: &GameState,
    progression: &ProgressionState,
    ui: &mut UiActions,
) {
    let vip =
        if matches!(
            game.active_event_effect(data),
            Some(EventEffect::LoungeClosed)
        ) {
            data.text("ui_lounge_closed").to_string()
        } else if game.special_table_busy {
            format!("{:.0}s", (game.special_table_timer / 1000.0).max(0.0))
        } else if game.customers.iter().any(|customer| {
            customer.is_seated && crate::engine::can_process_customer(customer, data)
        }) {
            data.text("ui_ready").to_string()
        } else {
            data.text("ui_lounge_locked").to_string()
        };
    let meat_total: i64 = game
        .ingredients
        .iter()
        .filter(|(name, amount)| {
            name.as_str() != data.balance.regular_ingredient_name && **amount > 0
        })
        .map(|(_, amount)| *amount)
        .sum();
    // The header keeps only quiet current-state anchors. Guests, prestige
    // detail, and purchase choices belong beside their decisions now.
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
            data.text("label_lounge"),
            vip,
            Color::new(0.74, 0.52, 0.92, 1.0),
            data.text("tip_lounge"),
        ),
    ];
    let gap = 8.0;
    let actions_w = if bar.w >= 980.0 { 196.0 } else { 170.0 };
    let tile_area = (bar.w - actions_w - 28.0).max(220.0);
    let tile_w = (tile_area - gap * (tiles.len() as f32 - 1.0)) / tiles.len() as f32;
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let mut hovered_tip: Option<(f32, &str)> = None;
    let mut x = bar.x + 12.0;
    for (index, (label, value, accent, tip)) in tiles.iter().enumerate() {
        let rect = Rect::new(x, bar.y + 8.0, tile_w, 44.0);
        draw_resource_tile(rect, label, value, Some(*accent));
        if index == 1 {
            let requirement = crate::engine::prestige_requirement(data, progression);
            let progress =
                (progression.total_score as f32 / requirement.max(1) as f32).clamp(0.0, 1.0);
            draw_rectangle(
                rect.x + 38.0,
                rect.y + rect.h - 7.0,
                (rect.w - 50.0).max(1.0),
                3.0,
                Color::new(0.16, 0.13, 0.15, 1.0),
            );
            draw_rectangle(
                rect.x + 38.0,
                rect.y + rect.h - 7.0,
                (rect.w - 50.0).max(1.0) * progress,
                3.0,
                Color::new(0.74, 0.35, 0.88, 1.0),
            );
        }
        if rect.contains(mouse) {
            hovered_tip = Some((rect.x + rect.w * 0.5, tip));
        }
        x += tile_w + gap;
    }
    let manage = Rect::new(bar.x + bar.w - actions_w + 8.0, bar.y + 13.0, 96.0, 34.0);
    let menu = Rect::new(bar.x + bar.w - 88.0, bar.y + 13.0, 72.0, 34.0);
    draw_button(manage, data.text("ui_manage"), true, false);
    draw_button(menu, data.text("ui_menu"), false, false);
    ui.management_button = Some(manage);
    ui.menu_button = Some(menu);
    if let Some((center_x, tip)) = hovered_tip {
        draw_tooltip(tip, center_x, bar.y + bar.h + 6.0);
    }
}

fn layout_rects(width: f32, height: f32) -> (Rect, Rect, Rect, Rect) {
    let margin = 8.0;
    let header_h = 68.0;
    let footer_h = 40.0;
    let gap = 8.0;
    let main_y = margin + header_h;
    let main_bottom = (height - footer_h - margin).max(main_y + 1.0);
    let (left, floor) = if width >= 1000.0 {
        let left_w = (width * 0.19).clamp(260.0, 320.0);
        let main_h = (main_bottom - main_y).max(1.0);
        let left = Rect::new(margin, main_y, left_w, main_h);
        let floor = Rect::new(
            left.x + left.w + gap,
            main_y,
            (width - margin - left.x - left.w - gap).max(1.0),
            main_h,
        );
        (left, floor)
    } else {
        let kitchen_h = if height >= 560.0 { 112.0 } else { 82.0 };
        let left = Rect::new(margin, main_y, (width - margin * 2.0).max(1.0), kitchen_h);
        let floor_y = left.y + left.h + gap;
        let floor = Rect::new(
            margin,
            floor_y,
            (width - margin * 2.0).max(1.0),
            (main_bottom - floor_y).max(1.0),
        );
        (left, floor)
    };
    let feed_y = (height - footer_h + 4.0).max(main_y + 1.0);
    let feed = Rect::new(
        margin,
        feed_y,
        (width - margin * 2.0).max(1.0),
        (height - feed_y).max(0.0),
    );
    (left, floor, Rect::new(0.0, 0.0, 0.0, 0.0), feed)
}

fn draw_portrait_notice(data: &GameData) {
    let width = screen_width();
    let height = screen_height();
    if width >= height || width >= 700.0 {
        return;
    }
    let panel = Rect::new(12.0, 92.0, width - 24.0, (height - 116.0).max(160.0));
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.02, 0.018, 0.022, 0.94),
    );
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, GOLD);
    let lines =
        macroquad_toolkit::ui::wrap_text(data.text("ui_rotate_landscape"), panel.w - 32.0, 18.0);
    for (index, line) in lines.iter().enumerate() {
        draw_ui_text(
            line,
            panel.x + 16.0,
            panel.y + 44.0 + index as f32 * 24.0,
            18.0,
            GOLD,
        );
    }
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
    let (left, floor, _right, feed) = layout_rects(screen_width(), screen_height());

    clear_background(BACKGROUND);
    draw_top_header(data, game, progression, &mut ui);
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
    draw_event_feed(feed, game, data, &mut ui);
    draw_floaters(floor, game);
    draw_portrait_notice(data);
    if !game.show_management && !game.show_pause_menu && !game.show_help && !game.show_history {
        draw_tutorial_panel(floor, game, data, &mut ui);
    }
    if game.show_management {
        draw_management_screen(game, progression, data, &mut ui);
        return ui;
    }
    if game.show_pause_menu {
        draw_pause_overlay(data, &mut ui);
        return ui;
    }
    if game.show_help {
        draw_help_overlay(data, &mut ui);
        return ui;
    }
    if game.show_history {
        draw_history_overlay(game, data, &mut ui);
        return ui;
    }

    draw_specialization_modal(game, progression, data, &mut ui);
    draw_prestige_modal(game, data, &mut ui);
    draw_day_summary(game, data, &mut ui);
    draw_processing_overlay(game, data);

    ui
}
