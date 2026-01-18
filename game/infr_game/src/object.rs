use bevy::prelude::*;
use std::sync::LazyLock;

pub static TILE_SIZE: LazyLock<UVec2> = LazyLock::new(|| {
    UVec2::new(
        infr_client::config::STARTUP_CONFIG.tile_size.0,
        infr_client::config::STARTUP_CONFIG.tile_size.1,
    )
});

#[derive(Component, Debug, Clone)]
#[require(infr_res::Animation, Position, Movement, Offset)]
pub struct Object {
    pub id: u32,
}

/// Stores the fraction position of 32x32 grid of an object.
#[derive(Component, Debug, Clone, Default)]
pub struct Position(pub Vec2);

/// Offsets the object position, making these objects appear off-screen.
/// Objects with non-zero offset do not contribute to screen size.
#[derive(Component, Debug, Clone, Default)]
pub struct Offset(pub Vec2);

/// Defines the current animation playing. Players cannot input until all movement animations are played.
#[derive(Component, Debug, Clone, Default)]
pub enum Movement {
    #[default]
    Stop,
    Swipe(Position),
    Teleport(Position),
    Remove,
}

pub(crate) fn update_object_transform(
    mut query: Query<(&mut Transform, &Position, &Offset)>,
    mut resolution: ResMut<infr_render::VirtualResolution>,
) {
    let mut x_min = f32::INFINITY;
    let mut y_min = f32::INFINITY;
    let mut x_max = -f32::INFINITY;
    let mut y_max = -f32::INFINITY;
    query.iter().for_each(|(_, position, offset)| {
        if offset.0.length() < 1e-5 {
            x_min = x_min.min(position.0.x);
            x_max = x_max.max(position.0.x);
            y_min = y_min.min(position.0.y);
            y_max = y_max.max(position.0.y);
        }
    });
    let x = (x_max - x_min + 1.0) * TILE_SIZE.x as f32;
    let y = (y_max - y_min + 1.0) * TILE_SIZE.y as f32;
    let side_length = (x / 16.0).max(y / 9.0);
    let new_resolution = (
        (side_length * 16.0).ceil() as u32,
        (side_length / 9.0).ceil() as u32,
    );
    let prev_resolusion = *resolution;
    if new_resolution.0 > prev_resolusion.x {
        *resolution = infr_render::VirtualResolution {
            x: new_resolution.0,
            y: new_resolution.1,
        };
    }

    let center_x = (x_min + x_max) / 2.0;
    let center_y = (y_min + y_max) / 2.0;
    query
        .par_iter_mut()
        .for_each(|(mut transform, position, offset)| {
            transform.translation = Vec3::new(
                (position.0.x - center_x) * TILE_SIZE.x as f32 + offset.0.x,
                (position.0.y - center_y) * TILE_SIZE.y as f32 + offset.0.y,
                1.0,
            );
        });
}
