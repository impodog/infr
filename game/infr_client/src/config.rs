use bevy::prelude::*;

use serde::Deserialize;
use std::ops::Deref;

/// Creates a wrapper, deserializing either path or direct input.
macro_rules! path_wrapper {
    ($wrapper:ident, $inner:ty) => {
        /// Wrapper that deserializes from either a file path (string) or direct value
        #[derive(Debug, Clone)]
        pub struct $wrapper($inner);
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
                        Ok($wrapper(value))
                    }
                    FileOrDirect::Direct(value) => Ok($wrapper(value)),
                }
            }
        }
    };
}

#[derive(Deserialize, Debug, Clone)]
pub struct DisplayConfig {
    pub tile_size: UVec2,
    pub window_size: UVec2,
}
path_wrapper!(DisplayConfigWrapper, DisplayConfig);

/// Serde entry point for the config file.
#[derive(Deserialize)]
pub struct ConfigFile {
    pub display: DisplayConfigWrapper,
}
