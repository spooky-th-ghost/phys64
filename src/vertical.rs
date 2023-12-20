use crate::types::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct SecondTakePlugin;

impl Plugin for SecondTakePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity::new(0.02))
            .insert_resource(CameraConfig {
                mode: CameraMode::Normal,
            })
            .add_systems(Startup, setup)
            .add_systems(
                FixedUpdate,
                (
                    read_inputs,
                    (
                        set_player_direction,
                        rotate_to_direction,
                        apply_gravity,
                        jump,
                        release_jump,
                        handle_ground_sensor,
                        apply_drift,
                    ),
                    apply_forces,
                    set_translation,
                )
                    .chain(),
            )
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    }
}

#[derive(PartialEq)]
pub enum CameraMode {
    Ortho,
    Normal,
}

#[derive(Resource)]
pub struct CameraConfig {
    mode: CameraMode,
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

fn setup(
    camera_config: Res<CameraConfig>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let projection_type = if camera_config.mode == CameraMode::Ortho {
        Projection::Orthographic(OrthographicProjection {
            scale: 0.025,
            ..default()
        })
    } else {
        Projection::default()
    };

    let camera_transform = if camera_config.mode == CameraMode::Ortho {
        Transform::from_translation(Vec3::new(0.0, 2.0, -30.0))
            .looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y)
    } else {
        Transform::from_translation(Vec3::new(0.0, 10.0, -15.0)).looking_at(Vec3::ZERO, Vec3::Y)
    };

    commands
        .spawn(Camera3dBundle {
            transform: camera_transform,
            projection: projection_type,
            ..default()
        })
        .insert(MainCamera);

    commands.spawn((
        PbrBundle {
            material: materials.add(Color::LIME_GREEN.into()),
            mesh: meshes.add(shape::Capsule { ..default() }.into()),
            ..default()
        },
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
        InputBuffer::default(),
    ));

    commands.spawn((
        PbrBundle {
            material: materials.add(Color::WHITE.into()),
            mesh: meshes.add(shape::Box::new(20.0, 0.5, 20.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -2.0, 0.0)),
            ..default()
        },
        Collider::cuboid(10.0, 0.25, 10.0),
        RigidBody::Fixed,
    ));
}

fn read_inputs(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut input_buffer_query: Query<&mut InputBuffer>,
) {
    for mut buffer in &mut input_buffer_query {
        buffer.tick(time.delta());
        if input.just_pressed(KeyCode::Space) {
            buffer.press(PlayerAction::Jump);
        }

        if input.just_released(KeyCode::Space) {
            buffer.release(PlayerAction::Jump);
        }

        if input.any_just_pressed(vec![KeyCode::A, KeyCode::D, KeyCode::S, KeyCode::W]) {
            buffer.press(PlayerAction::Move);
        }

        if input.any_just_released(vec![KeyCode::A, KeyCode::D, KeyCode::S, KeyCode::W]) {
            buffer.release(PlayerAction::Move);
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
        &GroundSensor,
    )>,
) {
    for (mut momentum, mut speed, direction, transform, ground_sensor) in &mut query {
        if ground_sensor.grounded() {
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
            forces.add_to(ForceId::Drift, drift * time.delta_seconds() * 0.25);
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
        } else {
            if forces.has_key(ForceId::Gravity) {
                forces.remove(ForceId::Gravity);
            }
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

fn handle_ground_sensor(
    mut ground_sensor_query: Query<(&mut GroundSensor, &mut Forces, &Transform)>,
    rapier_context: Res<RapierContext>,
) {
    for (mut ground_sensor, mut forces, transform) in &mut ground_sensor_query {
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
            ground_sensor.set_state(GroundedState::Grounded);
            forces.remove(ForceId::Jump);
        } else {
            ground_sensor.set_state(GroundedState::Airborne);
        }
    }
}

fn jump(mut query: Query<(&mut Forces, &InputBuffer, &GroundSensor), With<Player>>) {
    for (mut forces, buffer, sensor) in &mut query {
        if buffer.just_pressed(PlayerAction::Jump) && sensor.grounded() {
            println!("{:?}", f32::from(Unit(60)));
            forces.add(
                ForceId::Jump,
                Force::new(
                    Vec3::Y * f32::from(Unit(60)),
                    Some(0.15),
                    ForceDecayType::Manual,
                ),
            );
        }
    }
}

fn release_jump(mut player_query: Query<(&mut Forces, &Momentum, &InputBuffer), With<Player>>) {
    for (mut forces, momentum, buffer) in &mut player_query {
        if buffer.released(PlayerAction::Jump) || momentum.y() <= 0.0 {
            if forces.has_key(ForceId::Jump) {
                forces.remove(ForceId::Jump);
            }
        }
    }
}

fn apply_forces(time: Res<Time>, mut physics_query: Query<(&mut Momentum, &mut Forces)>) {
    for (mut momentum, mut forces) in &mut physics_query {
        forces.tick(time.delta());
        let forces_vector = forces.get_combined_force();
        momentum.set(forces_vector);
    }
}

pub fn set_translation(mut query: Query<(&mut KinematicCharacterController, &Momentum)>) {
    for (mut character, momentum) in &mut query {
        let mut translation_to_apply: Vec3 = Vec3::ZERO;
        if momentum.is_any() {
            translation_to_apply += momentum.get();
        }
        character.translation = Some(translation_to_apply);
    }
}
