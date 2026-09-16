//! Progression, clientele, pantry, recipes, prestige, and event-feed panels.

use super::common::{
    can_afford_cost, draw_button, draw_card, draw_centered_section_title, draw_panel,
    draw_row_value, ellipsize, format_unlock_cost, sorted_ingredient_lines, station_draw_color,
    ACCENT, CARD, GOLD, LINE, MUTED, SUCCESS, TEXT,
};
use super::types::UiActions;
use crate::data::GameData;
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub(super) fn draw_growth_panel(
    panel: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_panel(panel);
    draw_centered_section_title(data.text("label_guests"), panel);

    let x = panel.x + 14.0;
    let w = panel.w - 28.0;
    let content_y = panel.y + 42.0;
    let content_h = panel.h - 56.0;
    let gap = 8.0;
    let guests_h = (content_h * 0.26).clamp(130.0, 188.0);
    let upgrades_h = (content_h * 0.18).clamp(96.0, 130.0);
    let pantry_h = (content_h * 0.12).clamp(64.0, 88.0);
    let recipes_h = (content_h * 0.17).clamp(82.0, 118.0);

    let guests = Rect::new(x, content_y, w, guests_h);
    let upgrades = Rect::new(x, guests.y + guests.h + gap, w, upgrades_h);
    let pantry = Rect::new(x, upgrades.y + upgrades.h + gap, w, pantry_h);
    let recipes = Rect::new(x, pantry.y + pantry.h + gap, w, recipes_h);
    let prestige = Rect::new(
        x,
        recipes.y + recipes.h + gap,
        w,
        (panel.y + panel.h - (recipes.y + recipes.h + 14.0)).max(76.0),
    );

    draw_guest_card(guests, game, progression, data, ui);
    draw_upgrade_card(upgrades, progression, data, ui);
    draw_pantry_card(pantry, game, data);
    draw_recipe_card(recipes, progression, data, ui);
    draw_prestige_card(prestige, progression, data, ui);
}

pub(super) fn draw_event_feed(rect: Rect, game: &GameState, data: &GameData) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.045, 0.040, 0.048, 0.98),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE);
    draw_ui_text(
        data.text("ui_latest_events"),
        rect.x + 16.0,
        rect.y + 26.0,
        18.0,
        GOLD,
    );

    let mut x = rect.x + 150.0;
    for message in game.messages.iter().rev().take(5) {
        let text = ellipsize(message, 44);
        let text_dim = measure_ui_text(&text, None, 15, 1.0);
        let pill_w = (text_dim.width + 34.0).min(360.0);
        if x + pill_w > rect.x + rect.w - 14.0 {
            break;
        }
        draw_rectangle(x, rect.y + 7.0, pill_w, 28.0, CARD);
        draw_rectangle_lines(x, rect.y + 7.0, pill_w, 28.0, 1.0, LINE);
        draw_circle(x + 14.0, rect.y + 21.0, 5.0, ACCENT);
        draw_ui_text(&text, x + 26.0, rect.y + 26.0, 15.0, TEXT);
        x += pill_w + 10.0;
    }
}

