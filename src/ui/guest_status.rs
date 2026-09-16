//! Per-guest state readout: satisfaction, patience, and fattening progress
//! meters, the "plump and ready" callout, and a hover panel that explains
//! what the numbers mean. You can't play around state you can't see.

use super::common::{draw_bar, patience_color, patience_remaining_ratio, GOLD, LINE, MUTED, TEXT};
use crate::data::GameData;
use crate::engine::visits_until_ready_for;
use crate::state::{Customer, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const READY_GOLD: Color = Color::new(0.95, 0.78, 0.35, 1.0);
const PLUMP_PINK: Color = Color::new(0.92, 0.55, 0.62, 1.0);

pub(super) fn draw_guest_meters(
    pos: Vec2,
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
) {
    // Satisfaction: how well fed they are this sitting.
    draw_bar(
        pos.x - 52.0,
        pos.y - 72.0,
        104.0,
        6.0,
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
        pos.x - 52.0,
        pos.y + 8.0,
        104.0,
        5.0,
        patience,
        1.0,
        patience_color(patience),
    );

    draw_fattening_pips(pos, customer, data);
    draw_course_pacing_bar(pos, customer, data);
    draw_trait_alert(pos, customer, data);
}

/// Meal-rhythm strip above the satisfaction bar: fills sky-blue while the
/// guest eats (don't serve yet), drains green while they'd welcome the next
/// course, and pulses red once they've been kept waiting too long.
fn draw_course_pacing_bar(pos: Vec2, customer: &Customer, data: &GameData) {
    if customer.order_complete() || customer.courses_served() == 0 {
        return;
    }
    let (fraction, color) = if customer.eating_ms > 0.0 {
        (
            (customer.eating_ms / data.balance.course_eating_ms.max(1.0)).clamp(0.0, 1.0),
            SKYBLUE,
        )
    } else {
        let grace = data.balance.course_wait_grace_ms.max(1.0);
        if customer.waiting_ms <= grace {
            (1.0 - (customer.waiting_ms / grace).clamp(0.0, 1.0), LIME)
        } else {
            let pulse = ((macroquad::time::get_time() * 5.0).sin() * 0.5 + 0.5) as f32;
            (1.0, Color::new(0.94, 0.30 * pulse, 0.25 * pulse, 1.0))
        }
    };
    draw_bar(pos.x - 52.0, pos.y - 78.0, 104.0, 3.0, fraction, 1.0, color);
}

/// Telegraphed trait warning: what they're about to do and how long the
/// player has to answer.
fn draw_trait_alert(pos: Vec2, customer: &Customer, data: &GameData) {
    let Some(alert) = &customer.trait_alert else {
        return;
    };
    let Some(behavior) = data.trait_behavior(&alert.trait_key) else {
        return;
    };
    let window = data.balance.trait_telegraph_ms.max(500.0);
    let remaining = (alert.remaining_ms / window).clamp(0.0, 1.0);
    let text = format!("! {}", behavior.telegraph);
    let dim = measure_ui_text(&text, None, 14, 1.0);
    let rect = Rect::new(
        pos.x - dim.width * 0.5 - 8.0,
        pos.y - 152.0,
        dim.width + 16.0,
        24.0,
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
        rect.x + 8.0,
        rect.y + 17.0,
        14.0,
        Color::new(0.96, 0.72, 0.60, 1.0),
    );
    draw_bar(
        rect.x,
        rect.y + rect.h + 2.0,
        rect.w,
        3.0,
        remaining,
        1.0,
        Color::new(0.94, 0.42, 0.36, 1.0),
    );
}

/// One pip per satisfied visit toward Lounge readiness. The meter the whole
/// meta-loop hangs on, so it lives directly under every seated guest.
fn draw_fattening_pips(pos: Vec2, customer: &Customer, data: &GameData) {
    let needed = visits_until_ready_for(data, &customer.customer_type).max(1);
    let fed = customer.times_fed.min(needed);
    let ready = customer.times_fed >= needed;

    let spacing = 13.0;
    let total_w = spacing * (needed.saturating_sub(1)) as f32;
    let start_x = pos.x - total_w * 0.5;
    let y = pos.y + 21.0;
    for index in 0..needed {
        let filled = index < fed;
        let center = vec2(start_x + spacing * index as f32, y);
        if filled {
            draw_circle(center.x, center.y, 4.5, PLUMP_PINK);
        } else {
            draw_circle_lines(center.x, center.y, 4.5, 1.2, MUTED);
        }
    }

    if ready {
        let pulse = ((macroquad::time::get_time() * 4.0).sin() * 0.5 + 0.5) as f32;
        let label = data.text("ui_plump_ready");
        let dim = measure_ui_text(label, None, 13, 1.0);
        let rect = Rect::new(
            pos.x - dim.width * 0.5 - 8.0,
            y + 8.0,
            dim.width + 16.0,
            20.0,
        );
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.16, 0.06, 0.10, 0.90),
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.5,
            Color::new(
                READY_GOLD.r,
                READY_GOLD.g,
                READY_GOLD.b,
                0.55 + 0.45 * pulse,
            ),
        );
        draw_ui_text(label, rect.x + 8.0, rect.y + 15.0, 13.0, READY_GOLD);
    }
}

/// Detailed hover readout so every meter has words attached to it.
pub(super) fn draw_guest_hover_panel(
    pos: Vec2,
    hover_rect: Rect,
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
) {
    let mouse = vec2(mouse_position().0, mouse_position().1);
    if !hover_rect.contains(mouse) {
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
        format!("{} - {}", customer.display_name, type_line),
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
    let width = lines
        .iter()
        .map(|line| measure_ui_text(line, None, font as u16, 1.0).width)
        .fold(0.0_f32, f32::max)
        + 24.0;
    let height = lines.len() as f32 * 19.0 + 16.0;
    let panel = Rect::new(
        (pos.x + 64.0).min(screen_width() - width - 8.0),
        (pos.y - 120.0).max(8.0),
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
    for (index, line) in lines.iter().enumerate() {
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
