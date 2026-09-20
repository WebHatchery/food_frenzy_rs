//! On-demand management destination for the house's choices and catalogs.

use super::common::{
    can_afford_cost, draw_button, format_unlock_cost, GOLD, LINE, MUTED, SUCCESS, TEXT,
};
use super::types::{OverlayKind, UiActions};
use crate::data::{CustomerType, GameData};
use crate::state::{GameState, ProgressionState, INFINITE_INGREDIENTS};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, wrap_text};

const TAB_COUNT: usize = 4;

pub(super) fn draw_management_screen(
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    let width = screen_width();
    let height = screen_height();
    draw_rectangle(
        0.0,
        0.0,
        width,
        height,
        Color::new(0.01, 0.008, 0.012, 0.86),
    );
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
    ui.modal_open = true;
    ui.overlay = OverlayKind::Management;

    draw_ui_text(
        data.text("ui_management_title"),
        panel.x + 20.0,
        panel.y + 32.0,
        25.0,
        GOLD,
    );
    draw_ui_text(
        &data.text_format(
            "ui_management_summary",
            [
                ("cash", progression.currency.to_string()),
                ("larder", larder_total(game, data).to_string()),
            ]
            .as_slice(),
        ),
        panel.x + 20.0,
        panel.y + 54.0,
        14.0,
        MUTED,
    );
    let close = Rect::new(panel.x + panel.w - 92.0, panel.y + 16.0, 72.0, 42.0);
    draw_button(close, data.text("ui_back_to_service"), true, false);
    ui.management_close = Some(close);

    let tabs_y = panel.y + 72.0;
    let tab_w = ((panel.w - 40.0) / TAB_COUNT as f32 - 8.0).max(72.0);
    for tab in 0..TAB_COUNT {
        let rect = Rect::new(
            panel.x + 20.0 + tab as f32 * (tab_w + 8.0),
            tabs_y,
            tab_w,
            42.0,
        );
        draw_button(
            rect,
            data.text(tab_label_key(tab)),
            game.management_tab == tab as u8,
            false,
        );
        ui.management_tabs.push((tab as u8, rect));
    }

    let content = Rect::new(
        panel.x + 20.0,
        tabs_y + 54.0,
        (panel.w - 40.0).max(1.0),
        (panel.h - 192.0).max(1.0),
    );
    if game.management_tab == 2 && game.selected_recipe_id.is_some() {
        draw_recipe_detail(content, game, progression, data, ui);
    } else {
        match game.management_tab as usize {
            0 => draw_clientele_catalog(content, game, progression, data, ui),
            1 => draw_upgrade_catalog(content, game, progression, data, ui),
            2 => draw_recipe_catalog(content, game, progression, data, ui),
            _ => draw_prestige_catalog(content, progression, data, ui),
        }
        draw_pager(
            content,
            game.management_page,
            catalog_page_count(game, progression, data),
            data,
            ui,
        );
    }
}

fn tab_label_key(tab: usize) -> &'static str {
    match tab {
        0 => "ui_management_clientele",
        1 => "ui_management_upgrades",
        2 => "ui_management_recipes",
        _ => "ui_management_prestige",
    }
}

fn page_size() -> usize {
    if screen_height() < 520.0 {
        3
    } else {
        6
    }
}

fn catalog_len(game: &GameState, progression: &ProgressionState, data: &GameData) -> usize {
    match game.management_tab as usize {
        0 => data.customer_types.len(),
        1 => progression.upgrades.len(),
        2 => progression.recipes.len(),
        _ => data.prestige_perks.len(),
    }
}

fn catalog_page_count(game: &GameState, progression: &ProgressionState, data: &GameData) -> usize {
    catalog_len(game, progression, data)
        .div_ceil(page_size())
        .max(1)
}

fn draw_pager(content: Rect, page: usize, page_count: usize, data: &GameData, ui: &mut UiActions) {
    let y = content.y + content.h + 16.0;
    let previous = Rect::new(content.x, y, 112.0, 42.0);
    let next = Rect::new(content.x + content.w - 112.0, y, 112.0, 42.0);
    draw_button(previous, data.text("ui_previous"), page > 0, page == 0);
    draw_button(
        next,
        data.text("ui_next"),
        page + 1 < page_count,
        page + 1 >= page_count,
    );
    ui.management_previous = Some(previous);
    ui.management_next = Some(next);
    let label = format!("{} / {}", page + 1, page_count);
    let dim = measure_ui_text(&label, None, 16, 1.0);
    draw_ui_text(
        &label,
        content.x + (content.w - dim.width) * 0.5,
        y + 27.0,
        16.0,
        MUTED,
    );
}

