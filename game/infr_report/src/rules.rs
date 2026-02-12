use std::{collections::BTreeSet, time::Duration};

use bevy::prelude::*;

use infr_client::prelude::*;

/// Determines what color the rule highlighter is.
#[derive(Debug, Clone, Copy, Default)]
pub enum RuleStatus {
    #[default]
    Active,
    /// The start time of error display.
    Error(Duration),
}

/// Component to circle out rules with specific status.
#[derive(Component, Debug, Clone, Default)]
#[require(infr_client::SessionOnly, infr_client::Position, MeshMaterial2d<ColorMaterial>, Mesh2d)]
pub struct RuleHighlighter {
    pub status: RuleStatus,
    pub position: (Coord, Coord),
}

#[derive(Resource, Debug, Default)]
pub(crate) struct HighlighterMaterials {
    active: Vec<Handle<ColorMaterial>>,
    error: Vec<Handle<ColorMaterial>>,
}
impl HighlighterMaterials {
    fn random_active(&self) -> Handle<ColorMaterial> {
        self.active[rand::random_range(0..self.active.len())].clone()
    }

    fn random_error(&self) -> Handle<ColorMaterial> {
        self.error[rand::random_range(0..self.error.len())].clone()
    }
}

pub(crate) fn setup_materials(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    const TOTAL_COLORS: i32 = 8;
    let mut result = HighlighterMaterials::default();
    for i in 0..TOTAL_COLORS {
        result
            .active
            .push(materials.add(ColorMaterial::from_color(Color::hsla(
                120.0 + 120.0 * i as f32 / TOTAL_COLORS as f32,
                0.8,
                0.9,
                0.5,
            ))));
    }
    for i in 0..TOTAL_COLORS {
        result
            .error
            .push(materials.add(ColorMaterial::from_color(Color::hsla(
                -60.0 + 120.0 * i as f32 / TOTAL_COLORS as f32,
                0.9,
                1.0,
                0.7,
            ))));
    }
    commands.insert_resource(result);
}

pub(crate) fn update_highlighters(
    mut commands: Commands,
    q_highlighter: Query<(Entity, &RuleHighlighter)>,
    rules: Res<infr_client::RuleRanges>,
) {
    if !rules.is_changed() {
        return;
    }
    let positions = q_highlighter
        .iter()
        .filter_map(|(entity, highlighter)| {
            if !rules.0.contains(&highlighter.position) {
                commands.entity(entity).despawn();
                None
            } else {
                Some(highlighter.position)
            }
        })
        .collect::<BTreeSet<_>>();
    // Newly added rules
    for &(start, end) in rules.0.difference(&positions) {
        println!("New rules: {start:?} {end:?}");
        commands.spawn((
            RuleHighlighter {
                status: RuleStatus::Active,
                position: (start, end),
            },
            Visibility::Hidden,
        ));
    }
}

pub(crate) fn respond_to_contradiction(
    mut q_highlighter: Query<&mut RuleHighlighter>,
    mut reader: MessageReader<infr_client::LevelError>,
    time: Res<Time>,
) {
    let mut positions = BTreeSet::new();
    for error in reader.read() {
        if let transfer::ServerError::Contradiction(contradiction) = &error.0 {
            positions.extend(contradiction.rules.iter().copied());
        }
    }
    if positions.is_empty() {
        return;
    }
    q_highlighter.iter_mut().for_each(|mut highlighter| {
        if positions.contains(&highlighter.position) {
            highlighter.status = RuleStatus::Error(time.elapsed());
        }
    })
}

pub(crate) fn update_highlight_display(
    mut q_highlighter: Query<(
        &mut RuleHighlighter,
        &mut infr_client::Position,
        &mut Mesh2d,
        &mut MeshMaterial2d<ColorMaterial>,
        &mut Visibility,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    materials: Res<HighlighterMaterials>,
) {
    const RED_DURATION: Duration = Duration::new(2, 0);
    const LOW_THICKNESS: f32 = 2.0;
    const HIGH_THICKNESS: f32 = 2.5;

    q_highlighter.iter_mut().for_each(
        |(mut highlighter, mut position, mut mesh, mut material, mut visibility)| {
            if let RuleStatus::Error(start_time) = highlighter.status
                && (time.elapsed() - start_time > RED_DURATION)
            {
                highlighter.status = RuleStatus::Active;
            }
            if highlighter.is_changed() {
                *position = infr_client::Position(Vec2::new(
                    (highlighter.position.0.0 + highlighter.position.1.0) as f32 * 0.5,
                    (highlighter.position.0.1 + highlighter.position.1.1) as f32 * 0.5,
                ));
                let rect = Rectangle::new(
                    ((highlighter.position.1.0 - highlighter.position.0.0).abs() + 1) as f32
                        * config::CONFIG.display.tile_size.0 as f32,
                    ((highlighter.position.1.1 - highlighter.position.0.1).abs() + 1) as f32
                        * config::CONFIG.display.tile_size.1 as f32,
                );
                let new_mesh = match highlighter.status {
                    RuleStatus::Active => {
                        *material = MeshMaterial2d(materials.random_active());
                        meshes.add(rect.to_ring(LOW_THICKNESS))
                    }
                    RuleStatus::Error(_) => {
                        *material = MeshMaterial2d(materials.random_error());
                        meshes.add(rect.to_ring(HIGH_THICKNESS))
                    }
                };
                *mesh = Mesh2d(new_mesh);
                *visibility = Visibility::Inherited;
            }
        },
    );
}
