use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
mod types;
use types::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                set_player_direction,
                rotate_to_direction,
                handle_speed,
                jump,
                apply_gravity,
                apply_drift,
                set_translation,
                decay_momentum
            ).chain(),
        )
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0))
        .run();
}

#[derive(Default, Component)]
pub struct Direction(pub Vec3);

impl Direction {
    pub fn get(&self) -> Vec3 {
        self.0
    }

    pub fn set(&mut self, value: Vec3) {
        self.0 = value;
    }

    pub fn is_any(&self) -> bool {
        self.0 != Vec3::ZERO
    }

    pub fn is_active(&self) -> bool {
        self.0.length() >= 0.3
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Gravity::new(Unit(4)));

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
            ..default()
        },
        Direction::default(),
        Momentum::default(),
        Speed::default(),
        Forces::default(),
        GravityAffected,
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

fn apply_gravity(
    gravity: Res<Gravity>,
    mut character_query: Query<
        (&mut Forces, &KinematicCharacterControllerOutput),
        With<GravityAffected>,
    >,
) {
    for (mut forces, output) in &mut character_query {
        if !output.grounded {
            if !forces.has_key(gravity.entity()) {
                forces.add_constant(gravity.entity(), gravity.force());
            } else {
                forces.add_to(gravity.entity(), gravity.force());
            }
        } else {
            forces.remove(gravity.entity());
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
        let drift_entity = Entity::from_raw(24);
        if output.grounded {
            forces.remove(drift_entity);
        } else {
            let drift = get_direction_in_camera_space(camera_transform, &input);
            if forces.has_key(drift_entity) {
                forces.add_to(drift_entity, drift);
            } else {
                forces.add_to(drift_entity, drift);
            }
        }
    }
}

fn set_player_direction(
    mut player_query: Query<&mut Direction, With<Player>>,
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
    mut query: Query<(&mut Transform, &Direction, &Speed)>,
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
    mut query: Query<(&mut Momentum, &mut Speed, &Direction, &Transform, &KinematicCharacterControllerOutput)>,
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

fn jump(input: Res<Input<KeyCode>>, mut query: Query<(&mut Momentum, &KinematicCharacterControllerOutput)>) {
    for (mut momentum, output) in &mut query {
        if input.pressed(KeyCode::Space) && output.grounded {
            momentum.add(Vec3::Y * f32::from(Unit(24)));
        }
    }
}

pub fn set_translation(time: Res<Time>, mut query: Query<(&mut KinematicCharacterController, &mut Forces,  &Momentum)>) {
    for (mut character, mut forces, momentum) in &mut query {
        let mut translation_to_apply: Vec3 = Vec3::ZERO;
        if momentum.is_any() {
            translation_to_apply += momentum.get();
        }
        translation_to_apply += forces.update(time.delta());
        character.translation = Some(translation_to_apply);
    }
}

pub fn decay_momentum(time: Res<Time>, mut query: Query<&mut Momentum>) {
    for mut momentum in &mut query {
        momentum.decay(time.delta_seconds());
    }
}

