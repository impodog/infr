use bevy::prelude::*;

use infr_client::{Position, config};

/// Marks a background tile that marks the game board.
#[derive(Component)]
#[require(Position, crate::Animation)]
pub struct BackgroundTile;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundRange {
    x: (i32, i32),
    y: (i32, i32),
}
impl Default for BackgroundRange {
    fn default() -> Self {
        Self {
            x: (0, -1),
            y: (0, -1),
        }
    }
}

pub(crate) fn update_background_tile(
    commands: ParallelCommands,
    resolution: Res<crate::camera::VirtualResolution>,
    mut range: ResMut<BackgroundRange>,
    q_tiles: Query<(Entity, &Position), With<BackgroundTile>>,
    state: Res<State<infr_client::MapState>>,
    center: Res<infr_client::PositionCenter>,
) {
    // Prevent adding background tiles when there isn't any tiles.
    if *state.get() == infr_client::MapState::Loading {
        return;
    }
    let x_length = resolution.x as f32 / config::CONFIG.display.tile_size.0 as f32 * 0.5;
    let y_length = resolution.y as f32 / config::CONFIG.display.tile_size.1 as f32 * 0.5;
    let new_range = BackgroundRange {
        x: (
            (center.center.x - x_length - 0.5).floor() as i32,
            (center.center.x + x_length + 0.5).ceil() as i32,
        ),
        y: (
            (center.center.y - y_length - 0.5).floor() as i32,
            (center.center.y + y_length + 0.5).ceil() as i32,
        ),
    };
    if *range != new_range {
        // info!("Background range from {range:?} to {new_range:?}");
        q_tiles.par_iter().for_each(|(entity, position)| {
            let x = position.x.round() as i32;
            let y = position.y.round() as i32;
            if x < new_range.x.0 || x > new_range.x.1 || y < new_range.y.0 || y > new_range.y.1 {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        });
        commands.command_scope(|mut commands| {
            for x in new_range.x.0..=new_range.x.1 {
                for y in new_range.y.0..=new_range.y.1 {
                    // A new background tile.
                    if x < range.x.0 || x > range.x.1 || y < range.y.0 || y > range.y.1 {
                        commands.spawn((
                            BackgroundTile,
                            crate::Animation::new("Back"),
                            Position(Vec2::new(x as f32, y as f32)),
                            Transform::from_translation(Vec3::new(0.0, 0.0, -14.37)),
                        ));
                    }
                }
            }
        });
        *range = new_range;
    }
}

pub(crate) fn reset_background_range(
    mut reader: MessageReader<infr_client::LoadMap>,
    mut range: ResMut<BackgroundRange>,
) {
    if reader.read().next().is_some() {
        *range = Default::default();
    }
}

/// Marks the background animation that plays in the level.
#[derive(Component)]
#[require(crate::Animation)]
pub struct BackgroundPicture;

/// This is called only once at startup, to reuse the background picture.
pub(crate) fn setup_background(mut commands: Commands) {
    commands.spawn((BackgroundPicture, crate::camera::DISPLAY_RENDER_LAYER));
}

pub(crate) fn modify_background(
    meta: Res<infr_client::LevelMetadata>,
    mut q_background: Query<&mut crate::Animation, With<BackgroundPicture>>,
) -> Result<()> {
    if meta.is_changed() {
        let mut animation = q_background.single_mut()?;
        if let Some(ref background) = meta.background {
            animation.name = background.to_owned();
            animation.size = Vec2::new(
                config::CONFIG.display.window_size.0 as f32,
                config::CONFIG.display.window_size.1 as f32,
            );
        } else {
            animation.name = "".to_owned();
            animation.size = Vec2::default();
        }
    }
    Ok(())
}
