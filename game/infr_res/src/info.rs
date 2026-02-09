//! This file shows level related information.

use bevy::prelude::*;
use std::time::Duration;

/// Marks the title text node.
#[derive(Component)]
#[require(infr_client::SessionOnly, MorphEffect)]
pub struct TitleMarker;

#[derive(Component, Debug, Clone, Default)]
pub struct MorphEffect {
    pub start_time: Duration,
    pub total_time: Duration,
}

pub(crate) fn play_morph_effect(
    commands: ParallelCommands,
    mut q_morph_effect: Query<(
        Entity,
        &MorphEffect,
        Option<&mut BackgroundColor>,
        Option<&mut TextColor>,
    )>,
    time: Res<Time>,
) {
    q_morph_effect.par_iter_mut().for_each(
        |(entity, morph_effect, background_color, text_color)| {
            let x = (time.elapsed() - morph_effect.start_time).as_secs_f32()
                / morph_effect.total_time.as_secs_f32()
                * 2.0;
            let alpha = if x <= 1.0 {
                EaseFunction::SmootherStepIn.sample(x)
            } else {
                EaseFunction::CircularOut.sample(2.0 - x)
            };
            if let Some(alpha) = alpha {
                if let Some(mut background_color) = background_color {
                    *background_color = background_color.0.with_alpha(alpha).into();
                }
                if let Some(mut text_color) = text_color {
                    *text_color = text_color.with_alpha(alpha).into();
                }
            } else {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        },
    );
}

pub(crate) fn show_title(
    mut commands: Commands,
    time: Res<Time>,
    meta: Res<infr_client::LevelMetadata>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        TitleMarker,
        Text2d::new(meta.title.clone()),
        TextColor(Color::Srgba(Srgba::WHITE)),
        TextFont {
            font: asset_server.load(infr_client::config::CONFIG.fonts.text_font.clone()),
            font_size: 100.0,
            ..Default::default()
        },
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        Transform::from_translation(Vec3::new(0.0, 0.0, 14.37)),
        MorphEffect {
            start_time: time.elapsed(),
            total_time: Duration::from_secs(2),
        },
        crate::camera::DISPLAY_RENDER_LAYER,
    ));
}
