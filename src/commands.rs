//! Input translation: convert released UI controls and keyboard shortcuts into
//! explicit gameplay commands, then dispatch those commands.

use crate::data::{GameData, STATION_COLORS};
use crate::gameplay::{
    attract_customer_type, craft_recipe, invite_customer_to_vip, serve_customer, try_prestige,
};
use crate::player::{
    clear_player_carry, interact_with_nearest_customer, interact_with_nearest_station,
    player_target_for_customer, select_station_with_player, send_player_to_customer,
    set_player_target, start_station_with_player,
};
use crate::state::{GameState, GuestState, ProgressionState};
use crate::ui::{SettingsAction, SettingsActions, TitleAction, TitleActions, UiActions};
use macroquad::prelude::*;
use macroquad_toolkit::input::was_clicked_rect;

#[derive(Clone)]
pub enum UiCommand {
    StartCooking(String),
    SelectDish(String),
    Serve(u32),
    InviteVip(u32),
    BuyUpgrade(String),
    CraftRecipe(String),
    AttractCustomer(String),
    Prestige,
    ClearSelection,
    TutorialNext,
    TutorialSkip,
    ChooseSpecialization(String),
    ToggleGuestInfo(u32),
    ChoosePrestigePerk(String),
    StartNextDay,
    OpenManagement,
    CloseManagement,
    TogglePauseMenu,
    Resume,
    OpenHelp,
    CloseHelp,
    OpenHistory,
    CloseHistory,
    OpenSettings,
    ReturnTitle,
    SetManagementTab(u8),
    ManagementPrevious,
    ManagementNext,
    SelectRecipe(String),
    CloseRecipeDetail,
    CraftSelectedRecipe,
    PrestigePrevious,
    PrestigeNext,
    CancelPrestige,
    ConfirmPrestige,
    SpecializationPrevious,
    SpecializationNext,
}

pub struct UiCommandContext<'a> {
    data: &'a GameData,
    selected_station: &'a mut Option<String>,
    game_state: &'a mut GameState,
    progression: &'a mut ProgressionState,
    guest_state: &'a mut GuestState,
}

impl<'a> UiCommandContext<'a> {
    pub fn new(
        data: &'a GameData,
        selected_station: &'a mut Option<String>,
        game_state: &'a mut GameState,
        progression: &'a mut ProgressionState,
        guest_state: &'a mut GuestState,
    ) -> Self {
        Self {
            data,
            selected_station,
            game_state,
            progression,
            guest_state,
        }
    }
}

pub fn read_title_action(ui_hits: &TitleActions) -> Option<TitleAction> {
    ui_hits.released_action()
}

pub fn read_settings_action(ui_hits: &SettingsActions) -> Option<SettingsAction> {
    ui_hits.released_action()
}

