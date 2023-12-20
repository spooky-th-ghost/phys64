use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
mod types;
use types::*;
mod vertical;
use vertical::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .register_type::<Momentum>()
        // .add_plugins(FirstTakePlugin)
        .add_plugins(SecondTakePlugin)
        .run();
}

pub struct FirstTakePlugin;

impl Plugin for FirstTakePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity::new(0.02))
            .add_systems(Startup, setup)
            .add_systems(
                FixedUpdate,
                (
                    set_player_direction,
                    rotate_to_direction,
                    handle_speed,
                    jump,
                    release_jump,
                    apply_gravity,
                    apply_drift,
                    set_translation,
                    decay_momentum,
                    reset,
                    handle_ground_sensor,
                )
                    .chain(),
            )
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 2.0, -30.0))
                .looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
            ..default()
        })
        .insert(MainCamera);

    commands.spawn((
        TransformBundle::default(),
        Player,
        Collider::capsule_y(0.5, 0.5),
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            snap_to_ground: Some(CharacterLength::Absolute(0.25)),
            ..default()
        },
        MoveDirection::default(),
        Momentum::default(),
        Speed::default(),
        Forces::default(),
        GravityAffected,
        GroundSensor::default(),
    ));

    commands.spawn((
        TransformBundle {
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ..default()
        },
        Collider::cuboid(5.0, 5.0, 5.0),
        RigidBody::Fixed,
    ));

    commands.spawn((
        TransformBundle {
            local: Transform::from_translation(Vec3::new(0.0, -2.0, 0.0)),
            ..default()
        },
        Collider::cuboid(10.0, 0.25, 10.0),
        RigidBody::Fixed,
    ));
}

fn reset(
    mut player_query: Query<
        (
            &mut Transform,
            &mut MoveDirection,
            &mut Forces,
            &mut Momentum,
        ),
        With<Player>,
    >,
    input: Res<Input<KeyCode>>,
) {
    for (mut transform, mut direction, mut forces, mut momentum) in &mut player_query {
        if input.just_pressed(KeyCode::R) {
            transform.translation = Vec3::Y;
            direction.reset();
            forces.reset();
            momentum.reset();
        }
    }
}

fn apply_gravity(
    gravity: Res<Gravity>,
    mut character_query: Query<
        (&mut Forces, &KinematicCharacterControllerOutput),
        With<GravityAffected>,
    >,
) {
    for (mut forces, output) in &mut character_query {
        if !output.grounded {
            if !forces.has_key(ForceId::Gravity) {
                forces.add(
                    ForceId::Gravity,
                    Force::new(gravity.force(), None, ForceDecayType::Manual),
                );
            } else {
                forces.add_to(ForceId::Gravity, gravity.force());
            }
        } else {
            forces.remove(ForceId::Gravity);
        }
    }
}

fn apply_drift(
    mut character_query: Query<(&mut Forces, &KinematicCharacterControllerOutput)>,
    camera_query: Query<&Transform, With<MainCamera>>,
    input: Res<Input<KeyCode>>,
) {
    let camera_transform = camera_query.single();
    for (mut forces, output) in &mut character_query {
        if output.grounded {
            forces.remove(ForceId::Drift);
        } else {
            let drift = get_direction_in_camera_space(camera_transform, &input);
            forces.add_to(ForceId::Drift, drift);
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

fn rotate_to_direction(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MoveDirection, &Speed)>,
    mut rotation_target: Local<Transform>,
) {
    for (mut transform, direction, speed) in &mut query {
        rotation_target.translation = transform.translation;
        let flat_velo_direction = Vec3::new(direction.0.x, 0.0, direction.0.z).normalize_or_zero();
        if flat_velo_direction != Vec3::ZERO {
            let target_position = rotation_target.translation + flat_velo_direction;

            rotation_target.look_at(target_position, Vec3::Y);
            let turn_speed = speed.current() * 30.0;

            transform.rotation = transform
                .rotation
                .slerp(rotation_target.rotation, time.delta_seconds() * turn_speed);
        }
    }
}

fn handle_speed(
    time: Res<Time>,
    mut query: Query<(
        &mut Momentum,
        &mut Speed,
        &MoveDirection,
        &Transform,
        &KinematicCharacterControllerOutput,
    )>,
) {
    for (mut momentum, mut speed, direction, transform, output) in &mut query {
        if output.grounded {
            if direction.is_active() {
                speed.accelerate(time.delta(), time.delta_seconds());
                momentum.set(speed.current() * transform.forward());
                speed.reset_reset_timer();
            } else {
                momentum.clear_horizontal();
                speed.tick_reset_timer(time.delta());
                if speed.should_reset() {
                    speed.reset();
                }
            }
        }
    }
}

fn handle_ground_sensor(
    mut ground_sensor_query: Query<(&mut GroundSensor, &Transform)>,
    rapier_context: Res<RapierContext>,
) {
    for (mut ground_sensor, transform) in &mut ground_sensor_query {
        let shape_position = transform.translation + Vec3::NEG_Y * 0.8;
        let shape_rotation = transform.rotation;
        let cast_direction = Vec3::NEG_Y;
        let cast_shape = ground_sensor.shape_ref();
        let cast_distance = 0.3;
        let stop_at_penetration = false;
        let cast_filter = QueryFilter::new();

        if let Some((handle, hit)) = rapier_context.cast_shape(
            shape_position,
            shape_rotation,
            cast_direction,
            cast_shape,
            cast_distance,
            stop_at_penetration,
            cast_filter,
        ) {
            ground_sensor.set_state(GroundedState::Grounded);
        } else {
            ground_sensor.set_state(GroundedState::Airborne);
        }
    }
}

fn jump(input: Res<Input<KeyCode>>, mut query: Query<(&mut Forces, &GroundSensor)>) {
    for (mut forces, sensor) in &mut query {
        if input.just_pressed(KeyCode::Space) && sensor.grounded() {
            forces.add(
                ForceId::Jump,
                Force::new(
                    Vec3::Y * f32::from(Unit(75)),
                    Some(0.2),
                    ForceDecayType::Manual,
                ),
            );
        }
    }
}

fn release_jump(input: Res<Input<KeyCode>>, mut player_query: Query<&mut Forces, With<Player>>) {
    for mut forces in &mut player_query {
        if !input.pressed(KeyCode::Space) {
            if forces.has_key(ForceId::Jump) {
                forces.remove(ForceId::Jump);
            }
        }
    }
}

pub fn set_translation(
    time: Res<Time>,
    mut query: Query<(&mut KinematicCharacterController, &mut Forces, &Momentum)>,
) {
    for (mut character, mut forces, momentum) in &mut query {
        let mut translation_to_apply: Vec3 = Vec3::ZERO;
        if momentum.is_any() {
            translation_to_apply += momentum.get();
        }
        forces.tick(time.delta());
        translation_to_apply += forces.get_combined_force();
        character.translation = Some(translation_to_apply);
    }
}

pub fn decay_momentum(time: Res<Time>, mut query: Query<&mut Momentum>) {
    for mut momentum in &mut query {
        momentum.decay(time.delta_seconds());
    }
}
