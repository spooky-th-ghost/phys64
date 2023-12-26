use crate::types::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod lateral;
mod vertical;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            lateral::LateralMovementPlugin,
            vertical::VerticalMovementPlugin,
        ))
        .add_systems(
            FixedUpdate,
            (apply_forces, set_translation)
                .chain()
                .in_set(EngineSystemSet::ApplyMomentum),
        );
    }
}
fn apply_forces(time: Res<Time>, mut physics_query: Query<(&mut Momentum, &mut Forces)>) {
    for (mut momentum, mut forces) in &mut physics_query {
        forces.tick(time.delta());
        let forces_vector = forces.get_combined_force();
        momentum.set(forces_vector * time.delta_seconds());
    }
}

fn set_translation(mut query: Query<(&mut KinematicCharacterController, &Momentum)>) {
    for (mut character, momentum) in &mut query {
        let mut translation_to_apply: Vec3 = Vec3::ZERO;
        if momentum.is_any() {
            translation_to_apply += momentum.get();
        }
        character.translation = Some(translation_to_apply);
    }
}