fn row_rect(content: Rect, index: usize) -> Rect {
    let rows = page_size() as f32;
    let row_h = ((content.h - 8.0 * (rows - 1.0)) / rows).clamp(48.0, 78.0);
    Rect::new(
        content.x,
        content.y + index as f32 * (row_h + 8.0),
        content.w,
        row_h,
    )
}

fn visible_range(page: usize, len: usize) -> std::ops::Range<usize> {
    let start = page.saturating_mul(page_size()).min(len);
    let end = (start + page_size()).min(len);
    start..end
}

fn draw_clientele_catalog(
    content: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    let mut types: Vec<&CustomerType> = data.customer_types.iter().collect();
    types.sort_by(|left, right| {
        left.profile_tier
            .cmp(&right.profile_tier)
            .then_with(|| left.name.cmp(&right.name))
    });
    for (slot, index) in visible_range(game.management_page, types.len()).enumerate() {
        let customer = types[index];
        let row = row_rect(content, slot);
        let unlocked = progression.is_customer_unlocked(&customer.id);
        draw_row_surface(row, unlocked);
        let name = if unlocked {
            customer.name.clone()
        } else {
            format!("{} (locked)", customer.name.replace(" Girl", ""))
        };
        draw_ui_text(
            &name,
            row.x + 16.0,
            row.y + 23.0,
            17.0,
            if unlocked { TEXT } else { MUTED },
        );
        draw_ui_text(
            &format!(
                "Tier {}  •  {}",
                customer.profile_tier.max(1),
                customer.description
            ),
            row.x + 16.0,
            row.y + 44.0,
            13.0,
            MUTED,
        );
        let cost = format_unlock_cost(data, &customer.unlock_cost);
        let can_attract = !unlocked && can_afford_cost(game, &customer.unlock_cost);
        draw_ui_text(
            &format!("Yield: {}-meat", customer.id),
            row.x + row.w - 310.0,
            row.y + 23.0,
            13.0,
            Color::new(0.93, 0.52, 0.60, 1.0),
        );
        if unlocked {
            draw_ui_text(
                data.text("ui_on_floor"),
                row.x + row.w - 122.0,
                row.y + 30.0,
                14.0,
                SUCCESS,
            );
        } else {
            draw_ui_text(
                &cost,
                row.x + row.w - 310.0,
                row.y + 45.0,
                12.0,
                if can_attract { TEXT } else { MUTED },
            );
            let button = Rect::new(row.x + row.w - 112.0, row.y + 11.0, 96.0, 42.0);
            draw_button(button, data.text("ui_attract"), can_attract, !can_attract);
            ui.attract_buttons.insert(customer.id.clone(), button);
        }
    }
}

fn draw_upgrade_catalog(
    content: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    for (slot, index) in visible_range(game.management_page, progression.upgrades.len()).enumerate()
    {
        let upgrade = &progression.upgrades[index];
        let row = row_rect(content, slot);
        draw_row_surface(row, upgrade.level > 0);
        draw_ui_text(
            &format!(
                "{}  Lv. {}/{}",
                upgrade.name, upgrade.level, upgrade.max_level
            ),
            row.x + 16.0,
            row.y + 23.0,
            17.0,
            TEXT,
        );
        draw_ui_text(
            &upgrade.description,
            row.x + 16.0,
            row.y + 45.0,
            13.0,
            MUTED,
        );
        let maxed = upgrade.level >= upgrade.max_level;
        let can_buy = progression.currency >= upgrade.cost && !maxed;
        draw_ui_text(
            &format!("Cost: ${}", upgrade.cost),
            row.x + row.w - 270.0,
            row.y + 33.0,
            14.0,
            if can_buy { SUCCESS } else { MUTED },
        );
        let button = Rect::new(row.x + row.w - 112.0, row.y + 11.0, 96.0, 42.0);
        draw_button(
            button,
            if maxed {
                data.text("ui_maxed")
            } else {
                data.text("ui_upgrade")
            },
            can_buy,
            !can_buy,
        );
        ui.upgrade_buttons.insert(upgrade.id.clone(), button);
    }
}

fn draw_recipe_catalog(
    content: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    for (slot, index) in visible_range(game.management_page, progression.recipes.len()).enumerate()
    {
        let recipe = &progression.recipes[index];
        let row = row_rect(content, slot);
        draw_row_surface(row, recipe.unlocked);
        draw_ui_text(
            &recipe.name,
            row.x + 16.0,
            row.y + 23.0,
            17.0,
            if recipe.unlocked { TEXT } else { MUTED },
        );
        draw_ui_text(
            &format!(
                "{}  •  {}",
                recipe.unlock_condition,
                format_unlock_cost(data, &recipe.ingredients)
            ),
            row.x + 16.0,
            row.y + 45.0,
            13.0,
            MUTED,
        );
        let action = Rect::new(row.x + row.w - 112.0, row.y + 11.0, 96.0, 42.0);
        draw_button(action, data.text("ui_review"), recipe.unlocked, false);
        ui.recipe_buttons.insert(recipe.id.clone(), action);
    }
}

