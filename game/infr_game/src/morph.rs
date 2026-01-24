use bevy::prelude::*;

/// Request to morph the current session to a nearby one.
#[derive(Message, Debug)]
pub(crate) struct MorphSessionMessage {
    /// The offset of target objects.
    pub(crate) offset: Vec2,
}

pub(crate) fn spawn_matching_objects(
    mut reader: MessageReader<MorphSessionMessage>,
    mut writer: MessageWriter<crate::SpawnObjectMessage>,
    map: Res<crate::Map>,
) {
    for message in reader.read() {
        for id in map.client.objects.keys().copied() {
            writer.write(crate::SpawnObjectMessage {
                id,
                offset: message.offset,
            });
        }
    }
}

pub(crate) fn move_camera(
    mut camera: Query<(&mut Transform), With<infr_render::PixelCamera>>,
    mut state: ResMut<NextState<crate::GameState>>,
) -> Result<()> {
    let (transform) = camera.single_mut()?;
    // todo: Actually move camera.
    state.set(crate::GameState::Level);
    Ok(())
}
