//! UI hitbox records and semantic actions shared by rendering and input.

use macroquad::prelude::*;
use macroquad_toolkit::input::was_clicked_rect;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct UiActions {
    pub station_cook: Vec<(String, Rect)>,
    pub station_select: Vec<(String, Rect)>,
    pub serve_customer: HashMap<u32, Rect>,
    pub invite_customer: HashMap<u32, Rect>,
    pub upgrade_buttons: HashMap<String, Rect>,
    pub recipe_buttons: HashMap<String, Rect>,
    pub attract_buttons: HashMap<String, Rect>,
    pub prestige_button: Option<Rect>,
    pub clear_selection: Option<Rect>,
    pub tutorial_next: Option<Rect>,
    pub tutorial_skip: Option<Rect>,
    pub specialization_buttons: HashMap<String, Rect>,
    pub guest_info: HashMap<u32, Rect>,
    pub prestige_perk_buttons: HashMap<String, Rect>,
    pub day_next_button: Option<Rect>,
    pub management_button: Option<Rect>,
    pub menu_button: Option<Rect>,
    pub history_button: Option<Rect>,
    pub management_close: Option<Rect>,
    pub management_tabs: Vec<(u8, Rect)>,
    pub management_previous: Option<Rect>,
    pub management_next: Option<Rect>,
    pub recipe_detail_close: Option<Rect>,
    pub recipe_detail_craft: Option<Rect>,
    pub pause_resume: Option<Rect>,
    pub pause_help: Option<Rect>,
    pub pause_settings: Option<Rect>,
    pub pause_title: Option<Rect>,
    pub help_close: Option<Rect>,
    pub history_close: Option<Rect>,
    pub prestige_previous: Option<Rect>,
    pub prestige_next: Option<Rect>,
    pub prestige_cancel: Option<Rect>,
    pub prestige_confirm: Option<Rect>,
    pub specialization_previous: Option<Rect>,
    pub specialization_next: Option<Rect>,
    /// True while a full-screen overlay (specialization choice, clientele
    /// board) is up: only that overlay's own hitboxes accept clicks.
    pub modal_open: bool,
    pub overlay: OverlayKind,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlayKind {
    #[default]
    None,
    Management,
    Pause,
    Help,
    History,
    Specialization,
    Prestige,
    DaySummary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleAction {
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

#[derive(Clone, Debug)]
pub struct TitleActions {
    pub new_game: Rect,
    pub load_game: Rect,
    pub settings: Rect,
    pub exit: Rect,
}

impl TitleActions {
    pub fn action_at(&self, point: Vec2) -> Option<TitleAction> {
        if self.new_game.contains(point) {
            Some(TitleAction::NewGame)
        } else if self.load_game.contains(point) {
            Some(TitleAction::LoadGame)
        } else if self.settings.contains(point) {
            Some(TitleAction::Settings)
        } else if self.exit.contains(point) {
            Some(TitleAction::Exit)
        } else {
            None
        }
    }

    pub fn released_action(&self) -> Option<TitleAction> {
        if was_clicked_rect(self.new_game) {
            Some(TitleAction::NewGame)
        } else if was_clicked_rect(self.load_game) {
            Some(TitleAction::LoadGame)
        } else if was_clicked_rect(self.settings) {
            Some(TitleAction::Settings)
        } else if was_clicked_rect(self.exit) {
            Some(TitleAction::Exit)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    ToggleFullscreen,
    ToggleSound,
    Back,
}

#[derive(Clone, Debug)]
pub struct SettingsActions {
    pub fullscreen_toggle: Rect,
    pub sound_toggle: Rect,
    pub back: Rect,
}

impl SettingsActions {
    pub fn action_at(&self, point: Vec2) -> Option<SettingsAction> {
        if self.fullscreen_toggle.contains(point) {
            Some(SettingsAction::ToggleFullscreen)
        } else if self.sound_toggle.contains(point) {
            Some(SettingsAction::ToggleSound)
        } else if self.back.contains(point) {
            Some(SettingsAction::Back)
        } else {
            None
        }
    }

    pub fn released_action(&self) -> Option<SettingsAction> {
        if was_clicked_rect(self.fullscreen_toggle) {
            Some(SettingsAction::ToggleFullscreen)
        } else if was_clicked_rect(self.sound_toggle) {
            Some(SettingsAction::ToggleSound)
        } else if was_clicked_rect(self.back) {
            Some(SettingsAction::Back)
        } else {
            None
        }
    }
}
