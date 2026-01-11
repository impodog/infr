//! Loads client configuration.

use serde::{Deserialize, Serialize};

use std::{path::PathBuf, sync::LazyLock};

/// Config loaded at client startup, containing paths to other configs.
#[derive(Serialize, Deserialize, Debug)]
pub struct StartupConfig {
    /// The address the client should connect to.
    pub address: std::net::SocketAddr,
    pub refresh_interval: u32,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            address: std::net::SocketAddr::new("127.0.0.1".parse().unwrap(), 4321),
            refresh_interval: 20,
        }
    }
}

pub const STARTUP_CONFIG_PATH: &'static str = "client.toml";
pub static STARTUP_CONFIG: LazyLock<StartupConfig> =
    LazyLock::new(|| match std::fs::read_to_string(STARTUP_CONFIG_PATH) {
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
    });