pub fn read_input_action(ui_hits: UiActions) -> Option<UiCommand> {
    // Exactly one visible overlay owns input. This prevents a covered floor
    // action from firing while a player is reading a comparison or menu.
    match ui_hits.overlay {
        crate::ui::OverlayKind::Management => {
            if ui_hits.management_close.is_some_and(was_clicked_rect) {
                return Some(UiCommand::CloseManagement);
            }
            if let Some((tab, _)) = ui_hits
                .management_tabs
                .into_iter()
                .find(|(_, rect)| was_clicked_rect(*rect))
            {
                return Some(UiCommand::SetManagementTab(tab));
            }
            if ui_hits.recipe_detail_close.is_some_and(was_clicked_rect) {
                return Some(UiCommand::CloseRecipeDetail);
            }
            if ui_hits.recipe_detail_craft.is_some_and(was_clicked_rect) {
                return Some(UiCommand::CraftSelectedRecipe);
            }
            if ui_hits.management_previous.is_some_and(was_clicked_rect) {
                return Some(UiCommand::ManagementPrevious);
            }
            if ui_hits.management_next.is_some_and(was_clicked_rect) {
                return Some(UiCommand::ManagementNext);
            }
            if let Some(id) = find_map_hit(ui_hits.recipe_buttons) {
                return Some(UiCommand::SelectRecipe(id));
            }
            if let Some(id) = find_map_hit(ui_hits.upgrade_buttons) {
                return Some(UiCommand::BuyUpgrade(id));
            }
            if let Some(id) = find_map_hit(ui_hits.attract_buttons) {
                return Some(UiCommand::AttractCustomer(id));
            }
            if ui_hits.prestige_button.is_some_and(was_clicked_rect) {
                return Some(UiCommand::Prestige);
            }
            return None;
        }
        crate::ui::OverlayKind::Pause => {
            if ui_hits.pause_resume.is_some_and(was_clicked_rect) {
                return Some(UiCommand::Resume);
            }
            if ui_hits.pause_help.is_some_and(was_clicked_rect) {
                return Some(UiCommand::OpenHelp);
            }
            if ui_hits.pause_settings.is_some_and(was_clicked_rect) {
                return Some(UiCommand::OpenSettings);
            }
            if ui_hits.pause_title.is_some_and(was_clicked_rect) {
                return Some(UiCommand::ReturnTitle);
            }
            return None;
        }
        crate::ui::OverlayKind::Help => {
            return ui_hits
                .help_close
                .filter(|rect| was_clicked_rect(*rect))
                .map(|_| UiCommand::CloseHelp);
        }
        crate::ui::OverlayKind::History => {
            return ui_hits
                .history_close
                .filter(|rect| was_clicked_rect(*rect))
                .map(|_| UiCommand::CloseHistory);
        }
        crate::ui::OverlayKind::Specialization => {
            if ui_hits
                .specialization_previous
                .is_some_and(was_clicked_rect)
            {
                return Some(UiCommand::SpecializationPrevious);
            }
            if ui_hits.specialization_next.is_some_and(was_clicked_rect) {
                return Some(UiCommand::SpecializationNext);
            }
            return find_map_hit(ui_hits.specialization_buttons)
                .map(UiCommand::ChooseSpecialization);
        }
        crate::ui::OverlayKind::Prestige => {
            if ui_hits.prestige_cancel.is_some_and(was_clicked_rect) {
                return Some(UiCommand::CancelPrestige);
            }
            if ui_hits.prestige_previous.is_some_and(was_clicked_rect) {
                return Some(UiCommand::PrestigePrevious);
            }
            if ui_hits.prestige_next.is_some_and(was_clicked_rect) {
                return Some(UiCommand::PrestigeNext);
            }
            if ui_hits.prestige_confirm.is_some_and(was_clicked_rect) {
                return Some(UiCommand::ConfirmPrestige);
            }
            return find_map_hit(ui_hits.prestige_perk_buttons).map(UiCommand::ChoosePrestigePerk);
        }
        crate::ui::OverlayKind::DaySummary => {
            return ui_hits
                .day_next_button
                .filter(|rect| was_clicked_rect(*rect))
                .map(|_| UiCommand::StartNextDay);
        }
        crate::ui::OverlayKind::None => {}
    }
    if ui_hits.management_button.is_some_and(was_clicked_rect) {
        return Some(UiCommand::OpenManagement);
    }
    if ui_hits.menu_button.is_some_and(was_clicked_rect) {
        return Some(UiCommand::TogglePauseMenu);
    }
    if ui_hits.history_button.is_some_and(was_clicked_rect) {
        return Some(UiCommand::OpenHistory);
    }
    if ui_hits.tutorial_next.is_some_and(was_clicked_rect) {
        return Some(UiCommand::TutorialNext);
    }
    if ui_hits.tutorial_skip.is_some_and(was_clicked_rect) {
        return Some(UiCommand::TutorialSkip);
    }
    if ui_hits.clear_selection.is_some_and(was_clicked_rect) {
        return Some(UiCommand::ClearSelection);
    }
    if let Some(id) = find_map_hit(ui_hits.guest_info) {
        return Some(UiCommand::ToggleGuestInfo(id));
    }

    find_vec_hit(ui_hits.station_select)
        .map(UiCommand::SelectDish)
        .or_else(|| find_vec_hit(ui_hits.station_cook).map(UiCommand::StartCooking))
        .or_else(|| find_map_hit(ui_hits.serve_customer).map(UiCommand::Serve))
        .or_else(|| find_map_hit(ui_hits.invite_customer).map(UiCommand::InviteVip))
        .or_else(|| find_map_hit(ui_hits.upgrade_buttons).map(UiCommand::BuyUpgrade))
        .or_else(|| find_map_hit(ui_hits.recipe_buttons).map(UiCommand::CraftRecipe))
        .or_else(|| find_map_hit(ui_hits.attract_buttons).map(UiCommand::AttractCustomer))
        .or_else(|| {
            ui_hits
                .prestige_button
                .filter(|rect| was_clicked_rect(*rect))
                .map(|_| UiCommand::Prestige)
        })
}

