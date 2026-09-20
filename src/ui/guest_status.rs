//! Per-guest state readout: satisfaction, patience, and fattening progress
//! meters, the "plump and ready" callout, and a hover panel that explains
//! what the numbers mean. You can't play around state you can't see.

use super::common::{
    dish_label, draw_bar, ellipsize, patience_color, patience_remaining_ratio, GOLD, LINE, TEXT,
};
use crate::data::GameData;
use crate::engine::visits_until_ready_for;
use crate::state::{Customer, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, wrap_text};

pub(super) fn draw_guest_meters(
    pos: Vec2,
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
    scale: f32,
) {
    let scale = scale.clamp(0.6, 1.0);
    // Satisfaction: how well fed they are this sitting.
    draw_bar(
        pos.x - 52.0 * scale,
        pos.y - 72.0 * scale,
        104.0 * scale,
        6.0 * scale,
        customer.total_satisfaction,
        customer.max_satisfaction.total(),
        LIME,
    );

    if !customer.is_seated {
        return;
    }

    // Patience: how long before they storm out.
    let patience = patience_remaining_ratio(customer, data, progression, now_ms);
    draw_bar(
        pos.x - 52.0 * scale,
        pos.y + 8.0 * scale,
        104.0 * scale,
        5.0 * scale,
        patience,
        1.0,
        patience_color(patience),
    );

    draw_trait_alert(pos, customer, data, scale);
}

/// Telegraphed trait warning: what they're about to do and how long the
/// player has to answer.
fn draw_trait_alert(pos: Vec2, customer: &Customer, data: &GameData, scale: f32) {
    let Some(alert) = &customer.trait_alert else {
        return;
    };
    let Some(behavior) = data.trait_behavior(&alert.trait_key) else {
        return;
    };
    let window = data.balance.trait_telegraph_ms.max(500.0);
    let remaining = (alert.remaining_ms / window).clamp(0.0, 1.0);
    let text = format!("! {}", behavior.telegraph);
    let font = (14.0 * scale).max(11.0);
    let dim = measure_ui_text(&text, None, font as u16, 1.0);
    let rect = Rect::new(
        pos.x - dim.width * 0.5 - 8.0 * scale,
        pos.y - 152.0 * scale,
        dim.width + 16.0,
        24.0 * scale,
    );
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.18, 0.05, 0.05, 0.92),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.5,
        Color::new(0.94, 0.42, 0.36, 1.0),
    );
    draw_ui_text(
        &text,
        rect.x + 8.0 * scale,
        rect.y + 17.0 * scale,
        font,
        Color::new(0.96, 0.72, 0.60, 1.0),
    );
    draw_bar(
        rect.x,
        rect.y + rect.h + 2.0 * scale,
        rect.w,
        3.0 * scale,
        remaining,
        1.0,
        Color::new(0.94, 0.42, 0.36, 1.0),
    );
}

/// Detailed hover readout so every meter has words attached to it.
pub(super) fn draw_guest_hover_panel(
    pos: Vec2,
    hover_rect: Rect,
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
    pinned: bool,
) {
    let mouse = vec2(mouse_position().0, mouse_position().1);
    if !pinned && !hover_rect.contains(mouse) {
        return;
    }

    let customer_type = data.customer_type_by_id(&customer.customer_type);
    let type_line = customer_type
        .map(|item| {
            data.text_format(
                "ui_tier_name",
                [
                    ("name", item.name.clone()),
                    ("tier", item.profile_tier.max(1).to_string()),
                ]
                .as_slice(),
            )
        })
        .unwrap_or_else(|| data.text("ui_guest_type_unknown").to_string());
    let needed = visits_until_ready_for(data, &customer.customer_type);
    let fattening_line = if customer.times_fed >= needed {
        data.text("ui_plump_ready_sentence").to_string()
    } else {
        data.text_format(
            "ui_fattening",
            [
                ("fed", customer.times_fed.to_string()),
                ("needed", needed.to_string()),
                ("remaining", (needed - customer.times_fed).to_string()),
            ]
            .as_slice(),
        )
    };
    let patience = patience_remaining_ratio(customer, data, progression, now_ms);
    let mut lines = vec![
        format!("{} - {}", ellipsize(&customer.display_name, 26), type_line),
        data.text_format(
            "ui_satisfaction",
            [
                ("current", format!("{:.0}", customer.total_satisfaction)),
                ("max", format!("{:.0}", customer.max_satisfaction.total())),
            ]
            .as_slice(),
        ),
        data.text_format(
            "ui_patience",
            [("percent", format!("{:.0}", patience * 100.0))].as_slice(),
        ),
        fattening_line,
        data.text_format(
            "ui_tab",
            [("cash", customer.bill.max(0).to_string())].as_slice(),
        ),
    ];
    if !customer.order.is_empty() {
        let order = customer
            .order
            .iter()
            .map(|course| {
                let state = if course.served { "OK" } else { "next" };
                format!(
                    "{}: {} ({})",
                    course.label,
                    dish_label(data, &course.color),
                    state
                )
            })
            .collect::<Vec<_>>()
            .join("  ");
        lines.push(format!("{} {}", data.text("ui_order"), order));
    }
    if !customer.order_complete() && customer.courses_served() > 0 {
        lines.push(if customer.eating_ms > 0.0 {
            data.text_format(
                "ui_eating_course",
                [("seconds", format!("{:.0}", customer.eating_ms / 1000.0))].as_slice(),
            )
        } else if customer.waiting_ms <= data.balance.course_wait_grace_ms {
            data.text_format(
                "ui_next_course",
                [(
                    "seconds",
                    format!(
                        "{:.0}",
                        (data.balance.course_wait_grace_ms - customer.waiting_ms) / 1000.0
                    ),
                )]
                .as_slice(),
            )
        } else {
            data.text("ui_waiting_course").to_string()
        });
    }

    let font = 14.0;
    let width = (screen_width() - 16.0).clamp(220.0, 360.0);
    let wrapped_lines = lines
        .iter()
        .flat_map(|line| wrap_text(line, width - 24.0, font))
        .collect::<Vec<_>>();
    let height = wrapped_lines.len() as f32 * 19.0 + 16.0;
    let panel = Rect::new(
        (pos.x + 64.0).clamp(8.0, screen_width() - width - 8.0),
        (pos.y - height - 16.0).clamp(8.0, screen_height() - height - 8.0),
        width,
        height,
    );
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.02, 0.02, 0.025, 0.94),
    );
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, LINE);
    for (index, line) in wrapped_lines.iter().enumerate() {
        let color = if index == 0 { GOLD } else { TEXT };
        draw_ui_text(
            line,
            panel.x + 12.0,
            panel.y + 22.0 + index as f32 * 19.0,
            font,
            color,
        );
    }
}
