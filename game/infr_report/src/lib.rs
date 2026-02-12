mod rules;

use bevy::prelude::*;

pub struct InfrReportPlugin;

impl Plugin for InfrReportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, rules::setup_materials);
        app.add_systems(Update, rules::respond_to_contradiction);
        app.add_systems(
            PostUpdate,
            (rules::update_highlighters, rules::update_highlight_display),
        );
    }
}
