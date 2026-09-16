//! Guest and chef actors with accessible status, order, and interaction cues.

use super::common::{
    customer_fallback_color, dish_label, draw_bar, draw_button, draw_tooltip, ellipsize,
    floor_to_screen, player_near_customer, station_draw_color, GOLD, LINE, TEXT,
};
use super::guest_status::{draw_guest_hover_panel, draw_guest_meters};
use super::sprites::{self, Region};
use super::types::UiActions;
use crate::data::GameData;
use crate::state::{Course, Customer, GameState, PlayerActor, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use std::collections::HashMap;

fn customer_label(customer: &Customer, data: &GameData) -> String {
    let customer_type = data
        .customer_type_by_id(&customer.customer_type)
        .map(|item| item.name.replace(" Girl", ""))
        .unwrap_or_else(|| data.text("ui_guest_type_unknown").to_string());
    format!("{} / {}", customer.display_name, customer_type)
}

/// Draw the guest's ordered courses as a row of chips (dish name + colour dot),
/// dimmed and check-marked once served so the player can read the order at a
/// glance.
fn draw_order_courses(customer: &Customer, data: &GameData, pos: Vec2) {
    if customer.order.is_empty() {
        return;
    }
    let font = 12.0;
    let chip_h = 17.0;
    let gap = 4.0;
    let mut widths = Vec::with_capacity(customer.order.len());
    let mut total_w = 0.0;
    for course in &customer.order {
        let text = course_chip_text(course, data);
        let w = measure_ui_text(&text, None, font as u16, 1.0).width + 22.0;
        widths.push((text, w));
        total_w += w;
    }
    total_w += gap * (customer.order.len() as f32 - 1.0);

    let mut x = pos.x - total_w * 0.5;
    let y = pos.y - 66.0;
    for (course, (text, w)) in customer.order.iter().zip(widths) {
        draw_rectangle(
            x,
            y,
            w,
            chip_h,
            if course.served {
                Color::new(0.05, 0.10, 0.05, 0.80)
            } else {
                Color::new(0.02, 0.02, 0.025, 0.80)
            },
        );
        draw_circle(
            x + 8.0,
            y + chip_h * 0.5,
            4.0,
            station_draw_color(&course.color),
        );
        draw_ui_text(
            &text,
            x + 16.0,
            y + 13.0,
            font,
            if course.served { LIME } else { TEXT },
        );
        x += w + gap;
    }
}

fn course_chip_text(course: &Course, data: &GameData) -> String {
    let dish = ellipsize(&dish_label(data, &course.color), 14);
    if course.served {
        format!("{} {dish}", data.text("ui_served"))
    } else {
        format!("{}: {dish}", course.label)
    }
}

pub(super) fn draw_player_actor(
    pos: Vec2,
    player: &PlayerActor,
    data: &GameData,
    sheet: Option<&Texture2D>,
) {
    draw_player_actor_scaled(pos, player, 1.0, data, sheet);
}

pub(super) fn draw_player_actor_scaled(
    pos: Vec2,
    player: &PlayerActor,
    scale: f32,
    data: &GameData,
    sheet: Option<&Texture2D>,
) {
    let shadow = 24.0 * scale;
    draw_circle(
        pos.x,
        pos.y - 3.0 * scale,
        shadow,
        Color::new(0.02, 0.02, 0.025, 0.42),
    );

    if let Some(sheet) = sheet {
        sprites::blit_grounded(sheet, Region::Chef, pos.x, pos.y, 86.0 * scale);
    } else {
        let body_w = 28.0 * scale;
        let body_h = 36.0 * scale;
        let apron_w = 20.0 * scale;
        let apron_h = 28.0 * scale;
        let head = 14.0 * scale;
        let hat_w = 36.0 * scale;
        let hat_h = 10.0 * scale;
        draw_rectangle(
            pos.x - body_w * 0.5,
            pos.y - 42.0 * scale,
            body_w,
            body_h,
            Color::new(0.78, 0.78, 0.72, 1.0),
        );
        draw_rectangle(
            pos.x - apron_w * 0.5,
            pos.y - 34.0 * scale,
            apron_w,
            apron_h,
            Color::new(0.18, 0.20, 0.23, 1.0),
        );
        draw_circle(
            pos.x,
            pos.y - 51.0 * scale,
            head,
            Color::new(0.79, 0.62, 0.48, 1.0),
        );
        draw_rectangle(
            pos.x - hat_w * 0.5,
            pos.y - 68.0 * scale,
            hat_w,
            hat_h,
            Color::new(0.94, 0.92, 0.86, 1.0),
        );
        draw_circle(
            pos.x - 8.0 * scale,
            pos.y - 67.0 * scale,
            8.0 * scale,
            Color::new(0.94, 0.92, 0.86, 1.0),
        );
        draw_circle(
            pos.x + 4.0 * scale,
            pos.y - 71.0 * scale,
            9.0 * scale,
            Color::new(0.94, 0.92, 0.86, 1.0),
        );
        draw_rectangle_lines(
            pos.x - body_w * 0.5,
            pos.y - 42.0 * scale,
            body_w,
            body_h,
            1.0,
            LINE,
        );
    }

    if let Some(station) = &player.carried_station {
        draw_circle(
            pos.x + 19.0 * scale,
            pos.y - 31.0 * scale,
            9.0 * scale,
            station_draw_color(station),
        );
        draw_circle_lines(
            pos.x + 19.0 * scale,
            pos.y - 31.0 * scale,
            9.0 * scale,
            1.5,
            WHITE,
        );
    }

    if scale >= 0.85 {
        let label = data.text("ui_you");
        let font_size = (13.0 * scale).round() as u16;
        let text_dim = measure_ui_text(label, None, font_size, 1.0);
        draw_rectangle(
            pos.x - text_dim.width * 0.5 - 7.0,
            pos.y - 91.0 * scale,
            text_dim.width + 14.0,
            20.0 * scale,
            Color::new(0.02, 0.02, 0.025, 0.72),
        );
        draw_ui_text(
            label,
            pos.x - text_dim.width * 0.5,
            pos.y - 77.0 * scale,
            font_size as f32,
            TEXT,
        );
    }

    if player.action_lock_ms > 0.0 {
        draw_tooltip(data.text("ui_chef_finishing"), pos.x, pos.y - 106.0 * scale);
        draw_bar(
            pos.x - 26.0 * scale,
            pos.y - 13.0 * scale,
            52.0 * scale,
            5.0 * scale,
            player.action_lock_ms,
            900.0,
            ORANGE,
        );
    }
}

pub(super) fn draw_customer_sprite(
    floor: Rect,
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
    selected_station: &Option<String>,
    textures: &HashMap<String, Texture2D>,
    game: &GameState,
    ui: &mut UiActions,
) {
    let pos = floor_to_screen(floor, customer.floor_x, customer.floor_y);
    let sprite_rect = Rect::new(pos.x - 34.0, pos.y - 82.0, 68.0, 80.0);
    let can_serve = customer.is_seated
        && selected_station.as_ref().is_some_and(|color| {
            game.cooking_stations
                .get(color)
                .is_some_and(|station| !station.dishes.is_empty())
                && customer.next_course_for(color).is_some()
        });

    draw_circle(
        pos.x,
        pos.y - 2.0,
        28.0,
        Color::new(0.02, 0.02, 0.025, 0.38),
    );
    if let Some(texture) = textures.get(&customer.customer_type) {
        draw_texture_ex(
            texture,
            sprite_rect.x,
            sprite_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(sprite_rect.w, sprite_rect.h)),
                ..Default::default()
            },
        );
    } else {
        draw_circle(
            pos.x,
            pos.y - 43.0,
            26.0,
            customer_fallback_color(&customer.customer_type),
        );
    }

    let label = customer_label(customer, data);
    let text_dim = measure_ui_text(&label, None, 16, 1.0);
    let label_w = text_dim.width + 22.0;
    draw_rectangle(
        pos.x - label_w * 0.5,
        pos.y - 116.0,
        label_w,
        38.0,
        Color::new(0.02, 0.02, 0.025, 0.78),
    );
    draw_ui_text(
        &label,
        pos.x - label_w * 0.5 + 11.0,
        pos.y - 95.0,
        16.0,
        TEXT,
    );
    if crate::engine::is_regular(customer, data) {
        // Regulars get a little gold badge: the house knows this face.
        let badge = vec2(pos.x - label_w * 0.5 - 8.0, pos.y - 103.0);
        draw_circle(badge.x, badge.y, 7.0, Color::new(0.84, 0.60, 0.31, 1.0));
        draw_ui_text(
            data.text("ui_regular_badge"),
            badge.x - 4.0,
            badge.y + 5.0,
            12.0,
            Color::new(0.1, 0.06, 0.03, 1.0),
        );
    }
    draw_guest_meters(pos, customer, data, progression, now_ms);
    if customer.bill > 0 {
        let tab = format!("${}", customer.bill);
        let tab_dim = measure_ui_text(&tab, None, 14, 1.0);
        draw_rectangle(
            pos.x + 54.0,
            pos.y - 78.0,
            tab_dim.width + 12.0,
            18.0,
            Color::new(0.02, 0.02, 0.025, 0.78),
        );
        draw_ui_text(&tab, pos.x + 60.0, pos.y - 65.0, 14.0, GOLD);
    }
    if customer.depart_timer_ms > 0.0 {
        draw_tooltip(data.text("ui_paying"), pos.x, pos.y - 128.0);
    }
    if customer.is_seated {
        draw_order_courses(customer, data, pos);
    }
    let info_rect = Rect::new(pos.x + label_w * 0.5 - 26.0, pos.y - 111.0, 22.0, 22.0);
    draw_button(info_rect, data.text("ui_info"), true, false);
    ui.guest_info.insert(customer.id, info_rect);
    draw_guest_hover_panel(
        pos,
        sprite_rect,
        customer,
        data,
        progression,
        now_ms,
        game.selected_guest_id == Some(customer.id),
    );

    if can_serve {
        draw_rectangle_lines(
            sprite_rect.x,
            sprite_rect.y,
            sprite_rect.w,
            sprite_rect.h,
            2.0,
            SKYBLUE,
        );
        if player_near_customer(game, customer, data.balance.player_interaction_range) {
            draw_tooltip(data.text("ui_click_serve"), pos.x, pos.y - 145.0);
        }
        ui.serve_customer.entry(customer.id).or_insert(sprite_rect);
    }

    if customer.is_seated && crate::engine::can_process_customer(customer, data) {
        let invite_rect = Rect::new(pos.x - 36.0, pos.y + 22.0, 72.0, 28.0);
        draw_button(invite_rect, data.text("ui_vip"), true, false);
        ui.invite_customer.insert(customer.id, invite_rect);
    }
}
