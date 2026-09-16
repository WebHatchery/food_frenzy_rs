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
    pub clientele_board_toggle: Option<Rect>,
    pub guest_info: HashMap<u32, Rect>,
    pub prestige_perk_buttons: HashMap<String, Rect>,
    pub day_next_button: Option<Rect>,
    /// True while a full-screen overlay (specialization choice, clientele
    /// board) is up: only that overlay's own hitboxes accept clicks.
    pub modal_open: bool,
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
