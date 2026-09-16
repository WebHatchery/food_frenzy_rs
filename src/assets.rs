//! Asset-pack loading and texture lookup for the Macroquad renderer.

use crate::data::GameData;
use macroquad::prelude::*;
use macroquad_toolkit::assets::{load_texture_from_pack_or_file, AssetPack};
use std::collections::HashMap;

const ASSET_PACK_PATH: &str = "assets.zip";

const TITLE_TEXTURE_PATHS: [&str; 2] = [
    "assets/images/feast_frenzy_title.png",
    "feast_frenzy_title.png",
];

const INTERIOR_SHEET_PATH: &str = "assets/images/interior_sheet.png";

pub async fn load_asset_pack() -> Option<AssetPack> {
    AssetPack::load(ASSET_PACK_PATH).await.ok()
}

pub async fn load_character_textures(
    data: &GameData,
    asset_pack: Option<&AssetPack>,
) -> HashMap<String, Texture2D> {
    let mut textures = HashMap::new();
    for customer_type in &data.customer_types {
        let path = format!("assets/images/characters/{}.png", customer_type.id);
        if let Ok(texture) =
            load_texture_from_pack_or_file(asset_pack, &path, FilterMode::Linear).await
        {
            textures.insert(customer_type.id.clone(), texture);
        }
    }

    textures
}

/// The interim environment sprite sheet (floors, appliances, furniture, decor,
/// chef). Nearest filtering keeps the pixel art crisp when scaled.
pub async fn load_interior_sheet(asset_pack: Option<&AssetPack>) -> Option<Texture2D> {
    load_texture_from_pack_or_file(asset_pack, INTERIOR_SHEET_PATH, FilterMode::Nearest)
        .await
        .ok()
}

pub async fn load_title_texture(asset_pack: Option<&AssetPack>) -> Option<Texture2D> {
    for path in TITLE_TEXTURE_PATHS {
        if let Ok(texture) =
            load_texture_from_pack_or_file(asset_pack, path, FilterMode::Linear).await
        {
            return Some(texture);
        }
    }

    None
}
