//! Application coordination: load the data and assets, advance the active
//! screen, and bridge UI intents to the game services.

use crate::assets::{
    load_asset_pack, load_character_textures, load_interior_sheet, load_title_texture,
};
use crate::audio::AudioBank;
use crate::commands::{
    apply_ui_command, clear_empty_selection, handle_keyboard_shortcuts, read_input_action,
    read_settings_action, read_title_action,
};
use crate::data::GameData;
use crate::lifecycle::{load_saved_game, start_new_game};
use crate::persistence::save_game;
use crate::player::handle_player_keyboard_movement;
use crate::simulation::update_game_world;
use crate::state::{
    DayStats, FloaterKind, GameState, GuestState, ProcessingCinematic, ProgressionState, Timers,
};
use crate::ui::{
    draw_and_collect_hitboxes, draw_settings_screen, draw_title_screen, SettingsAction, TitleAction,
};
use macroquad::prelude::*;
use macroquad_toolkit::capture;
use std::collections::HashMap;

const SAVE_INTERVAL_MS: f32 = 5_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppScreen {
    Title,
    Settings,
    Playing,
}

struct App {
    data: GameData,
    audio: AudioBank,
    character_textures: HashMap<String, Texture2D>,
    interior_sheet: Option<Texture2D>,
    title_texture: Option<Texture2D>,
    game_state: GameState,
    progression_state: ProgressionState,
    guest_state: GuestState,
    timers: Timers,
    selected_station: Option<String>,
    app_screen: AppScreen,
    title_message: String,
    fullscreen_enabled: bool,
}

pub async fn run() {
    let mut app = App::load().await;

    // Screenshot harness: when feast_FRENZY_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit. The harness clock
    // is passed through the same tick path as normal frame time.
    if let Some(configs) = capture::CaptureConfig::all_from_env("feast_FRENZY") {
        for config in configs {
            app.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |dt_ms| {
                app.tick(dt_ms);
            })
            .await;
        }
        return;
    }

    loop {
        app.tick(get_frame_time() * 1000.0);
        next_frame().await;
    }
}

impl App {
    async fn load() -> Self {
        let data = GameData::load();
        let asset_pack = load_asset_pack().await;
        let character_textures = load_character_textures(&data, asset_pack.as_ref()).await;
        let interior_sheet = load_interior_sheet(asset_pack.as_ref()).await;
        let title_texture = load_title_texture(asset_pack.as_ref()).await;
        // Audio loads last: the sound manager takes ownership of the pack.
        let audio = AudioBank::load(asset_pack).await;
        let game_state = GameState::new(&data);
        let progression_state = ProgressionState::from_game_data(&data);

        Self {
            data,
            audio,
            character_textures,
            interior_sheet,
            title_texture,
            game_state,
            progression_state,
            guest_state: GuestState::new(),
            timers: Timers::new(),
            selected_station: None,
            app_screen: AppScreen::Title,
            title_message: String::new(),
            fullscreen_enabled: false,
        }
    }

