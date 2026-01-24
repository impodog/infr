use bevy::prelude::*;
use infr_client::config::TILE_SIZE;

#[derive(Component, Debug, Clone)]
#[require(infr_res::Animation, Position, Movement, Offset)]
pub struct Object {
    pub id: u32,
    /// The session that the object is tied to.
    pub tied_session: u32,
}

/// Stores the fraction position of 32x32 grid of an object.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Position(pub Vec2);

/// Offsets the object position, making these objects appear off-screen.
/// Objects with non-zero offset do not contribute to screen size.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Offset(pub Vec2);

/// Defines the current animation playing. Players cannot input until all movement animations are played.
#[derive(Component, Debug, Clone, Copy, Default)]
pub enum Movement {
    #[default]
    Stop,
    Swipe(Position),
    Teleport(Position),
    Remove,
}

/// Calculated each time to find the current boundaries of all objects.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentMapSize(pub Rect);

pub(crate) fn update_object_transform(
    mut query: Query<(&mut Transform, &Position, &Offset)>,
    mut resolution: ResMut<infr_render::VirtualResolution>,
    mut commands: Commands,
) {
    let mut x_min = f32::INFINITY;
    let mut y_min = f32::INFINITY;
    let mut x_max = f32::NEG_INFINITY;
    let mut y_max = f32::NEG_INFINITY;
    query.iter().for_each(|(_, position, offset)| {
        if offset.0.length() < 1e-5 {
            x_min = x_min.min(position.0.x);
            x_max = x_max.max(position.0.x);
            y_min = y_min.min(position.0.y);
            y_max = y_max.max(position.0.y);
        }
    });
    commands.insert_resource(CurrentMapSize(Rect::new(x_min, y_min, x_max, y_max)));
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
                (position.0.x - center_x + offset.0.x) * TILE_SIZE.x as f32,
                (position.0.y - center_y + offset.0.y) * TILE_SIZE.y as f32,
                1.0,
            );
        });
}

#[derive(Message, Debug, Clone)]
pub struct SpawnObjectMessage {
    pub id: u32,
    pub offset: Vec2,
}

pub(crate) fn spawn_object(
    mut reader: MessageReader<SpawnObjectMessage>,
    mut commands: Commands,
    map: Res<crate::Map>,
) {
    for message in reader.read() {
        let object = if let Some(object) = map.client.objects.get(&message.id) {
            object
        } else {
            return;
        };
        commands.spawn((
            Object {
                id: message.id,
                tied_session: map.client.session_id,
            },
            Offset(message.offset),
            Position(Vec2::new(object.coord.0 as f32, object.coord.1 as f32)),
            infr_res::Animation::new_or_default(object.sprite.clone()),
        ));
    }
}
