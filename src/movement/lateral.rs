use crate::{camera::MainCamera, types::*};
use bevy::prelude::*;

pub struct LateralMovementPlugin;

impl Plugin for LateralMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                set_player_direction,
                rotate_to_direction,
                handle_speed,
                apply_drift,
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

fn handle_speed(
    time: Res<Time>,
    mut query: Query<(
        &mut Forces,
        &mut Speed,
        &MoveDirection,
        &Transform,
        &GroundSensor,
    )>,
) {
    for (mut forces, mut speed, direction, transform, ground_sensor) in &mut query {
        if ground_sensor.grounded() {
            if direction.is_active() {
                speed.accelerate(time.delta(), time.delta_seconds());
                let movement_force = speed.current() * transform.forward();
                forces.add(
                    ForceId::Run,
                    Force::new(movement_force, None, ForceDecayType::Manual),
                );
                speed.reset_reset_timer();
            } else {
                if let Some(run_vec) = forces.get_vector(ForceId::Run) {
                    forces.remove(ForceId::Run);
                    forces.add(
                        ForceId::Skid,
                        Force::new(run_vec * 0.33, Some(0.25), ForceDecayType::Automatic),
                    )
                }
                speed.tick_reset_timer(time.delta());
                if speed.should_reset() {
                    speed.reset();
                }
            }
        }
    }
}

fn apply_drift(
    time: Res<Time>,
    mut character_query: Query<(&mut Forces, &GroundSensor)>,
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

fn rotate_to_direction(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MoveDirection, &Speed, &GroundSensor)>,
    mut rotation_target: Local<Transform>,
) {
    for (mut transform, direction, speed, ground_sensor) in &mut query {
        if ground_sensor.grounded() {
            rotation_target.translation = transform.translation;
            let flat_velo_direction =
                Vec3::new(direction.0.x, 0.0, direction.0.z).normalize_or_zero();
            if flat_velo_direction != Vec3::ZERO {
                let target_position = rotation_target.translation + flat_velo_direction;

                rotation_target.look_at(target_position, Vec3::Y);
                let turn_speed = speed.current() * 0.5;

                transform.rotation = transform
                    .rotation
                    .slerp(rotation_target.rotation, time.delta_seconds() * turn_speed);
            }
        }
    }
}
