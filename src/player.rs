use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerData {
    pub player_position: Vec3,
    pub held_object_position: Vec3,
    pub held_object_index: IndexPointer,
    pub distance_from_floor: f32,
    pub floor_normal: Vec3,
    pub speed: f32,
    pub defacto_speed: f32,
    pub kicked_wall: Option<Entity>,
    pub jump_stage: u8,
}

#[derive(Default)]
pub enum IndexPointer {
    #[default]
    Empty,
    FindAt(usize),
    WaitFor(usize),
}