fn draw_guest_card(
    card: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_card(card, data.text("label_guests"));
    // Open the full clientele ladder overlay.
    let board_button = Rect::new(card.x + card.w - 72.0, card.y + 8.0, 60.0, 24.0);
    draw_button(board_button, data.text("ui_ladder"), true, false);
    if ui.clientele_board_toggle.is_none() {
        ui.clientele_board_toggle = Some(board_button);
    }
    if game.customers.is_empty() {
        draw_ui_text(
            data.text("ui_waiting_arrivals"),
            card.x + 12.0,
            card.y + 54.0,
            15.0,
            MUTED,
        );
    } else {
        let split_y = card.y + card.h - 62.0;
        let max_rows = (((split_y - card.y - 42.0) / 30.0).floor() as usize).clamp(1, 3);
        for (index, customer) in game.customers.iter().take(max_rows).enumerate() {
            let y = card.y + 50.0 + index as f32 * 30.0;
            let color = data
                .customer_type_by_id(&customer.customer_type)
                .and_then(|customer_type| customer_type.preferred_dishes.first())
                .map(|dish| station_draw_color(dish))
                .unwrap_or(GOLD);
            draw_circle(card.x + 18.0, y - 5.0, 8.0, color);
            draw_ui_text(
                &ellipsize(&customer.display_name, 18),
                card.x + 34.0,
                y,
                15.0,
                TEXT,
            );
            draw_ui_text(
                if customer.is_seated {
                    data.text("ui_guest_seated")
                } else {
                    data.text("ui_guest_arriving")
                },
                card.x + card.w - 78.0,
                y,
                13.0,
                MUTED,
            );
        }
    }

    let next_y = card.y + card.h - 58.0;
    draw_ui_text(
        data.text("ui_next_clientele"),
        card.x + 12.0,
        next_y,
        14.0,
        GOLD,
    );
    let mut locked_customer_types: Vec<_> = data
        .customer_types
        .iter()
        .filter(|customer_type| !progression.is_customer_unlocked(&customer_type.id))
        .collect();
    locked_customer_types.sort_by(|left, right| {
        left.profile_tier
            .cmp(&right.profile_tier)
            .then_with(|| left.name.cmp(&right.name))
    });

    if let Some(customer_type) = locked_customer_types.first() {
        let can_attract = can_afford_cost(game, &customer_type.unlock_cost);
        draw_ui_text(
            &ellipsize(&customer_type.name.replace(" Girl", ""), 18),
            card.x + 12.0,
            next_y + 25.0,
            16.0,
            if can_attract { TEXT } else { MUTED },
        );
        draw_ui_text(
            &ellipsize(&format_unlock_cost(data, &customer_type.unlock_cost), 22),
            card.x + 12.0,
            next_y + 43.0,
            12.0,
            MUTED,
        );
        let button_rect = Rect::new(card.x + card.w - 86.0, next_y + 13.0, 72.0, 30.0);
        draw_button(
            button_rect,
            data.text("ui_attract"),
            can_attract,
            !can_attract,
        );
        ui.attract_buttons
            .insert(customer_type.id.clone(), button_rect);
    } else {
        draw_ui_text(
            data.text("ui_all_guests_unlocked"),
            card.x + 12.0,
            next_y + 25.0,
            15.0,
            MUTED,
        );
    }
}

fn draw_upgrade_card(
    card: Rect,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_card(card, data.text("ui_upgrades"));
    let mut y = card.y + 52.0;
    let row_gap = ((card.h - 54.0) / 2.0).clamp(30.0, 48.0);
    for upgrade in progression.upgrades.iter().take(2) {
        let can_buy = progression.currency >= upgrade.cost && upgrade.level < upgrade.max_level;
        draw_ui_text(
            &data.text_format(
                "ui_upgrade_level",
                [
                    ("name", ellipsize(&upgrade.name, 18)),
                    ("level", upgrade.level.to_string()),
                    ("max", upgrade.max_level.to_string()),
                ]
                .as_slice(),
            ),
            card.x + 12.0,
            y,
            15.0,
            if can_buy { TEXT } else { MUTED },
        );
        draw_ui_text(
            &data.text_format("ui_cost", [("amount", upgrade.cost.to_string())].as_slice()),
            card.x + 12.0,
            y + 17.0,
            12.0,
            SUCCESS,
        );
        let button_rect = Rect::new(card.x + card.w - 82.0, y - 18.0, 68.0, 30.0);
        draw_button(button_rect, data.text("ui_upgrade"), can_buy, !can_buy);
        ui.upgrade_buttons.insert(upgrade.id.clone(), button_rect);
        y += row_gap;
    }
}

