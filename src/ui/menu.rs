//! Title and settings screens, including their touch-sized hit targets.

use super::common::{draw_menu_button, BACKGROUND, MUTED, PANEL, TEXT};
use super::types::{SettingsActions, TitleActions};
use crate::data::GameData;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_text_centered_in_box;
use macroquad_toolkit::ui::draw_ui_text;

fn draw_title_background(title_texture: Option<&Texture2D>, data: &GameData) {
    let width = screen_width();
    let height = screen_height();
    clear_background(Color::new(0.02, 0.018, 0.016, 1.0));

    if let Some(texture) = title_texture {
        let scale = (width / texture.width()).max(height / texture.height());
        let dest_size = vec2(texture.width() * scale, texture.height() * scale);
        let dest_x = (width - dest_size.x) * 0.5;
        let dest_y = (height - dest_size.y) * 0.5;
        draw_texture_ex(
            texture,
            dest_x,
            dest_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                ..Default::default()
            },
        );
        draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.0, 0.10));
    } else {
        draw_text_centered_in_box(
            data.text("title_name"),
            0.0,
            height * 0.22,
            width,
            120.0,
            72.0,
            TEXT,
        );
    }

    let band_h = 170.0_f32.min(height * 0.28);
    draw_rectangle(
        0.0,
        height - band_h,
        width,
        band_h,
        Color::new(0.02, 0.018, 0.016, 0.78),
    );
    draw_rectangle(
        0.0,
        height - band_h,
        width,
        1.0,
        Color::new(0.80, 0.50, 0.25, 0.55),
    );
}

fn title_button_layout(width: f32, height: f32) -> TitleActions {
    let gap = 16.0;
    if width >= 860.0 {
        let button_w = ((width - 96.0 - gap * 3.0) / 4.0).clamp(168.0, 250.0);
        let total_w = button_w * 4.0 + gap * 3.0;
        let start_x = (width - total_w) * 0.5;
        let y = height - 108.0;
        return TitleActions {
            new_game: Rect::new(start_x, y, button_w, 54.0),
            load_game: Rect::new(start_x + (button_w + gap), y, button_w, 54.0),
            settings: Rect::new(start_x + (button_w + gap) * 2.0, y, button_w, 54.0),
            exit: Rect::new(start_x + (button_w + gap) * 3.0, y, button_w, 54.0),
        };
    }

    let button_w = (width - 64.0).clamp(220.0, 320.0);
    let button_h = 48.0;
    let start_x = (width - button_w) * 0.5;
    let start_y = (height - (button_h * 4.0 + gap * 3.0) - 24.0).max(120.0);
    TitleActions {
        new_game: Rect::new(start_x, start_y, button_w, button_h),
        load_game: Rect::new(start_x, start_y + (button_h + gap), button_w, button_h),
        settings: Rect::new(
            start_x,
            start_y + (button_h + gap) * 2.0,
            button_w,
            button_h,
        ),
        exit: Rect::new(
            start_x,
            start_y + (button_h + gap) * 3.0,
            button_w,
            button_h,
        ),
    }
}

fn draw_quiet_menu_button(rect: Rect, text: &str) {
    let hovered = rect.contains(vec2(mouse_position().0, mouse_position().1));
    let fill = if hovered {
        Color::new(0.16, 0.14, 0.13, 0.92)
    } else {
        Color::new(0.06, 0.055, 0.052, 0.72)
    };
    let border = if hovered {
        Color::new(0.78, 0.52, 0.30, 1.0)
    } else {
        Color::new(0.42, 0.36, 0.32, 0.85)
    };
    macroquad_toolkit::ui::draw_surface(
        rect,
        &macroquad_toolkit::ui::SurfaceStyle::new(fill).with_border(1.0, border),
    );
    draw_text_centered_in_box(
        text,
        rect.x + 8.0,
        rect.y,
        rect.w - 16.0,
        rect.h,
        (rect.h * 0.34).clamp(16.0, 20.0),
        if hovered { TEXT } else { MUTED },
    );
}