    /// Seed a specific scene for the screenshot harness.
    fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "title" => self.app_screen = AppScreen::Title,
            "settings" => self.app_screen = AppScreen::Settings,
            "clientele_board" => {
                self.start_new_game();
                self.seed_gameplay_demo();
                self.game_state.show_clientele_board = true;
            }
            "dining_rush" => {
                // A rush in full swing: extra guests inbound, banner up.
                self.start_new_game();
                self.seed_gameplay_demo();
                self.game_state.active_event = Some(crate::state::ActiveEvent {
                    event_id: "dinner-rush".to_string(),
                    remaining_ms: 30_000.0,
                });
                for _ in 0..40 {
                    update_game_world(
                        200.0,
                        &self.data,
                        &mut self.game_state,
                        &mut self.progression_state,
                        &mut self.guest_state,
                        &mut self.timers,
                    );
                }
            }
            "day_summary" => {
                // End of a productive first day: the closing ledger is up.
                self.start_new_game();
                self.seed_gameplay_demo();
                self.game_state.day_cycle.stats = DayStats {
                    cash_earned: 184,
                    renown_earned: 655,
                    guests_served: 9,
                    guests_lost: 1,
                    meat_gained: 7,
                    guests_processed: 2,
                    fresh_dishes: 6,
                    best_combo: 8,
                };
                self.game_state.day_cycle.summary_pending = true;
            }
            "specialization" => {
                // First processing just happened and no style chosen yet:
                // the house-style modal is up.
                self.start_new_game();
                self.seed_gameplay_demo();
                self.game_state.tutorial.skip();
                self.progression_state
                    .processed_customer_counts
                    .insert("pig".to_string(), 1);
            }
            "lounge" => {
                // The processing sequence mid-reveal, for verifying the
                // dramatized payoff without playing to a first processing.
                self.start_new_game();
                self.seed_gameplay_demo();
                let mut cinematic = ProcessingCinematic::new(
                    "Marnie".to_string(),
                    "pig".to_string(),
                    4,
                    "pig-meat".to_string(),
                    320,
                    64,
                    (420.0, 300.0),
                )
                .with_timing(self.data.balance.cinematic.clone());
                let reveal = self.data.balance.cinematic.reveal_ms;
                cinematic.advance(cinematic.total_ms() - reveal + 400.0);
                self.game_state.processing_cinematic = Some(cinematic);
            }
            _ => {
                // Default: jump straight into gameplay on a fresh save, then
                // warm the world into a lively state so headless captures show
                // seated guests and active stations instead of an empty room.
                self.start_new_game();
                self.seed_gameplay_demo();
            }
        }
    }

    /// Advance a fresh game into a representative mid-service moment for the
    /// screenshot harness: guests seated, one dish plated, another cooking.
    /// Capture-only — the normal game loop never calls this.
    fn seed_gameplay_demo(&mut self) {
        let step = |app: &mut Self, dt_ms: f32| {
            update_game_world(
                dt_ms,
                &app.data,
                &mut app.game_state,
                &mut app.progression_state,
                &mut app.guest_state,
                &mut app.timers,
            );
        };

        // Run ~12s of simulation so both opening guests arrive and take a table.
        for _ in 0..60 {
            step(self, 200.0);
        }
        // Plate a quick appetizer and get a slower roast going for visible state.
        crate::gameplay::start_cooking(
            "blue",
            &self.data,
            &self.progression_state,
            &mut self.game_state,
        );
        crate::gameplay::start_cooking(
            "yellow",
            &self.data,
            &self.progression_state,
            &mut self.game_state,
        );
        for _ in 0..18 {
            step(self, 200.0);
        }
        // Carry the ready appetizer so the serve prompt and buttons are shown.
        self.selected_station = Some("blue".to_string());
    }

    fn tick(&mut self, dt_ms: f32) {
        match self.app_screen {
            AppScreen::Title => self.tick_title(),
            AppScreen::Settings => self.tick_settings(),
            AppScreen::Playing => self.tick_playing(dt_ms),
        }
    }

    fn tick_title(&mut self) {
        let title_hits =
            draw_title_screen(self.title_texture.as_ref(), &self.title_message, &self.data);
        if let Some(action) = read_title_action(&title_hits) {
            self.handle_title_action(action);
        }

        if is_key_pressed(KeyCode::Enter) {
            self.start_new_game();
        }
    }

    fn tick_settings(&mut self) {
        let settings_hits =
            draw_settings_screen(self.fullscreen_enabled, self.audio.enabled, &self.data);
        if let Some(action) = read_settings_action(&settings_hits) {
            match action {
                SettingsAction::ToggleFullscreen => {
                    self.fullscreen_enabled = !self.fullscreen_enabled;
                    set_fullscreen(self.fullscreen_enabled);
                }
                SettingsAction::ToggleSound => {
                    self.audio.enabled = !self.audio.enabled;
                }
                SettingsAction::Back => {
                    self.app_screen = AppScreen::Title;
                }
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.app_screen = AppScreen::Title;
        }
    }

    fn tick_playing(&mut self, dt_ms: f32) {
        if self.game_state.processing_cinematic.is_some() {
            self.tick_processing_cinematic(dt_ms);
            return;
        }
        // The end-of-day ledger and the prestige choice both pause the world;
        // their modals still take clicks below.
        let paused = self.game_state.day_cycle.summary_pending || self.game_state.pending_prestige;
        if paused {
            self.game_state.floaters.update(dt_ms);
        } else {
            update_game_world(
                dt_ms,
                &self.data,
                &mut self.game_state,
                &mut self.progression_state,
                &mut self.guest_state,
                &mut self.timers,
            );
            handle_player_keyboard_movement(dt_ms, &self.data, &mut self.game_state);
        }

        let ui_hits = draw_and_collect_hitboxes(
            &self.game_state,
            &self.progression_state,
            &self.data,
            self.timers.elapsed_ms,
            &self.selected_station,
            &self.character_textures,
            self.interior_sheet.as_ref(),
        );

        if let Some(command) = read_input_action(ui_hits) {
            apply_ui_command(
                command,
                &self.data,
                &mut self.selected_station,
                &mut self.game_state,
                &mut self.progression_state,
                &mut self.guest_state,
            );
        }

        if !paused {
            handle_keyboard_shortcuts(
                &self.data,
                &mut self.selected_station,
                &mut self.game_state,
                &mut self.progression_state,
                &mut self.guest_state,
            );
            clear_empty_selection(&self.data, &mut self.selected_station, &mut self.game_state);
        }
        self.drain_sfx();
        self.save_if_due(dt_ms);
    }

    /// Play whatever sound cues gameplay queued this frame.
    fn drain_sfx(&mut self) {
        let cues = std::mem::take(&mut self.game_state.sfx_queue);
        for cue in cues {
            self.audio.play(cue);
        }
    }

    /// While the Last Meal Lounge sequence plays, the world holds its breath:
    /// simulation pauses, gameplay input is ignored, and the moment owns the
    /// screen. Rewards were applied when the invite succeeded.
    fn tick_processing_cinematic(&mut self, dt_ms: f32) {
        self.game_state.floaters.update(dt_ms);
        if let Some(cinematic) = &mut self.game_state.processing_cinematic {
            cinematic.advance(dt_ms);
        }

        let _ = draw_and_collect_hitboxes(
            &self.game_state,
            &self.progression_state,
            &self.data,
            self.timers.elapsed_ms,
            &self.selected_station,
            &self.character_textures,
            self.interior_sheet.as_ref(),
        );

        let dismiss = self
            .game_state
            .processing_cinematic
            .as_ref()
            .is_some_and(|cinematic| {
                cinematic.finished()
                    || (cinematic.can_dismiss()
                        && (is_mouse_button_released(MouseButton::Left)
                            || is_key_pressed(KeyCode::Space)
                            || is_key_pressed(KeyCode::Enter)))
            });
        if dismiss {
            if let Some(cinematic) = self.game_state.processing_cinematic.take() {
                let (x, y) = cinematic.from_floor;
                self.game_state.floaters.spawn_at(
                    format!("+{} {}", cinematic.meat_gain, cinematic.meat_type),
                    FloaterKind::Meat,
                    x,
                    y,
                );
                self.game_state.floaters.spawn_at(
                    format!(
                        "+{} renown  +${}",
                        cinematic.renown_gain, cinematic.cash_gain
                    ),
                    FloaterKind::Renown,
                    x,
                    (y - 46.0).max(40.0),
                );
            }
        }
        self.drain_sfx();
        self.save_if_due(dt_ms);
    }

    fn handle_title_action(&mut self, action: TitleAction) {
        match action {
            TitleAction::NewGame => self.start_new_game(),
            TitleAction::LoadGame => self.load_saved_game(),
            TitleAction::Settings => {
                self.app_screen = AppScreen::Settings;
            }
            TitleAction::Exit => {
                macroquad::miniquad::window::quit();
            }
        }
    }

    fn start_new_game(&mut self) {
        start_new_game(
            &self.data,
            &mut self.game_state,
            &mut self.progression_state,
            &mut self.guest_state,
            &mut self.timers,
            &mut self.selected_station,
        );
        self.title_message.clear();
        self.app_screen = AppScreen::Playing;
    }

    fn load_saved_game(&mut self) {
        match load_saved_game(
            &self.data,
            &mut self.game_state,
            &mut self.progression_state,
            &mut self.guest_state,
            &mut self.timers,
            &mut self.selected_station,
        ) {
            Ok(()) => {
                self.title_message.clear();
                self.app_screen = AppScreen::Playing;
            }
            Err(message) => {
                self.title_message = message;
            }
        }
    }

    fn save_if_due(&mut self, dt_ms: f32) {
        self.timers.save_timer.set_interval(SAVE_INTERVAL_MS);
        if !self.timers.save_timer.tick_once(dt_ms) {
            return;
        }

        if let Err(error) = save_game(
            &self.game_state,
            &self.progression_state,
            &self.guest_state,
            &self.timers,
            &self.selected_station,
        ) {
            self.game_state.add_message(
                self.data
                    .text_format("message_missing_save", [("error", error)].as_slice()),
            );
        }
    }
}
