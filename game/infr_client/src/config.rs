//! Loads client configuration.

use bevy::prelude::UVec2;

use serde::{Deserialize, Serialize};

use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

/// Config loaded at client startup, containing paths to other configs.
#[derive(Serialize, Deserialize, Debug)]
pub struct StartupConfig {
    /// The address the client should connect to.
    pub address: String,
    #[serde(skip)]
    base_url: Option<reqwest::Url>,
    pub refresh_interval: u32,
    pub sprite_config: PathBuf,
    pub window_size: (u32, u32),
    pub pixel_grid: (u32, u32),
    pub tile_size: (u32, u32),
    pub fullscreen: bool,
}
impl StartupConfig {
    /// Returns a url with the given path under the configured host.
    pub fn url(&self, path: &str) -> reqwest::Url {
        let mut url = self.base_url.as_ref().expect("You should only use this struct by the static STARTUP_CONFIG, which would initialize base_url").clone();
        url.set_path(path);
        url
    }
}
impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:4321".into(),
            base_url: None,
            refresh_interval: 20,
            sprite_config: "assets/sprites/config.json".into(),
            window_size: (1920, 1080),
            pixel_grid: (800, 450),
            tile_size: (32, 32),
            fullscreen: false,
        }
    }
}

pub const STARTUP_CONFIG_PATH: &str = "client.toml";
pub static STARTUP_CONFIG: LazyLock<StartupConfig> = LazyLock::new(|| {
    let mut config: StartupConfig = match std::fs::read_to_string(STARTUP_CONFIG_PATH) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => {
                log::info!("Client startup config loaded: {config:?}");
                config
            }
            Err(err) => {
                panic!("Failed to parse {:?}: {err}", STARTUP_CONFIG_PATH);
            }
        },
        Err(_) => {
            log::info!(
                "Unable to read {:?}; Using default configuration...",
                STARTUP_CONFIG_PATH
            );
            Default::default()
        }
    };
    config.base_url = Some(
        reqwest::Url::parse(&format!("http://{}", config.address))
            .expect("Failed to parse base URL"),
    );
    config
});

/// One sprite atlas that will be played repeatedly in the game.
#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteAtlas {
    pub path: PathBuf,
    #[serde(default)]
    pub offset: (u32, u32),
    #[serde(default = "return_default_size")]
    pub size: (u32, u32),
    #[serde(default = "return_1")]
    pub count: u32,
    /// Milliseconds between switching frames.
    #[serde(default = "return_1000")]
    pub interval: u32,
}
const fn return_default_size() -> (u32, u32) {
    (32, 32)
}
const fn return_1() -> u32 {
    1
}
const fn return_1000() -> u32 {
    1000
}

/// Configuration for sprites in the game.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct SpriteConfig {
    /// Any child configurations that will be merged into this config.
    #[serde(default)]
    pub include: Vec<PathBuf>,
    /// Map from aliases to the actual sprite.
    #[serde(default)]
    pub map: HashMap<String, Vec<SpriteAtlas>>,
}

impl SpriteConfig {
    pub fn load(path: PathBuf) -> Self {
        let base_path = path
            .parent()
            .expect("Config file should be a file with a parent directory");
        let mut config: SpriteConfig = match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    log::info!("Sprite config loaded at {path:?}");
                    config
                }
                Err(err) => {
                    panic!("Failed to parse sprite config: {err}");
                }
            },
            Err(err) => {
                log::error!("Unable to read sprite config {path:?}(skipped): {err}");
                Default::default()
            }
        };
        for list in config.map.values_mut() {
            let mut new_list = Vec::new();
            for mut atlas in list.drain(..) {
                let new_path = base_path.join(&atlas.path);
                if let Ok(new_path) = new_path.canonicalize() {
                    atlas.path = new_path;
                    new_list.push(atlas);
                } else {
                    log::error!("Unable to read configurated image path: {new_path:?}(skipped)");
                }
            }
            *list = new_list;
        }
        for sub_path in config.include.drain(..) {
            let sub_path = base_path.join(&sub_path);
            config
                .map
                .extend(SpriteConfig::load(sub_path).map.into_iter());
        }
        config
    }
}

pub static SPRITE_CONFIG: LazyLock<SpriteConfig> =
    LazyLock::new(|| SpriteConfig::load(STARTUP_CONFIG.sprite_config.clone()));

pub static TILE_SIZE: LazyLock<UVec2> =
    LazyLock::new(|| UVec2::new(STARTUP_CONFIG.tile_size.0, STARTUP_CONFIG.tile_size.1));
