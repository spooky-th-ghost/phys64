use bevy::{prelude::*, utils::HashMap};
use std::collections::HashSet;

#[derive(Resource)]
pub struct Gravity {
    force: Vec3,
}

impl Gravity {
    pub fn new(amount: f32) -> Self {
        let force = Vec3::NEG_Y * amount;

        Gravity { force }
    }

    pub fn force(&self) -> Vec3 {
        self.force
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum PlayerAction {
    Jump,
    Move,
}

#[derive(Component, Default)]
pub struct InputBuffer {
    pressed_actions: HashSet<PlayerAction>,
    stale_actions: HashSet<PlayerAction>,
    buffered_actions: HashMap<PlayerAction, Timer>,
}

impl InputBuffer {
    pub fn just_pressed(&self, action: PlayerAction) -> bool {
        match self.pressed_actions.get(&action) {
            Some(_) => match self.stale_actions.get(&action) {
                None => return true,
                _ => (),
            },
            _ => (),
        }

        match self.buffered_actions.get(&action) {
            Some(_) => match self.stale_actions.get(&action) {
                Some(_) => return false,
                None => return true,
            },
            None => false,
        }
    }

    pub fn pressed(&self, action: PlayerAction) -> bool {
        self.pressed_actions.get(&action).is_some()
    }

    pub fn released(&self, action: PlayerAction) -> bool {
        self.pressed_actions.get(&action).is_none() && self.buffered_actions.get(&action).is_none()
    }

    pub fn press(&mut self, action: PlayerAction) {
        self.buffered_actions
            .insert(action, Timer::from_seconds(0.166, TimerMode::Once));
        self.pressed_actions.insert(action);
    }

    pub fn release(&mut self, action: PlayerAction) {
        self.buffered_actions.remove(&action);
        self.stale_actions.remove(&action);
        self.pressed_actions.remove(&action);
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        let mut stale_buffers: Vec<PlayerAction> = Vec::new();
        self.buffered_actions
            .iter_mut()
            .for_each(|(action, timer)| {
                timer.tick(delta);
                match timer.finished() {
                    true => {
                        self.stale_actions.insert(*action);
                        stale_buffers.push(*action);
                    }
                    _ => (),
                }
            });
        for action in stale_buffers.iter() {
            self.buffered_actions.remove(action);
            self.stale_actions.insert(*action);
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
pub struct Momentum(pub Vec3);

impl Momentum {
    pub fn get(&self) -> Vec3 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.0.y
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

#[derive(Default, PartialEq)]
pub enum JumpStage {
    #[default]
    Single,
    Double,
    Tripple,
}

impl JumpStage {
    fn get_jump_force(&self) -> f32 {
        match self {
            Self::Single => f32::from(Unit(60)),
            Self::Double => f32::from(Unit(65)),
            Self::Tripple => f32::from(Unit(70)),
        }
    }
}

#[derive(Component, Default)]
pub struct Jumper {
    stage: JumpStage,
}

impl Jumper {
    pub fn increase_stage(&mut self) {
        match self.stage {
            JumpStage::Single => self.stage = JumpStage::Double,
            JumpStage::Double => self.stage = JumpStage::Tripple,
            _ => (),
        }
    }

    pub fn reset_stage(&mut self) {
        self.stage = JumpStage::Single;
    }

    pub fn get_force(&self) -> f32 {
        self.stage.get_jump_force()
    }
}

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
            base: Unit(8).into(),
            current: Unit(8).into(),
            accel: 2.5,
            max: Unit(40).into(),
            base_max: Unit(40).into(),
            accel_timer: Timer::from_seconds(0.3, TimerMode::Once),
            reset_timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

#[derive(Default, Component)]
pub struct MoveDirection(pub Vec3);

impl MoveDirection {
    pub fn get(&self) -> Vec3 {
        self.0
    }

    pub fn set(&mut self, value: Vec3) {
        self.0 = value;
    }

    pub fn reset(&mut self) {
        self.0 = Vec3::ZERO;
    }

    pub fn is_any(&self) -> bool {
        self.0 != Vec3::ZERO
    }

    pub fn is_active(&self) -> bool {
        self.0.length() >= 0.3
    }
}

#[derive(PartialEq, Eq)]
pub enum ForceDecayType {
    Automatic,
    Manual,
}

#[derive(Component)]
pub struct Force {
    applied_force: Vec3,
    expiration_timer: Option<Timer>,
    decay_type: ForceDecayType,
}

impl Force {
    pub fn new(applied_force: Vec3, lifespan: Option<f32>, decay_type: ForceDecayType) -> Self {
        let expiration_timer = if let Some(total_lifetime) = lifespan {
            Some(Timer::from_seconds(total_lifetime, TimerMode::Once))
        } else {
            None
        };

        Force {
            applied_force,
            expiration_timer,
            decay_type,
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
            true
        }
    }

    pub fn add_force(&mut self, force: Vec3) {
        self.applied_force += force;
    }

    pub fn add_time(&mut self, seconds: f32) {
        if let Some(timer) = self.expiration_timer.as_mut() {
            let remaining = timer.remaining_secs();
            self.expiration_timer = Some(Timer::from_seconds(seconds + remaining, TimerMode::Once));
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum ForceId {
    Gravity,
    Jump,
    Wind,
    Slide,
    Drift,
}

#[derive(Component, Default)]
pub struct Forces {
    forces: HashMap<ForceId, Force>,
    scheduled_to_remove: HashSet<ForceId>,
}

impl Forces {
    pub fn add(&mut self, force_id: ForceId, force: Force) {
        self.forces.insert(force_id, force);
    }

    pub fn length(&self) -> usize {
        self.forces.len()
    }

    pub fn add_to(&mut self, force_id: ForceId, amount: Vec3) {
        if let Some(force) = self.forces.get_mut(&force_id) {
            force.add_force(amount);
        } else {
            self.add(force_id, Force::new(amount, None, ForceDecayType::Manual));
        }
    }

    pub fn remove(&mut self, force_id: ForceId) {
        let mut can_remove = true;
        if let Some(force) = self.forces.get(&force_id) {
            if !force.finished() {
                can_remove = false;
            }
        }
        if can_remove {
            let _ = self.forces.remove(&force_id);
        } else {
            self.scheduled_to_remove.insert(force_id);
        }
    }

    pub fn has_key(&self, force_id: ForceId) -> bool {
        if let Some(_) = self.forces.get(&force_id) {
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.forces = bevy::utils::HashMap::new();
    }

    fn remove_finished_forces(&mut self) {
        let mut dead_keys: Vec<ForceId> = Vec::new();
        self.forces
            .iter()
            .filter(|(_, value)| value.finished() && value.decay_type == ForceDecayType::Automatic)
            .for_each(|(key, _)| dead_keys.push(*key));

        dead_keys.iter().for_each(|key| {
            self.forces.remove(key);
        });
    }

    fn remove_scheduled_forces(&mut self) {
        let mut finished_keys: Vec<ForceId> = Vec::new();

        self.forces
            .iter()
            .filter(|(_, value)| value.finished() && value.decay_type == ForceDecayType::Manual)
            .for_each(|(key, _)| finished_keys.push(*key));

        finished_keys.iter().for_each(|key| {
            if self.scheduled_to_remove.contains(key) {
                self.forces.remove(key);
                self.scheduled_to_remove.remove(key);
            }
        });
    }

    pub fn get_combined_force(&self) -> Vec3 {
        self.forces.values().map(|force| force.applied_force).sum()
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        self.forces.values_mut().for_each(|force| force.tick(delta));
        self.remove_finished_forces();
        self.remove_scheduled_forces();
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum GroundedState {
    #[default]
    Grounded,
    Airborne,
}

#[derive(Component)]
pub struct GroundSensor {
    shape: bevy_rapier3d::prelude::Collider,
    state: GroundedState,
}

impl GroundSensor {
    pub fn grounded(&self) -> bool {
        self.state == GroundedState::Grounded
    }

    pub fn set_state(&mut self, state: GroundedState) {
        self.state = state;
    }

    pub fn shape_ref(&self) -> &bevy_rapier3d::prelude::Collider {
        &self.shape
    }
}

impl Default for GroundSensor {
    fn default() -> Self {
        GroundSensor {
            shape: bevy_rapier3d::prelude::Collider::cuboid(0.25, 0.1, 0.25),
            state: GroundedState::default(),
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
