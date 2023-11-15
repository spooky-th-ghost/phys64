use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    let gravity_force: f32 = Unit(4).into();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                set_player_direction,
                apply_momentum,
                rotate_to_direction,
                handle_speed,
                apply_gravity,
            ),
        )
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0))
        .insert_resource(Gravity(Vec3::NEG_Y * gravity_force))
        .run();
}

#[derive(Resource)]
struct Gravity(Vec3);

impl Gravity {
    pub fn get(&self) -> Vec3 {
        self.0
    }
    pub fn set(&mut self, value: Vec3) {
        self.0 = value;
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainCamera;

#[derive(Component, Default)]
struct Direction(pub Vec3);

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

#[derive(Default, Component)]
pub struct Momentum(pub Vec3);

impl Momentum {
    pub fn get(&self) -> Vec3 {
        self.0
    }

    pub fn set(&mut self, value: Vec3) {
        self.0 = value;
    }

    pub fn add(&mut self, value: Vec3) {
        self.0 += value;
    }

    pub fn is_any(&self) -> bool {
        self.0 != Vec3::ZERO
    }

    pub fn is_active(&self) -> bool {
        self.0.length() >= 0.3
    }

    pub fn clear_vertical(&mut self) {
        self.0.y = 0.0;
    }

    pub fn clear_horizontal(&mut self) {
        self.0.x = 0.0;
        self.0.z = 0.0;
    }

    pub fn reset(&mut self) {
        self.0 = Vec3::ZERO;
    }
}

#[derive(Component)]
pub struct GravityAffected;

#[derive(Component)]
pub struct Speed {
    current: f32,
    accel: f32,
    base: f32,
    max: f32,
    base_max: f32,
    accel_timer: Timer,
    reset_timer: Timer,
}

#[derive(Component)]
pub struct Force {
    applied_force: Vec3,
    expiration_timer: Timer,
    lifetime: f32,
}

impl Force {
    pub fn new(applied_force: Vec3, lifetime: f32) -> Self {
        Force {
            applied_force,
            expiration_timer: Timer::from_seconds(lifetime, TimerMode::Once),
            lifetime,
        }
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        self.expiration_timer.tick(delta);
    }

    pub fn finished(&self) -> bool {
        self.expiration_timer.finished()
    }

    pub fn reset(&mut self) {
        self.expiration_timer = Timer::from_seconds(self.lifetime, TimerMode::Once);
    }

    pub fn add_force(&mut self, force: Vec3) {
        self.applied_force += force;
    }

    pub fn add_time(&mut self, seconds: f32) {
        let remaining = self.expiration_timer.remaining_secs();
        self.expiration_timer = Timer::from_seconds(seconds + remaining, TimerMode::Once);
    }
}

#[derive(Component)]
pub struct Forces {
    forces: bevy::utils::HashMap<Entity, Force>,
}

impl Forces {
    pub fn add(&mut self, entity: Entity, force: Force) {
        self.forces.insert(entity, force);
    }

    pub fn remove(&mut self, entity: Entity) {
        let _result = self.forces.remove(&entity);
    }

    pub fn has_key(&self, entity: Entity) -> bool {
        if let Some(_) = self.forces.get(&entity) {
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        self.forces.values_mut().for_each(|force| force.tick(delta));
    }

    pub fn remove_finished_forces(&mut self) {
        let mut dead_keys: Vec<Entity> = Vec::new();
        self.forces
            .iter()
            .filter(|(_, value)| value.finished())
            .for_each(|(key, _)| dead_keys.push(*key));

        dead_keys.iter().for_each(|key| {
            self.forces.remove(key);
        });
    }

    pub fn get_total_force(&self) -> Vec3 {
        self.forces.values().map(|force| force.applied_force).sum()
    }

    pub fn update(&mut self, delta: std::time::Duration) -> Vec3 {
        self.tick(delta);
        let total_force = self.get_total_force();
        self.remove_finished_forces();
        total_force
    }
}

impl Speed {
    pub fn reset(&mut self) {
        self.current = self.base;
        self.max = self.base_max;
        self.accel_timer.reset();
    }

    pub fn tick_reset_timer(&mut self, delta: std::time::Duration) {
        self.reset_timer.tick(delta);
    }

    pub fn reset_reset_timer(&mut self) {
        self.reset_timer = Timer::from_seconds(0.25, TimerMode::Once);
    }

    pub fn should_reset(&self) -> bool {
        self.reset_timer.finished()
    }

    pub fn apply_speed(&mut self, value: f32) {
        self.current += value;
        self.cap();
    }

    pub fn cap(&mut self) {
        if self.current > 0.0 {
            if self.current >= self.max {
                self.current = self.max;
            }
        } else {
            self.current = self.current * 0.5;
        }
    }

    pub fn accelerate(&mut self, delta: std::time::Duration, seconds: f32) {
        self.accel_timer.tick(delta);
        if self.accel_timer.finished() {
            if self.current < self.max {
                self.current = self.current + (self.max - self.current) * (seconds * self.accel);
            } else {
                self.current = self.max;
            }
        }
    }
}

impl Default for Speed {
    fn default() -> Self {
        Speed {
            base: Unit(4).into(),
            current: Unit(4).into(),
            accel: 2.5,
            max: Unit(48).into(),
            base_max: Unit(48).into(),
            accel_timer: Timer::from_seconds(0.6, TimerMode::Once),
            reset_timer: Timer::from_seconds(0.25, TimerMode::Once),
        }
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

    let mut force_map: bevy::utils::HashMap<Entity, Force> = bevy::utils::HashMap::new();

    force_map.insert(Entity::from_raw(2), Force::new(Vec3::ONE, 1.0));
    force_map.insert(
        Entity::from_raw(7),
        Force::new(Vec3::new(1.3, -0.2, 17.8), 2.0),
    );
    force_map.insert(
        Entity::from_raw(19),
        Force::new(Vec3::new(1.1, 23.2, 9.8), 3.0),
    );

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
        Forces { forces: force_map },
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
        (&mut Momentum, &KinematicCharacterControllerOutput),
        With<GravityAffected>,
    >,
) {
    for (mut momentum, controller) in &mut character_query {
        if !controller.grounded {
            momentum.add(gravity.get());
        } else {
            momentum.clear_vertical();
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
            let turn_speed = speed.current * 30.0;

            transform.rotation = transform
                .rotation
                .slerp(rotation_target.rotation, time.delta_seconds() * turn_speed);
        }
    }
}

fn handle_speed(
    time: Res<Time>,
    mut query: Query<(&mut Momentum, &mut Speed, &Direction, &Transform)>,
) {
    for (mut momentum, mut speed, direction, transform) in &mut query {
        if direction.is_active() {
            speed.accelerate(time.delta(), time.delta_seconds());
            momentum.set(speed.current * transform.forward());
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

pub fn apply_momentum(mut query: Query<(&mut KinematicCharacterController, &Momentum)>) {
    for (mut character, momentum) in &mut query {
        if momentum.is_any() {
            character.translation = Some(momentum.get());
        }
    }
}

#[derive(Default, Debug)]
pub struct Unit(pub i32);

impl std::ops::Add for Unit {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Unit(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for Unit {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for Unit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Unit(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for Unit {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl From<f32> for Unit {
    fn from(value: f32) -> Self {
        Unit((value * 128.0) as i32)
    }
}

impl From<i32> for Unit {
    fn from(value: i32) -> Self {
        Unit(value)
    }
}

impl From<Unit> for f32 {
    fn from(value: Unit) -> Self {
        value.0 as f32 / 128.0
    }
}
