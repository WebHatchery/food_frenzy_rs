//! Reviewable prestige choice: inspect a perk, then explicitly confirm the reset.

use super::common::{draw_button, GOLD, LINE, MUTED, TEXT};
use super::types::{OverlayKind, UiActions};
use crate::data::GameData;
use crate::state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, wrap_text};

pub(super) fn draw_prestige_modal(game: &GameState, data: &GameData, ui: &mut UiActions) {
    if !game.pending_prestige || data.prestige_perks.is_empty() {
        return;
    }

    let width = screen_width();
    let height = screen_height();
    draw_rectangle(
        0.0,
        0.0,
        width,
        height,
        Color::new(0.01, 0.008, 0.012, 0.82),
    );
    ui.modal_open = true;
    ui.overlay = OverlayKind::Prestige;

    let panel = Rect::new(
        12.0,
        12.0,
        (width - 24.0).max(1.0),
        (height - 24.0).max(1.0),
    );
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.045, 0.038, 0.045, 0.99),
    );
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, GOLD);
    let headline = data.text("ui_prestige_question");
    let headline_dim = measure_ui_text(headline, None, 24, 1.0);
    draw_ui_text(
        headline,
        width * 0.5 - headline_dim.width * 0.5,
        panel.y + 34.0,
        24.0,
        GOLD,
    );
    draw_ui_text(
        data.text("ui_prestige_select_hint"),
        panel.x + 24.0,
        panel.y + 62.0,
        14.0,
        MUTED,
    );
    draw_ui_text(
        data.text("ui_prestige_reset_summary"),
        panel.x + 24.0,
        panel.y + 84.0,
        14.0,
        TEXT,
    );

    let page_size = if width < 900.0 { 2 } else { 3 };
    let page_count = data.prestige_perks.len().div_ceil(page_size).max(1);
    let page = game.prestige_page.min(page_count.saturating_sub(1));
    let visible = data
        .prestige_perks
        .iter()
        .skip(page * page_size)
        .take(page_size);
    let gap = 16.0;
    let card_w =
        ((panel.w - 48.0 - gap * (page_size as f32 - 1.0)) / page_size as f32).clamp(180.0, 300.0);
    let card_h = (panel.h - 192.0).clamp(150.0, 250.0);
    let total_w = card_w * page_size as f32 + gap * (page_size as f32 - 1.0);
    let start_x = width * 0.5 - total_w * 0.5;
    for (index, perk) in visible.enumerate() {
        let card = Rect::new(
            start_x + index as f32 * (card_w + gap),
            panel.y + 104.0,
            card_w,
            card_h,
        );
        let selected = game.selected_prestige_perk.as_deref() == Some(perk.id.as_str());
        draw_rectangle(
            card.x,
            card.y,
            card.w,
            card.h,
            if selected {
                Color::new(0.10, 0.075, 0.085, 0.99)
            } else {
                Color::new(0.05, 0.042, 0.050, 0.98)
            },
        );
        draw_rectangle_lines(
            card.x,
            card.y,
            card.w,
            card.h,
            if selected { 2.5 } else { 1.5 },
            if selected {
                Color::new(0.40, 0.64, 0.92, 1.0)
            } else {
                LINE
            },
        );
        draw_ui_text(&perk.name, card.x + 14.0, card.y + 30.0, 18.0, GOLD);
        let mut y = card.y + 60.0;
        for line in wrap_text(&perk.description, card.w - 28.0, 14.0) {
            draw_ui_text(&line, card.x + 14.0, y, 14.0, TEXT);
            y += 19.0;
        }
        draw_ui_text(
            if selected {
                data.text("ui_selected")
            } else {
                data.text("ui_select")
            },
            card.x + 14.0,
            card.y + card.h - 16.0,
            13.0,
            if selected {
                Color::new(0.40, 0.64, 0.92, 1.0)
            } else {
                MUTED
            },
        );
        ui.prestige_perk_buttons.insert(perk.id.clone(), card);
    }

    if page_count > 1 {
        let previous = Rect::new(panel.x + 24.0, panel.y + panel.h - 58.0, 112.0, 42.0);
        let next = Rect::new(
            panel.x + panel.w - 136.0,
            panel.y + panel.h - 58.0,
            112.0,
            42.0,
        );
        draw_button(previous, data.text("ui_previous"), page > 0, page == 0);
        draw_button(
            next,
            data.text("ui_next"),
            page + 1 < page_count,
            page + 1 >= page_count,
        );
        ui.prestige_previous = Some(previous);
        ui.prestige_next = Some(next);
        let page_label = format!("{} / {}", page + 1, page_count);
        let dim = measure_ui_text(&page_label, None, 14, 1.0);
        draw_ui_text(
            &page_label,
            width * 0.5 - dim.width * 0.5,
            panel.y + panel.h - 32.0,
            14.0,
            MUTED,
        );
    }

    let cancel = Rect::new(width * 0.5 - 150.0, panel.y + panel.h - 58.0, 112.0, 42.0);
    let confirm = Rect::new(width * 0.5 + 38.0, panel.y + panel.h - 58.0, 150.0, 42.0);
    draw_button(cancel, data.text("ui_cancel"), false, false);
    let confirm_label = game
        .selected_prestige_perk
        .as_deref()
        .and_then(|id| data.prestige_perk_by_id(id))
        .map(|perk| format!("Confirm: {}", perk.name))
        .unwrap_or_else(|| data.text("ui_confirm_prestige").to_string());
    draw_button(
        confirm,
        &confirm_label,
        game.selected_prestige_perk.is_some(),
        game.selected_prestige_perk.is_none(),
    );
    ui.prestige_cancel = Some(cancel);
    ui.prestige_confirm = Some(confirm);
}
