use crate::Font;
use anyhow::Result;
use bevy_asset::{AssetLoader,AssetStorage};
use std::path::Path;

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader<Font> for FontLoader {
    fn from_storage(&self, _asset_path: &Path, storage: AssetStorage) -> Result<Font> {
        Ok(Font::try_from_bytes(storage.into_vec ())?)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["ttf"];
        EXTENSIONS
    }
}