pub fn apply_ui_command(command: UiCommand, mut context: UiCommandContext<'_>) {
    match command {
        UiCommand::StartCooking(station_color) => {
            start_station_with_player(
                &station_color,
                context.data,
                context.progression,
                context.selected_station,
                context.game_state,
            );
        }
        UiCommand::SelectDish(station_color) => {
            select_station_with_player(
                station_color,
                context.data,
                context.selected_station,
                context.game_state,
            );
        }
        UiCommand::Serve(customer_id) => {
            serve_selected_customer(customer_id, &mut context);
        }
        UiCommand::InviteVip(customer_id) => {
            invite_selected_customer(customer_id, &mut context);
        }
        UiCommand::BuyUpgrade(upgrade_id) => {
            buy_upgrade(upgrade_id, &mut context);
        }
        UiCommand::CraftRecipe(recipe_id) => {
            craft_recipe(
                &recipe_id,
                context.data,
                context.game_state,
                context.progression,
            );
        }
        UiCommand::AttractCustomer(customer_type_id) => {
            attract_customer_type(
                &customer_type_id,
                context.data,
                context.game_state,
                context.progression,
            );
        }
        UiCommand::Prestige => {
            try_prestige(context.data, context.progression, context.game_state);
        }
        UiCommand::ClearSelection => {
            *context.selected_station = None;
            clear_player_carry(context.data, context.game_state);
        }
        UiCommand::ToggleGuestInfo(customer_id) => {
            context.game_state.selected_guest_id =
                (context.game_state.selected_guest_id != Some(customer_id)).then_some(customer_id);
        }
        UiCommand::TutorialNext => {
            context
                .game_state
                .tutorial
                .advance(&context.data.tutorial_steps);
        }
        UiCommand::TutorialSkip => {
            context.game_state.tutorial.skip();
            context
                .game_state
                .add_message(context.data.text("message_tutorial_skipped"));
        }
        UiCommand::ChooseSpecialization(specialization_id) => {
            choose_specialization(&specialization_id, &mut context);
        }
        UiCommand::ChoosePrestigePerk(perk_id) => {
            if context.data.prestige_perk_by_id(&perk_id).is_some() {
                context.game_state.selected_prestige_perk = Some(perk_id);
            }
        }
        UiCommand::StartNextDay => {
            start_next_day(&mut context);
        }
        UiCommand::OpenManagement => {
            context.game_state.show_management = true;
            context.game_state.show_pause_menu = false;
            context.game_state.show_help = false;
            context.game_state.show_history = false;
        }
        UiCommand::CloseManagement => {
            context.game_state.show_management = false;
            context.game_state.selected_recipe_id = None;
        }
        UiCommand::TogglePauseMenu => {
            context.game_state.show_pause_menu = true;
            context.game_state.show_help = false;
            context.game_state.show_history = false;
        }
        UiCommand::Resume => {
            context.game_state.show_pause_menu = false;
        }
        UiCommand::OpenHelp => {
            context.game_state.show_pause_menu = false;
            context.game_state.show_help = true;
        }
        UiCommand::CloseHelp => {
            context.game_state.show_help = false;
        }
        UiCommand::OpenHistory => {
            context.game_state.show_history = true;
        }
        UiCommand::CloseHistory => {
            context.game_state.show_history = false;
        }
        UiCommand::OpenSettings | UiCommand::ReturnTitle => {
            context.game_state.show_pause_menu = false;
        }
        UiCommand::SetManagementTab(tab) => {
            context.game_state.management_tab = tab.min(3);
            context.game_state.management_page = 0;
            context.game_state.selected_recipe_id = None;
        }
        UiCommand::ManagementPrevious => {
            context.game_state.management_page =
                context.game_state.management_page.saturating_sub(1);
        }
        UiCommand::ManagementNext => {
            let count = match context.game_state.management_tab {
                0 => context.data.customer_types.len(),
                1 => context.progression.upgrades.len(),
                2 => context.progression.recipes.len(),
                _ => context.data.prestige_perks.len(),
            };
            let page_size = if screen_height() < 520.0 { 3 } else { 6 };
            let max_page = count.saturating_sub(1) / page_size;
            context.game_state.management_page =
                (context.game_state.management_page + 1).min(max_page);
        }
        UiCommand::SelectRecipe(recipe_id) => {
            context.game_state.selected_recipe_id = Some(recipe_id);
        }
        UiCommand::CloseRecipeDetail => {
            context.game_state.selected_recipe_id = None;
        }
        UiCommand::CraftSelectedRecipe => {
            if let Some(recipe_id) = context.game_state.selected_recipe_id.clone() {
                craft_recipe(
                    &recipe_id,
                    context.data,
                    context.game_state,
                    context.progression,
                );
            }
        }
        UiCommand::PrestigePrevious => {
            context.game_state.prestige_page = context.game_state.prestige_page.saturating_sub(1);
        }
        UiCommand::PrestigeNext => {
            let page_size = if screen_width() < 900.0 { 2 } else { 3 };
            let max_page = context.data.prestige_perks.len().saturating_sub(1) / page_size;
            context.game_state.prestige_page = (context.game_state.prestige_page + 1).min(max_page);
        }
        UiCommand::CancelPrestige => {
            context.game_state.pending_prestige = false;
            context.game_state.selected_prestige_perk = None;
            context.game_state.prestige_page = 0;
        }
        UiCommand::ConfirmPrestige => {
            if let Some(perk_id) = context.game_state.selected_prestige_perk.clone() {
                crate::gameplay::confirm_prestige(
                    &perk_id,
                    context.data,
                    context.game_state,
                    context.progression,
                );
            }
        }
        UiCommand::SpecializationPrevious => {
            context.game_state.specialization_page =
                context.game_state.specialization_page.saturating_sub(1);
        }
        UiCommand::SpecializationNext => {
            let max_page = context.data.specializations.len().saturating_sub(1);
            context.game_state.specialization_page =
                (context.game_state.specialization_page + 1).min(max_page);
        }
    }
}