fn draw_recipe_detail(
    content: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    let Some(id) = game.selected_recipe_id.as_deref() else {
        return;
    };
    let Some(recipe) = progression.recipes.iter().find(|recipe| recipe.id == id) else {
        return;
    };
    let detail = Rect::new(content.x, content.y, content.w, content.h);
    draw_row_surface(detail, recipe.unlocked);
    draw_ui_text(&recipe.name, detail.x + 20.0, detail.y + 32.0, 24.0, GOLD);
    let close = Rect::new(detail.x + detail.w - 108.0, detail.y + 14.0, 92.0, 42.0);
    draw_button(close, data.text("ui_back_to_catalog"), false, false);
    ui.recipe_detail_close = Some(close);
    let mut y = detail.y + 72.0;
    for line in wrap_text(&recipe.description, (detail.w * 0.58).max(180.0), 16.0) {
        draw_ui_text(&line, detail.x + 20.0, y, 16.0, TEXT);
        y += 22.0;
    }
    y += 10.0;
    draw_ui_text(
        data.text("ui_recipe_requirements"),
        detail.x + 20.0,
        y,
        16.0,
        GOLD,
    );
    y += 24.0;
    for (ingredient, amount) in sorted_cost(&recipe.ingredients) {
        let owned = game.ingredients.get(&ingredient).copied().unwrap_or(0);
        let owned_text = if owned == INFINITE_INGREDIENTS {
            "∞".to_string()
        } else {
            owned.to_string()
        };
        draw_ui_text(
            &format!("{ingredient}: {owned_text} / {amount}"),
            detail.x + 28.0,
            y,
            15.0,
            if owned == INFINITE_INGREDIENTS || owned >= amount {
                SUCCESS
            } else {
                Color::new(0.94, 0.42, 0.36, 1.0)
            },
        );
        y += 21.0;
    }
    draw_ui_text(
        &format!("Unlock: {}", recipe.unlock_condition),
        detail.x + 20.0,
        y + 8.0,
        14.0,
        if recipe.unlocked { SUCCESS } else { MUTED },
    );
    draw_ui_text(
        &format!(
            "Outcome: ${} base value  •  +{} future capacity",
            recipe.base_value, recipe.capacity_bonus
        ),
        detail.x + 20.0,
        y + 31.0,
        14.0,
        TEXT,
    );
    let can_craft = recipe.unlocked
        && recipe
            .ingredients
            .iter()
            .all(|(key, amount)| game.has_ing(key, *amount));
    let craft = Rect::new(
        detail.x + detail.w - 180.0,
        detail.y + detail.h - 58.0,
        160.0,
        42.0,
    );
    draw_button(craft, data.text("ui_craft"), can_craft, !can_craft);
    ui.recipe_detail_craft = Some(craft);
}

fn draw_prestige_catalog(
    content: Rect,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    let requirement = crate::engine::prestige_requirement(data, progression);
    draw_ui_text(
        &format!(
            "Level {}  •  {} / {} renown",
            progression.prestige_level, progression.total_score, requirement
        ),
        content.x + 16.0,
        content.y + 28.0,
        18.0,
        TEXT,
    );
    draw_ui_text(
        data.text("ui_prestige_catalog_hint"),
        content.x + 16.0,
        content.y + 54.0,
        14.0,
        MUTED,
    );
    let action = Rect::new(content.x + 16.0, content.y + 76.0, 190.0, 44.0);
    let can_prestige = progression.total_score >= requirement;
    draw_button(
        action,
        data.text("ui_review_prestige"),
        can_prestige,
        !can_prestige,
    );
    ui.prestige_button = Some(action);
    draw_ui_text(
        &format!(
            "{} permanent choices are available after the review.",
            data.prestige_perks.len()
        ),
        content.x + 224.0,
        content.y + 104.0,
        14.0,
        MUTED,
    );
}

fn draw_row_surface(rect: Rect, active: bool) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if active {
            Color::new(0.07, 0.075, 0.062, 0.98)
        } else {
            Color::new(0.055, 0.048, 0.055, 0.98)
        },
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE);
}

fn sorted_cost(cost: &std::collections::HashMap<String, i64>) -> Vec<(String, i64)> {
    let mut values: Vec<_> = cost
        .iter()
        .map(|(key, amount)| (key.clone(), *amount))
        .collect();
    values.sort_by(|left, right| left.0.cmp(&right.0));
    values
}

fn larder_total(game: &GameState, data: &GameData) -> i64 {
    game.ingredients
        .iter()
        .filter(|(key, amount)| {
            key.as_str() != data.balance.regular_ingredient_name && **amount > 0
        })
        .map(|(_, amount)| *amount)
        .sum()
}
