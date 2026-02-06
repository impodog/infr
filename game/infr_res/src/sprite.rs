use bevy::prelude::*;
use infr_client::prelude::*;
use std::{collections::HashMap, time::Duration};

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

impl Animation {
    /// Creates an animation with default tile size.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: Vec2::new(
                config::CONFIG.display.tile_size.0 as f32,
                config::CONFIG.display.tile_size.1 as f32,
            ),
        }
    }

    /// Creates an animation if the image is available, or returns the default value.
    pub fn new_or_default(name: Option<String>) -> Self {
        if let Some(name) = name {
            Self::new(name)
        } else {
            Self::default()
        }
    }
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
    mut query: Query<
        (
            &Animation,
            &mut Sprite,
            &mut AnimationClock,
            &mut Visibility,
        ),
        Changed<Animation>,
    >,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas_handles: ResMut<AnimationAtlasHandles>,
) {
    let default_sprite = config::CONFIG
        .sprites
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
        .for_each(|(animation, mut sprite, mut clock, mut visibility)| {
            if animation.name.is_empty() {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Inherited;
            }
            let Some(config) = config::CONFIG.sprites.map.get(&animation.name) else {
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

macro_rules! try_return {
    ($name: expr) => {{
        let name = $name;
        if config::CONFIG.sprites.map.get(&name).is_some() {
            return name;
        }
    }};
}

fn select_object_animation_helper(nature: &str, direction: transfer::Direction) -> String {
    let (nature, suffix): (&str, &'static [&'static str]) = if let Some(first) =
        nature.chars().next()
        && !first.is_alphanumeric()
    {
        let rest = &nature[first.len_utf8()..];
        match first {
            '$' => (rest, &["Ins"]),
            '@' => (rest, &["Sym"]),
            '%' => (rest, &["Sym", "Ins"]),
            '=' => (rest, &["Ops", "Sym"]),
            _ => {
                error!("Unknown nature prefix: {first}");
                (rest, &[])
            }
        }
    } else {
        (nature, &[])
    };
    let dir = match direction {
        transfer::Direction::RIGHT => "Right",
        transfer::Direction::UP => "Up",
        transfer::Direction::LEFT => "Left",
        transfer::Direction::DOWN => "Down",
        _ => "Unknown",
    };
    for suffix in suffix
        .iter()
        .map(std::ops::Deref::deref)
        .chain(std::iter::once(""))
    {
        for direction in [dir, ""] {
            try_return!(format!("{}{}{}", nature, suffix, direction));
        }
    }
    "Empty".to_owned()
}

pub(crate) fn select_object_animation(
    commands: ParallelCommands,
    q_object: Query<(Entity, &infr_client::ObjectState), Changed<infr_client::ObjectState>>,
) {
    q_object.par_iter().for_each(|(entity, object_state)| {
        let name = select_object_animation_helper(&object_state.nature, object_state.direction);
        commands.command_scope(|mut commands| {
            commands.entity(entity).insert(Animation {
                name,
                size: Vec2::new(
                    config::CONFIG.display.tile_size.0 as f32,
                    config::CONFIG.display.tile_size.1 as f32,
                ),
            });
        });
    });
}