fn buy_upgrade(upgrade_id: String, context: &mut UiCommandContext<'_>) {
    let cost = context
        .progression
        .upgrades
        .iter()
        .find(|upgrade| upgrade.id == upgrade_id)
        .map(|upgrade| upgrade.cost)
        .unwrap_or(0);
    if context.progression.buy_upgrade(&upgrade_id) {
        context.game_state.add_message(context.data.text_format(
            "message_upgrade_bought",
            [("upgrade", upgrade_id.clone())].as_slice(),
        ));
        context.game_state.floaters.spawn(
            context.data.text_format(
                "message_upgrade_floater",
                [("cost", cost.to_string())].as_slice(),
            ),
            crate::state::FloaterKind::Cash,
            crate::state::FloaterAnchor::Header,
        );
    } else {
        context.game_state.add_message(context.data.text_format(
            "message_upgrade_unavailable",
            [("upgrade", upgrade_id)].as_slice(),
        ));
    }
}

fn choose_specialization(id: &str, context: &mut UiCommandContext<'_>) {
    let Some(def) = context.data.specialization_by_id(id) else {
        return;
    };
    if context.progression.choose_specialization(def) {
        context.game_state.add_message(context.data.text_format(
            "message_house_style_selected",
            [("style", def.name.clone()), ("flavor", def.flavor.clone())].as_slice(),
        ));
        context.game_state.floaters.spawn(
            context.data.text_format(
                "message_house_style_floater",
                [("style", def.name.clone())].as_slice(),
            ),
            crate::state::FloaterKind::Renown,
            crate::state::FloaterAnchor::Header,
        );
    }
}

