use bevy::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Creates a wrapper, deserializing either path or direct input.
macro_rules! path_wrapper {
    ($wrapper:ident, $inner:ty) => {
        /// Wrapper that deserializes from either a file path (string) or direct value
        #[derive(Debug, Clone, Default)]
        pub struct $wrapper($inner, pub PathBuf);
        impl Deref for $wrapper {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
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
}
path_wrapper!(DisplayConfigWrapper, DisplayConfig);
impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            tile_size: (32, 32),
            window_size: (1920, 1080),
        }
    }
}
#[derive(Deserialize, Debug, Clone, Default)]
pub struct LevelPack(pub Vec<PathBuf>);
path_wrapper!(LevelPackWrapper, LevelPack);

impl LevelPackWrapper {
    pub fn find(&self, name: &str) -> Option<PathBuf> {
        for path in self.0.0.iter() {
            let target = self.1.join(path).join(name).with_extension("toml");
            if target.exists() {
                return Some(target);
            }
        }
        None
    }
}

/// Serde entry point for the config file.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub display: DisplayConfigWrapper,
    pub levels: HashMap<String, LevelPackWrapper>,
}

impl ConfigFile {
    pub fn find_level(&self, pack: &str, name: &str) -> Option<PathBuf> {
        let pack = self.levels.get(pack)?;
        pack.find(name)
    }
}

pub static CONFIG: LazyLock<ConfigFile> =
    LazyLock::new(|| match std::fs::read_to_string("client.toml") {
        Ok(content) => toml::from_str(content.as_str()).expect("client.toml failed to parse"),
        Err(err) => {
            warn!("Unable to find client.toml: {err}, using default config");
            ConfigFile::default()
        }
    });
