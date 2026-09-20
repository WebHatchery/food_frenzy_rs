//! Macroquad UI drawing and hitbox collection for Feast Frenzy.

mod actors;
mod ambience;
mod common;
mod day_summary;
mod dining;
mod floaters;
mod growth;
mod guest_status;
mod kitchen;
mod layout;
mod lounge;
mod management;
mod menu;
mod prestige_modal;
mod specialization;
mod sprites;
mod tutorial_panel;
mod types;
mod utility;

pub use layout::draw_and_collect_hitboxes;
pub use menu::{draw_settings_screen, draw_title_screen};
pub use types::{
    OverlayKind, SettingsAction, SettingsActions, TitleAction, TitleActions, UiActions,
};
