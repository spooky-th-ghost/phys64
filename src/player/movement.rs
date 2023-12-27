use super::Player;
use crate::{
    camera::MainCamera,
    input::{InputBuffer, PlayerAction},
    types::*,
};
use bevy::prelude::*;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                set_player_direction,
                apply_drift,
                jump,
                release_jump,
                enter_sliding,
            )
                .in_set(EngineSystemSet::CalculateMomentum),
        );
    }
}

fn get_direction_in_camera_space(
    camera_transform: &Transform,
    input: &Res<Input<KeyCode>>,
) -> Vec3 {
    let mut forward = camera_transform.forward();
    forward.y = 0.0;
    forward = forward.normalize();

    let mut right = camera_transform.right();
    right.y = 0.0;
    right = right.normalize();

    let mut x = 0.0;
    let mut y = 0.0;

    if input.pressed(KeyCode::A) {
        x -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        x += 1.0;
    }
    if input.pressed(KeyCode::W) {
        y += 1.0;
    }
    if input.pressed(KeyCode::S) {
        y -= 1.0;
    }

    let right_vec: Vec3 = x * right;
    let forward_vec: Vec3 = y * forward;

    right_vec + forward_vec
}

fn set_player_direction(
    mut player_query: Query<&mut MoveDirection, With<Player>>,
    camera_query: Query<&Transform, With<MainCamera>>,
    input: Res<Input<KeyCode>>,
) {
    let camera_transform = camera_query.single();
    for mut direction in &mut player_query {
        direction.0 = get_direction_in_camera_space(camera_transform, &input);
    }
}

fn apply_drift(
    time: Res<Time>,
    mut character_query: Query<(&mut Forces, &GroundSensor), With<Player>>,
    camera_query: Query<&Transform, With<MainCamera>>,
    input: Res<Input<KeyCode>>,
) {
    let camera_transform = camera_query.single();
    for (mut forces, ground_sensor) in &mut character_query {
        if ground_sensor.grounded() {
            forces.remove(ForceId::Drift);
        } else {
            let drift = get_direction_in_camera_space(camera_transform, &input);
            forces.add_to(ForceId::Drift, drift * time.delta_seconds() * 5.0);
        }
    }
}

fn enter_sliding(
    mut commands: Commands,
    mut player_query: Query<(Entity, &InputBuffer, &GroundSensor, &mut Forces), Without<Sliding>>,
) {
    for (entity, buffer, ground_sensor, mut forces) in &mut player_query {
        if buffer.pressed(PlayerAction::Crouch)
            && forces.has_key(ForceId::Run)
            && ground_sensor.grounded()
        {
            let run_vector = forces.get_vector(ForceId::Run);

            if let Some(vector) = run_vector {
                forces.remove(ForceId::Run);

                commands.entity(entity).insert(Sliding);
                forces.add(
                    ForceId::Slide,
                    Force::new(vector * 2.0, None, ForceDecayType::Manual),
                );
            }
        }
    }
}

fn jump(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Forces,
            &InputBuffer,
            &GroundSensor,
            &Jumper,
            Has<Sliding>,
        ),
        With<Player>,
    >,
) {
    for (entity, mut forces, buffer, sensor, jumper, is_sliding) in &mut query {
        if buffer.just_pressed(PlayerAction::Jump) && sensor.grounded() {
            if is_sliding {
                commands.entity(entity).remove::<Sliding>();
            }
            forces.add(
                ForceId::Jump,
                Force::new(
                    Vec3::Y * jumper.get_force(),
                    Some(0.15),
                    ForceDecayType::Manual,
                ),
            );
        }
    }
}

fn release_jump(mut player_query: Query<(&mut Forces, &Momentum, &InputBuffer), With<Player>>) {
    for (mut forces, momentum, buffer) in &mut player_query {
        if (buffer.released(PlayerAction::Jump) || momentum.y() <= 0.0)
            && forces.has_key(ForceId::Jump)
        {
            forces.remove(ForceId::Jump);
        }
    }
}
