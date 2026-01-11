use serde::{Deserialize, Serialize};

use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartupConfig {
    /// The address to run the server on.
    pub address: std::net::SocketAddr,
    pub refresh_interval: u64,
    pub script_config: PathBuf,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            address: std::net::SocketAddr::new("127.0.0.1".parse().unwrap(), 4321),
            refresh_interval: 30,
            script_config: PathBuf::from("scripts/scripts.toml"),
        }
    }
}

pub const STARTUP_CONFIG_PATH: &'static str = "server.toml";

pub static STARTUP_CONFIG: LazyLock<StartupConfig> =
    LazyLock::new(|| match std::fs::read_to_string(STARTUP_CONFIG_PATH) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => {
                log::info!("Startup config loaded: {config:?}");
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
            StartupConfig::default()
        }
    });

/// Maps scripts names to their specific locations.
///
/// This may be done in two ways: 1. explicitly write the map; 2. add included paths where the program will automatically find the lua file.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ScriptConfig {
    pub scripts: HashMap<String, PathBuf>,
    pub included_paths: Vec<PathBuf>,
}

impl ScriptConfig {
    /// Looks for a script with the name.
    pub fn query(&self, name: &str) -> Option<PathBuf> {
        if let Some(path) = self.scripts.get(name) {
            Some(path.clone())
        } else {
            for path in &self.included_paths {
                let mut script_path = path.clone();
                script_path.push(format!("{}.lua", name));
                if script_path.exists() {
                    return Some(script_path);
                }
            }
            None
        }
    }
}

pub static SCRIPT_CONFIG: LazyLock<ScriptConfig> = LazyLock::new(|| {
    let base_path = STARTUP_CONFIG
        .script_config
        .parent()
        .expect("Expected script config path to be a file(with a parent dir)");
    let mut config = match std::fs::read_to_string(&STARTUP_CONFIG.script_config) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => {
                log::info!("Script config loaded: {config:?}");
                config
            }
            Err(err) => {
                panic!("Failed to parse {:?}: {err}", STARTUP_CONFIG.script_config);
            }
        },
        Err(_) => {
            log::info!(
                "Unable to read {:?}; Using default configuration...",
                STARTUP_CONFIG.script_config
            );
            let mut config = ScriptConfig::default();
            config.included_paths.push(".".into());
            config
        }
    };
    for path in config.scripts.values_mut() {
        let new_path = base_path.join(&path);
        *path = new_path;
    }
    for path in config.included_paths.iter_mut() {
        let new_path = base_path.join(&path);
        *path = new_path;
    }
    config
});
