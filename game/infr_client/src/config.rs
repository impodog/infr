use bevy::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::LazyLock;

/// Creates a wrapper, deserializing either path or direct input.
macro_rules! path_wrapper {
    ($wrapper:ident, $inner:ty) => {
        path_wrapper!($wrapper, $inner, toml);
    };
    ($wrapper:ident, $inner:ty, $module: ident) => {
        /// Wrapper that deserializes from either a file path (string) or direct value
        #[derive(Debug, Clone, Default)]
        pub struct $wrapper($inner, pub PathBuf);
        impl Deref for $wrapper {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl DerefMut for $wrapper {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl<'de> Deserialize<'de> for $wrapper {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(untagged)]
                enum FileOrDirect {
                    Path(String),
                    Direct($inner),
                }
                match FileOrDirect::deserialize(deserializer)? {
                    FileOrDirect::Path(path_str) => {
                        // Read file contents
                        let path = std::path::Path::new(&path_str);
                        let contents = std::fs::read_to_string(path).map_err(|e| {
                            serde::de::Error::custom(format!(
                                "failed to read file '{}': {}",
                                path.display(),
                                e
                            ))
                        })?;
                        let value = toml::from_str::<$inner>(&contents).map_err(|e| {
                            serde::de::Error::custom(format!(
                                "failed to deserialize TOML from file '{}': {}",
                                path.display(),
                                e
                            ))
                        })?;
                        Ok($wrapper(
                            value,
                            path.canonicalize().unwrap().parent().unwrap().into(),
                        ))
                    }
                    FileOrDirect::Direct(value) => Ok($wrapper(value, Default::default())),
                }
            }
        }
    };
}

#[derive(Deserialize, Debug, Clone)]
pub struct DisplayConfig {
    pub tile_size: (u32, u32),
    pub window_size: (u32, u32),
    pub fullscreen: bool,
}
path_wrapper!(DisplayConfigWrapper, DisplayConfig);

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            tile_size: (32, 32),
            window_size: (1920, 1080),
            fullscreen: false,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct LevelPack {
    pub levels: Vec<PathBuf>,
}
path_wrapper!(LevelPackWrapper, LevelPack);

impl LevelPackWrapper {
    pub fn find(&self, name: &str) -> Option<PathBuf> {
        for path in self.0.levels.iter() {
            let target = self.1.join(path).join(name).with_extension("toml");
            if target.exists() {
                return Some(target);
            }
        }
        None
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub refresh_interval: u64,
}
path_wrapper!(ServerConfigWrapper, ServerConfig);

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:4321".to_owned(),
            refresh_interval: 20,
        }
    }
}

/// Game instance and game play related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ClientConfig {
    pub framerate: u32,
    #[serde(skip)]
    pub frame_duration: std::time::Duration,
    pub movement_velocity: f32,
    pub teleport_velocity: f32,
}
path_wrapper!(ClientConfigWrapper, ClientConfig);

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            framerate: 60,
            frame_duration: std::time::Duration::from_secs_f32(1.0 / 60.0),
            movement_velocity: 5.0,
            teleport_velocity: 20.0,
        }
    }
}

/// One sprite atlas that will be played repeatedly in the game.
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Default, Deserialize, Debug, Clone)]
pub struct SpriteConfig {
    /// Any child configurations that will be merged into this config.
    #[serde(default)]
    pub include: Vec<PathBuf>,
    /// Map from aliases to the actual sprite.
    #[serde(default)]
    pub map: HashMap<String, Vec<SpriteAtlas>>,
    /// Defines tinting style aliases.
    #[serde(default)]
    pub tinting: HashMap<String, Color>,
}

impl SpriteConfig {
    pub fn load(path: PathBuf) -> Self {
        let base_path = path
            .parent()
            .expect("Config file should be a file with a parent directory");
        let mut config: SpriteConfig = match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    info!("Sprite config loaded at {path:?}");
                    config
                }
                Err(err) => {
                    panic!("Failed to parse sprite config: {err}");
                }
            },
            Err(err) => {
                error!("Unable to read sprite config {path:?}(skipped): {err}");
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
                    error!("Unable to read configurated image path: {new_path:?}(skipped)");
                }
            }
            *list = new_list;
        }
        for sub_path in config.include.drain(..) {
            let sub_path = base_path.join(&sub_path);
            let SpriteConfig { map, tinting, .. } = SpriteConfig::load(sub_path);
            config.map.extend(map.into_iter());
            config.tinting.extend(tinting.into_iter())
        }
        config
    }
}

/// Serde entry point for the config file.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub display: DisplayConfigWrapper,
    #[serde(default)]
    pub levels: HashMap<String, LevelPackWrapper>,
    #[serde(default)]
    pub server: ServerConfigWrapper,
    #[serde(default)]
    pub client: ClientConfigWrapper,
    /// This is the field provided by the config file, serves as the entry point for `Self::sprites`.
    #[serde(default = "default_sprite_path")]
    pub sprites_path: PathBuf,
    #[serde(skip)]
    pub sprites: SpriteConfig,
}
fn default_sprite_path() -> PathBuf {
    PathBuf::from("assets/sprites/config.json")
}

impl ConfigFile {
    pub fn find_level(&self, pack: &str, name: &str) -> Option<PathBuf> {
        let pack = self.levels.get(pack)?;
        pack.find(name)
    }
}

pub static CONFIG: LazyLock<ConfigFile> =
    LazyLock::new(|| match std::fs::read_to_string("client.toml") {
        Ok(content) => {
            let mut config: ConfigFile =
                toml::from_str(content.as_str()).expect("client.toml failed to parse");
            config.client.frame_duration =
                std::time::Duration::from_secs_f32(1.0 / config.client.framerate as f32);
            config.sprites = SpriteConfig::load(config.sprites_path.clone());
            config
        }
        Err(err) => {
            warn!("Unable to find client.toml: {err}, using default config");
            ConfigFile::default()
        }
    });