fn start_next_day(context: &mut UiCommandContext<'_>) {
    let next_day = context.game_state.day_cycle.day.saturating_add(1);
    let next_goal =
        crate::engine::select_next_day_goal(context.data, next_day, context.progression);
    context.game_state.day_cycle.start_next_day(next_goal.id);
    context.game_state.add_message(context.data.text_format(
        "message_day_start",
        [("day", context.game_state.day_cycle.day.to_string())].as_slice(),
    ));
}

pub fn handle_keyboard_shortcuts(
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    if is_key_pressed(KeyCode::C) {
        *selected_station = None;
        clear_player_carry(data, game_state);
        return;
    }
    if game_state.player.action_lock_ms > 0.0 {
        return;
    }
    if is_key_pressed(KeyCode::P) {
        try_prestige(data, progression, game_state);
    }
    if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Space) {
        if interact_with_nearest_customer(
            data,
            selected_station,
            game_state,
            progression,
            guest_state,
        ) {
            return;
        }
        interact_with_nearest_station(data, selected_station, game_state, progression);
    }

    for (key, station_color) in station_key_bindings() {
        if is_key_pressed(key) {
            start_station_with_player(
                station_color,
                data,
                progression,
                selected_station,
                game_state,
            );
        }
    }
}

pub fn clear_empty_selection(
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
) {
    let should_clear = selected_station.as_ref().is_some_and(|station| {
        game_state
            .cooking_stations
            .get(station)
            .is_none_or(|station| station.dishes.is_empty())
    });
    if should_clear {
        *selected_station = None;
        if !game_state.player.clear_carry_on_arrival {
            clear_player_carry(data, game_state);
        }
    }
}

fn serve_selected_customer(customer_id: u32, context: &mut UiCommandContext<'_>) {
    if let Some(station_color) = context.selected_station.clone() {
        if serve_customer(
            &station_color,
            customer_id,
            context.data,
            context.game_state,
            context.progression,
            context.guest_state,
        ) {
            send_player_to_customer(
                customer_id,
                &station_color,
                context.data,
                context.game_state,
            );
            *context.selected_station = None;
        }
    } else {
        context
            .game_state
            .add_message(context.data.text("message_select_dish"));
    }
}

fn invite_selected_customer(customer_id: u32, context: &mut UiCommandContext<'_>) {
    let player_target = player_target_for_customer(customer_id, context.game_state);
    if invite_customer_to_vip(
        customer_id,
        context.data,
        context.game_state,
        context.progression,
        context.guest_state,
    ) {
        if let Some((x, y)) = player_target {
            set_player_target(
                context.game_state,
                x,
                y,
                context.data.text("task_vip"),
                None,
                false,
            );
        }
        *context.selected_station = None;
        clear_player_carry(context.data, context.game_state);
    }
}

fn find_vec_hit(values: Vec<(String, Rect)>) -> Option<String> {
    values
        .into_iter()
        .find(|(_, rect)| was_clicked_rect(*rect))
        .map(|(value, _)| value)
}

fn find_map_hit<K>(values: std::collections::HashMap<K, Rect>) -> Option<K> {
    values
        .into_iter()
        .find(|(_, rect)| was_clicked_rect(*rect))
        .map(|(value, _)| value)
}

fn station_key_bindings() -> [(KeyCode, &'static str); 4] {
    [
        (KeyCode::Key1, STATION_COLORS[0]),
        (KeyCode::Key2, STATION_COLORS[1]),
        (KeyCode::Key3, STATION_COLORS[2]),
        (KeyCode::Key4, STATION_COLORS[3]),
    ]
}