fn draw_pantry_card(card: Rect, game: &GameState, data: &GameData) {
    draw_card(card, data.text("ui_pantry"));
    let ingredients = sorted_ingredient_lines(game);
    if ingredients.is_empty() {
        draw_ui_text(
            data.text("ui_larder_bare"),
            card.x + 12.0,
            card.y + 62.0,
            15.0,
            MUTED,
        );
        return;
    }

    let slot_w = ((card.w - 86.0) / 4.0).max(48.0);
    let slot_h = (card.h - 48.0).clamp(24.0, 38.0);
    for (index, line) in ingredients.iter().take(4).enumerate() {
        let slot = Rect::new(
            card.x + 12.0 + index as f32 * (slot_w + 8.0),
            card.y + 36.0,
            slot_w,
            slot_h,
        );
        draw_rectangle(
            slot.x,
            slot.y,
            slot.w,
            slot.h,
            Color::new(0.09, 0.08, 0.08, 1.0),
        );
        draw_rectangle_lines(slot.x, slot.y, slot.w, slot.h, 1.0, LINE);
        draw_circle(
            slot.x + 16.0,
            slot.y + slot.h * 0.5,
            9.0,
            Color::new(0.72, 0.48, 0.34, 1.0),
        );
        draw_ui_text(
            &ellipsize(line, 9),
            slot.x + 31.0,
            slot.y + slot.h * 0.5 + 5.0,
            12.0,
            TEXT,
        );
    }
}

fn draw_recipe_card(
    card: Rect,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_card(card, data.text("ui_recipes"));
    let slots = 4;
    let gap = 8.0;
    let slot_w = (card.w - 24.0 - gap * (slots as f32 - 1.0)) / slots as f32;
    let slot_h = (card.h - 52.0).clamp(34.0, 66.0);
    for (index, recipe) in progression.recipes.iter().take(slots).enumerate() {
        let slot = Rect::new(
            card.x + 12.0 + index as f32 * (slot_w + gap),
            card.y + 40.0,
            slot_w,
            slot_h,
        );
        draw_rectangle(
            slot.x,
            slot.y,
            slot.w,
            slot.h,
            Color::new(0.08, 0.07, 0.075, 1.0),
        );
        draw_rectangle_lines(slot.x, slot.y, slot.w, slot.h, 1.0, LINE);
        draw_circle(
            slot.x + slot.w * 0.5,
            slot.y + slot.h * 0.36,
            (slot.h * 0.20).clamp(11.0, 16.0),
            if recipe.unlocked { SUCCESS } else { MUTED },
        );
        draw_ui_text(
            &ellipsize(&recipe.name, 10),
            slot.x + 6.0,
            slot.y + slot.h - 10.0,
            12.0,
            if recipe.unlocked { TEXT } else { MUTED },
        );
        ui.recipe_buttons.insert(recipe.id.clone(), slot);
    }
}

fn draw_prestige_card(
    card: Rect,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_card(card, data.text("ui_prestige"));
    let requirement = crate::engine::prestige_requirement(data, progression);
    let progress = progression.total_score as f32 / requirement.max(1) as f32;
    let compact = card.h < 108.0;
    let row_y = if compact {
        card.y + 30.0
    } else {
        card.y + 42.0
    };
    draw_row_value(
        &data.text_format(
            "ui_level",
            [("level", progression.prestige_level.to_string())].as_slice(),
        ),
        &format!("{}/{}", progression.total_score, requirement),
        Rect::new(card.x + 12.0, row_y, card.w - 24.0, 22.0),
        TEXT,
    );
    if !compact {
        draw_rectangle(
            card.x + 12.0,
            card.y + 72.0,
            card.w - 24.0,
            8.0,
            Color::new(0.16, 0.13, 0.15, 1.0),
        );
        draw_rectangle(
            card.x + 12.0,
            card.y + 72.0,
            (card.w - 24.0) * progress.clamp(0.0, 1.0),
            8.0,
            Color::new(0.74, 0.35, 0.88, 1.0),
        );
    }

    let prestige_rect = Rect::new(card.x + 12.0, card.y + card.h - 38.0, card.w - 24.0, 28.0);
    let can_prestige = progression.total_score >= requirement;
    draw_button(
        prestige_rect,
        &data.text_format(
            "ui_prestige_button",
            [("reward", progression.prestige_reward().to_string())].as_slice(),
        ),
        can_prestige,
        !can_prestige,
    );
    ui.prestige_button = Some(prestige_rect);
}
