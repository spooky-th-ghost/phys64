use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
mod types;
use types::*;
mod systems;
use systems::*;
mod camera;
use camera::*;
mod input;
use input::*;
mod player;
use player::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .register_type::<Momentum>()
        .add_plugins(MovementPlugin)
        .add_plugins(InputPlugin)
        .configure_sets(
            FixedUpdate,
            (
                PlayerSystemSet::Input,
                PlayerSystemSet::CalculateMomentum,
                PlayerSystemSet::ApplyMomentum,
            )
                .chain(),
        )
        .run();
}