pub fn draw_title_screen(
    title_texture: Option<&Texture2D>,
    status_message: &str,
    data: &GameData,
) -> TitleActions {
    let width = screen_width();
    let height = screen_height();
    draw_title_background(title_texture, data);

    let actions = title_button_layout(width, height);

    // The house's public promise: warm and welcoming on first read, and only
    // later — once the player learns what the Lounge is for — quietly sinister.
    draw_text_centered_in_box(
        data.text("title_promise"),
        24.0,
        actions.new_game.y - 76.0,
        width - 48.0,
        26.0,
        19.0,
        LIGHTGRAY,
    );

    if !status_message.is_empty() {
        draw_text_centered_in_box(
            status_message,
            24.0,
            actions.new_game.y - 42.0,
            width - 48.0,
            28.0,
            19.0,
            LIGHTGRAY,
        );
    }

    draw_menu_button(actions.new_game, data.text("menu_new_game"));
    draw_menu_button(actions.load_game, data.text("menu_load_game"));
    draw_quiet_menu_button(actions.settings, data.text("menu_settings"));
    draw_quiet_menu_button(actions.exit, data.text("menu_exit"));

    actions
}

fn draw_toggle(rect: Rect, enabled: bool, data: &GameData) {
    let bg = if enabled {
        Color::new(0.25, 0.58, 0.39, 1.0)
    } else {
        Color::new(0.25, 0.25, 0.27, 1.0)
    };
    let border = if enabled { LIME } else { MUTED };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border);

    let knob_size = rect.h - 10.0;
    let knob_x = if enabled {
        rect.x + rect.w - knob_size - 5.0
    } else {
        rect.x + 5.0
    };
    draw_rectangle(
        knob_x,
        rect.y + 5.0,
        knob_size,
        knob_size,
        Color::new(0.94, 0.93, 0.86, 1.0),
    );

    draw_text_centered_in_box(
        if enabled {
            data.text("toggle_on")
        } else {
            data.text("toggle_off")
        },
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        16.0,
        WHITE,
    );
}

pub fn draw_settings_screen(
    fullscreen_enabled: bool,
    sound_enabled: bool,
    data: &GameData,
) -> SettingsActions {
    let width = screen_width();
    let height = screen_height();
    clear_background(BACKGROUND);

    draw_rectangle(0.0, 0.0, width, 96.0, Color::new(0.075, 0.065, 0.06, 1.0));
    draw_rectangle(0.0, 95.0, width, 1.0, Color::new(0.80, 0.50, 0.25, 0.55));
    draw_ui_text(data.text("menu_settings"), 32.0, 60.0, 36.0, TEXT);

    let content_w = width.min(720.0);
    let content_x = (width - content_w) * 0.5;
    let row_surface = macroquad_toolkit::ui::SurfaceStyle::new(PANEL)
        .with_border(1.0, Color::new(0.20, 0.20, 0.22, 1.0));

    let row = Rect::new(content_x + 24.0, height * 0.32, content_w - 48.0, 78.0);
    macroquad_toolkit::ui::draw_surface(row, &row_surface);
    draw_ui_text(
        data.text("settings_fullscreen"),
        row.x + 24.0,
        row.y + 48.0,
        24.0,
        TEXT,
    );
    let toggle = Rect::new(row.x + row.w - 142.0, row.y + 16.0, 112.0, 46.0);
    draw_toggle(toggle, fullscreen_enabled, data);

    let sound_row = Rect::new(row.x, row.y + row.h + 16.0, row.w, 78.0);
    macroquad_toolkit::ui::draw_surface(sound_row, &row_surface);
    draw_ui_text(
        data.text("settings_sound"),
        sound_row.x + 24.0,
        sound_row.y + 48.0,
        24.0,
        TEXT,
    );
    let sound_toggle = Rect::new(
        sound_row.x + sound_row.w - 142.0,
        sound_row.y + 16.0,
        112.0,
        46.0,
    );
    draw_toggle(sound_toggle, sound_enabled, data);

    let back = Rect::new(content_x + 24.0, height - 112.0, 180.0, 54.0);
    draw_menu_button(back, data.text("menu_back"));

    SettingsActions {
        fullscreen_toggle: row,
        sound_toggle: sound_row,
        back,
    }
}
