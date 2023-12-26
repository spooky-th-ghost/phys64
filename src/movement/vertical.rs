use crate::{
    input::{InputBuffer, PlayerAction},
    types::*,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct VerticalMovementPlugin;

impl Plugin for VerticalMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                handle_jump_timer,
                apply_gravity,
                jump,
                release_jump,
                handle_ground_sensor,
            )
                .in_set(EngineSystemSet::CalculateMomentum),
        );
    }
}

fn handle_jump_timer(time: Res<Time>, mut jump_query: Query<(&mut Jumper, &GroundSensor)>) {
    for (mut jumper, ground_sensor) in &mut jump_query {
        if ground_sensor.grounded() {
            jumper.tick(time.delta());
        }
    }
}

fn apply_gravity(
    gravity: Res<Gravity>,
    mut character_query: Query<(&mut Forces, &GroundSensor), With<GravityAffected>>,
) {
    for (mut forces, ground_sensor) in &mut character_query {
        if !ground_sensor.grounded() {
            if !forces.has_key(ForceId::Gravity) {
                forces.add(
                    ForceId::Gravity,
                    Force::new(gravity.force(), None, ForceDecayType::Manual),
                );
            } else {
                forces.add_to(ForceId::Gravity, gravity.force());
            }
        } else if forces.has_key(ForceId::Gravity) {
            forces.remove(ForceId::Gravity);
        }
    }
}

fn handle_ground_sensor(
    mut ground_sensor_query: Query<(
        &mut GroundSensor,
        &mut Forces,
        &mut Jumper,
        &Momentum,
        &Transform,
    )>,
    rapier_context: Res<RapierContext>,
) {
    for (mut ground_sensor, mut forces, mut jumper, momentum, transform) in &mut ground_sensor_query
    {
        let shape_position = transform.translation + Vec3::NEG_Y * 0.8;
        let shape_rotation = transform.rotation;
        let cast_direction = Vec3::NEG_Y;
        let cast_shape = ground_sensor.shape_ref();
        let cast_distance = 0.3;
        let stop_at_penetration = false;
        let cast_filter = QueryFilter::new();

        if let Some(_) = rapier_context.cast_shape(
            shape_position,
            shape_rotation,
            cast_direction,
            cast_shape,
            cast_distance,
            stop_at_penetration,
            cast_filter,
        ) {
            if momentum.y() <= 0.0 && !ground_sensor.grounded() {
                ground_sensor.set_state(GroundedState::Grounded);
                forces.remove(ForceId::Jump);
                jumper.land();
            }
        } else {
            ground_sensor.set_state(GroundedState::Airborne);
        }
    }
}

fn jump(mut query: Query<(&mut Forces, &InputBuffer, &GroundSensor, &Jumper), With<Player>>) {
    for (mut forces, buffer, sensor, jumper) in &mut query {
        if buffer.just_pressed(PlayerAction::Jump) && sensor.grounded() {
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
