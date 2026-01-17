use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*};
use infr_client::config::SPRITE_CONFIG;

#[derive(Default, Debug, Clone, Component)]
#[require(AnimationClock, Sprite)]
pub struct Animation {
    pub name: String,
    pub size: Vec2,
}
#[derive(Default, Debug, Component)]
pub(crate) struct AnimationClock {
    timer: Timer,
    total: usize,
}

#[derive(Resource, Default, Debug)]
pub(crate) struct AnimationAtlasHandles(HashMap<String, Handle<TextureAtlasLayout>>);

fn convert_to_sprite(
    name: String,
    asset_server: &AssetServer,
    atlas: &infr_client::config::SpriteAtlas,
    layouts: &mut Assets<TextureAtlasLayout>,
    atlas_handles: &mut AnimationAtlasHandles,
) -> Sprite {
    let image: Handle<Image> = asset_server.load(atlas.path.clone());
    let layout = atlas_handles
        .0
        .entry(name)
        .or_insert_with(|| {
            layouts.add(TextureAtlasLayout::from_grid(
                UVec2 {
                    x: atlas.size.0,
                    y: atlas.size.1,
                },
                atlas.count,
                1,
                None,
                Some(UVec2 {
                    x: atlas.offset.0,
                    y: atlas.offset.1,
                }),
            ))
        })
        .clone();
    Sprite {
        image,
        texture_atlas: Some(TextureAtlas { layout, index: 0 }),
        ..Default::default()
    }
}

pub(crate) fn modify_animation(
    mut query: Query<(&Animation, &mut Sprite, &mut AnimationClock), Changed<Animation>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas_handles: ResMut<AnimationAtlasHandles>,
) {
    let default_sprite = SPRITE_CONFIG
        .map
        .get("Empty")
        .and_then(|sprites| sprites.first())
        .map(|sprite| {
            convert_to_sprite(
                "Empty".to_owned(),
                &asset_server,
                sprite,
                &mut layouts,
                &mut atlas_handles,
            )
        })
        .unwrap_or_default();
    query
        .iter_mut()
        .for_each(|(animation, mut sprite, mut clock)| {
            let Some(config) = SPRITE_CONFIG.map.get(&animation.name) else {
                *sprite = default_sprite.clone();
                return;
            };
            if config.is_empty() {
                *sprite = default_sprite.clone();
                return;
            }
            let atlas = &config[rand::random_range(0..config.len())];
            *sprite = convert_to_sprite(
                animation.name.clone(),
                &asset_server,
                atlas,
                &mut layouts,
                &mut atlas_handles,
            );
            sprite.custom_size = Some(animation.size);
            clock.timer = Timer::new(
                Duration::from_millis(atlas.interval as u64),
                TimerMode::Repeating,
            );
            clock.total = atlas.count as usize;
        });
}

pub(crate) fn tick_animation(
    mut query: Query<(&mut Sprite, &mut AnimationClock)>,
    time: Res<Time>,
) {
    query.par_iter_mut().for_each(|(mut sprite, mut clock)| {
        if clock.timer.tick(time.delta()).just_finished() {
            let Some(texture_atlas) = &mut sprite.texture_atlas else {
                return;
            };
            texture_atlas.index = (texture_atlas.index + 1) % clock.total;
        }
    });
}
