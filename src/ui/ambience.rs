//! Ambient dining-room life: idle chatter bubbles over seated guests and the
//! room's tone darkening as the clientele ladder climbs. Draw-only — timing
//! derives from the wall clock and guest ids, so no state is carried.

use super::common::{floor_to_screen, LINE};
use crate::data::GameData;
use crate::state::{Customer, GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const CHATTER_CYCLE_S: f64 = 19.0;
const CHATTER_SHOW_S: f64 = 3.8;

const BUBBLE_BG: Color = Color::new(0.93, 0.90, 0.82, 1.0);
const BUBBLE_TEXT: Color = Color::new(0.12, 0.09, 0.08, 1.0);
const TONE_TINT: Color = Color::new(0.10, 0.01, 0.03, 1.0);
const TONE_GLOW: Color = Color::new(0.55, 0.10, 0.10, 1.0);

pub(super) fn draw_ambient_chatter(floor: Rect, game: &GameState, data: &GameData, now_ms: f64) {
    for customer in &game.customers {
        // Warnings own the space above the guest; chatter yields to them.
        if !customer.is_seated || customer.trait_alert.is_some() {
            continue;
        }
        let clock_s = now_ms / 1000.0 + f64::from(customer.id) * 7.31;
        let phase = clock_s % CHATTER_CYCLE_S;
        if phase >= CHATTER_SHOW_S {
            continue;
        }
        let cycle_index = (clock_s / CHATTER_CYCLE_S) as usize;
        let Some(line) = chatter_line(customer, data, cycle_index) else {
            continue;
        };

        let pos = floor_to_screen(floor, customer.floor_x, customer.floor_y);
        let fade = ((CHATTER_SHOW_S - phase) / 0.6).min(phase / 0.4).min(1.0) as f32;
        let dim = measure_ui_text(line, None, 13, 1.0);
        let bubble = Rect::new(pos.x + 22.0, pos.y - 148.0, dim.width + 18.0, 24.0);
        draw_rectangle(
            bubble.x,
            bubble.y,
            bubble.w,
            bubble.h,
            with_alpha(BUBBLE_BG, 0.88 * fade),
        );
        draw_rectangle_lines(
            bubble.x,
            bubble.y,
            bubble.w,
            bubble.h,
            1.0,
            with_alpha(LINE, fade),
        );
        // Little tail pointing at the speaker.
        draw_triangle(
            vec2(bubble.x + 4.0, bubble.y + bubble.h),
            vec2(bubble.x + 14.0, bubble.y + bubble.h),
            vec2(bubble.x - 2.0, bubble.y + bubble.h + 8.0),
            with_alpha(BUBBLE_BG, 0.88 * fade),
        );
        draw_ui_text(
            line,
            bubble.x + 9.0,
            bubble.y + 17.0,
            13.0,
            with_alpha(BUBBLE_TEXT, fade),
        );
    }
}

fn chatter_line<'data>(
    customer: &Customer,
    data: &'data GameData,
    cycle_index: usize,
) -> Option<&'data str> {
    let personality_lines = customer
        .personality
        .as_deref()
        .and_then(|personality| data.personality_by_id(personality))
        .map(|personality| personality.chatter.as_slice())
        .filter(|lines| !lines.is_empty());
    match personality_lines {
        Some(lines) => Some(lines[(customer.id as usize + cycle_index) % lines.len()].as_str()),
        None => data
            .ui_text
            .fallback_chatter_line(customer.id as usize + cycle_index),
    }
}

/// The higher the clientele ladder climbs, the less cozy the room reads: a
/// creeping red-dark cast, deliberate and just shy of subliminal.
pub(super) fn draw_tier_tone(floor: Rect, progression: &ProgressionState, data: &GameData) {
    let tone_tier = data
        .customer_types
        .iter()
        .filter(|customer_type| progression.is_customer_unlocked(&customer_type.id))
        .map(|customer_type| customer_type.profile_tier)
        .max()
        .unwrap_or(1)
        .max(1);
    if tone_tier <= 1 {
        return;
    }
    let depth = ((tone_tier - 1) as f32 * 0.045).min(0.16);
    draw_rectangle(
        floor.x,
        floor.y,
        floor.w,
        floor.h,
        with_alpha(TONE_TINT, depth),
    );
    // A thin blood-warm glow line along the top wall, brighter each tier.
    draw_rectangle(
        floor.x,
        floor.y,
        floor.w,
        3.0,
        with_alpha(TONE_GLOW, depth * 2.2),
    );
}
