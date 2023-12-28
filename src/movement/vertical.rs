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
                handle_ground_sensor,
                stick_to_slopes,
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

fn stick_to_slopes(mut character_query: Query<(&mut Forces, &GroundSensor)>) {
    for (mut forces, sensor) in &mut character_query {
        if sensor.grounded() && sensor.get_surface_angle() > 5.0 {
            if !forces.has_key(ForceId::Slope) {
                forces.add(
                    ForceId::Slope,
                    Force::new(
                        Vec3::NEG_Y * sensor.get_surface_angle() * sensor.get_surface_angle(),
                        None,
                        ForceDecayType::Manual,
                    ),
                );
            }
        } else if !sensor.grounded() {
            if forces.has_key(ForceId::Slope) {
                forces.remove(ForceId::Slope);
            }
        }
    }
}

fn handle_ground_sensor(
    mut ground_sensor_query: Query<(
        Entity,
        &mut GroundSensor,
        &mut Forces,
        &mut Jumper,
        &Momentum,
        &Transform,
    )>,
    rapier_context: Res<RapierContext>,
) {
    for (entity, mut ground_sensor, mut forces, mut jumper, momentum, transform) in
        &mut ground_sensor_query
    {
        // Detect the ground using a shape cast
        let cast_origin = transform.translation + Vec3::NEG_Y * 0.8;
        let shape_rotation = transform.rotation;
        let cast_direction = Vec3::NEG_Y;
        let cast_shape = ground_sensor.shape_ref();
        let cast_distance = 0.3;
        let stop_at_penetration = false;
        let cast_filter = QueryFilter::new().exclude_collider(entity);

        if let Some(_) = rapier_context.cast_shape(
            cast_origin,
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
                forces.remove(ForceId::Slide);
                jumper.land();
            }
        } else {
            ground_sensor.set_state(GroundedState::Airborne);
        }
        //Cast a ray to get the angle of our slope
        if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(
            cast_origin,
            cast_direction,
            cast_distance,
            true,
            cast_filter,
        ) {
            ground_sensor.set_normal(intersection.normal);
        }
    }
}
