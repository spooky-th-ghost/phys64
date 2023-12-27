use bevy::prelude::*;

mod movement;

#[derive(Component)]
pub struct Player;

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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(movement::PlayerMovementPlugin)
            .add_systems(Update, update_player_data);
    }
}

fn update_player_data(
    mut player_data: ResMut<PlayerData>,
    player_query: Query<&Transform, With<Player>>,
) {
    for transform in &player_query {
        player_data.player_position = transform.translation;
    }
}
