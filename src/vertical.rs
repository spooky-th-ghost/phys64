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
                    (apply_gravity, jump, release_jump, handle_ground_sensor),
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

#[derive(Default, Resource)]
pub struct DebugCounter(u32);

impl DebugCounter {
    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn increase(&mut self) {
        self.0 += 1;
    }
}

fn read_inputs(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut input_buffer_query: Query<&mut InputBuffer>,
    mut jump_presses: Local<DebugCounter>,
) {
    for mut buffer in &mut input_buffer_query {
        buffer.tick(time.delta());
        if input.just_pressed(KeyCode::Space) {
            jump_presses.increase();
            buffer.press(PlayerAction::Jump);
        }

        if input.just_released(KeyCode::Space) {
            buffer.release(PlayerAction::Jump);
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
