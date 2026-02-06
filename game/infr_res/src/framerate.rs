use bevy::prelude::*;
use infr_client::prelude::*;

pub(crate) fn control_framerate(world: &mut World) {
    let target_delta = config::CONFIG.client.frame_duration;
    let time = world.get_resource::<Time>().unwrap();
    let delta = time.delta();
    if delta < target_delta {
        let remain = target_delta - delta;
        std::thread::sleep(remain);
    }
}
