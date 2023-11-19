use bevy::prelude::*;

#[derive(Resource)]
pub struct Gravity {
    force: Vec3,
    entity: Entity,
}

impl Gravity {
    pub fn new(amount: Unit) -> Self {
        let force = Vec3::NEG_Y * f32::from(amount);
        let entity = Entity::from_raw(99);

        Gravity{force, entity}
    }

    pub fn force(&self) -> Vec3 {
        self.force
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn set(&mut self, value: Vec3) {
        self.force = value;
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MainCamera;


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

    pub fn decay(&mut self, time: f32) {
        let current = self.get();
        self.set(current.lerp(Vec3::ZERO, time));
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

impl Speed {
    pub fn current(&self) -> f32 {
        self.current
    }

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

#[derive(Component)]
pub struct Force {
    applied_force: Vec3,
    expiration_timer: Option<Timer>,
    lifetime: f32,
}

impl Force {
    pub fn new(applied_force: Vec3, lifespan: Option<f32>) -> Self {
        let (expiration_timer,lifetime) = if let Some(total_lifetime) = lifespan {
            (Some(Timer::from_seconds(total_lifetime, TimerMode::Once)), total_lifetime)
        } else {
            (None, 0.0)
        };

        Force {
            applied_force,
            expiration_timer,
            lifetime,
        }
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        if let Some(timer) = self.expiration_timer.as_mut() {
            timer.tick(delta);
        }
    }

    pub fn finished(&self) -> bool {
        if let Some(timer) = &self.expiration_timer {
            timer.finished()
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        if let Some(_) = self.expiration_timer {
            self.expiration_timer =  Some(Timer::from_seconds(self.lifetime, TimerMode::Once));
        }
    }

    pub fn add_force(&mut self, force: Vec3) {
        self.applied_force += force;
    }

    pub fn add_time(&mut self, seconds: f32) {
        if let Some(mut timer) = self.expiration_timer.as_mut() {
            let remaining = timer.remaining_secs();
            self.expiration_timer = Some(Timer::from_seconds(seconds + remaining, TimerMode::Once));
        }
    }
}

#[derive(Component, Default)]
pub struct Forces {
    forces: bevy::utils::HashMap<Entity, Force>,
}

impl Forces {
    pub fn add(&mut self, entity: Entity, force: Force) {
        self.forces.insert(entity, force);
    }

    pub fn add_constant(&mut self, entity: Entity, amount: Vec3) {
        self.forces.insert(entity, Force::new(amount, None));
    }

    pub fn add_to(&mut self, entity: Entity, amount: Vec3) {
        if let Some(mut force) = self.forces.get_mut(&entity) {
            force.add_force(amount);
        }
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
