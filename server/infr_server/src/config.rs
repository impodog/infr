use serde::{Deserialize, Serialize};

use std::sync::LazyLock;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartupConfig {
    /// The address to run the server on.
    pub address: std::net::SocketAddr,
    pub refresh_interval: u64,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            address: std::net::SocketAddr::new("127.0.0.1".parse().unwrap(), 4321),
            refresh_interval: 30,
        }
    }
}

pub static STARTUP_CONFIG: LazyLock<StartupConfig> =
    LazyLock::new(|| match std::fs::read_to_string("config.toml") {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => {
                log::info!("Startup config loaded: {config:?}");
                config
            }
            Err(err) => {
                panic!("Failed to parse config.toml: {err}");
            }
        },
        Err(_) => {
            log::info!("Unable to read config.toml; Using default configuration...");
            StartupConfig::default()
        }
    });
