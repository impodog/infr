use bevy::{prelude::*, window::WindowResolution};
use infr_client::config::STARTUP_CONFIG;

pub struct WindowMarker;

#[derive(Debug, Resource)]
pub struct WindowTitle(pub String);
impl Default for WindowTitle {
    fn default() -> Self {
        Self("Infr".to_owned())
    }
}

pub(crate) fn setup_window(mut query: Query<&mut Window>, title: Res<WindowTitle>) -> Result<()> {
    let mut window = query.single_mut()?;
    if STARTUP_CONFIG.fullscreen {
        window.mode = bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    }
    window.title = title.0.clone();
    window.resolution =
        WindowResolution::new(STARTUP_CONFIG.window_size.0, STARTUP_CONFIG.window_size.1);

    Ok(())
}

pub(crate) fn update_window(mut query: Query<&mut Window>, title: Res<WindowTitle>) -> Result<()> {
    let mut window = query.single_mut()?;
    if title.is_changed() {
        window.title = title.0.clone();
    }
    Ok(())
}
