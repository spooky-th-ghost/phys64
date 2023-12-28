use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
mod types;
use types::*;
mod camera;
use camera::*;
mod input;
use input::*;
mod player;
use player::*;
mod movement;
use movement::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .register_type::<Momentum>()
        .register_type::<GroundSensor>()
        .add_plugins((MovementPlugin, InputPlugin, CameraPlugin, PlayerPlugin))
        .insert_resource(Gravity::new(0.4))
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0))
        .insert_resource(PlayerData::default())
        .add_systems(Startup, setup)
        .configure_sets(
            FixedUpdate,
            (
                EngineSystemSet::Input,
                EngineSystemSet::CalculateMomentum,
                EngineSystemSet::ApplyMomentum,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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
            snap_to_ground: Some(CharacterLength::Absolute(10.0)),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(0.5),
                min_width: CharacterLength::Absolute(0.25),
                include_dynamic_bodies: true,
            }),
            ..default()
        },
        MoveDirection::default(),
        Momentum::default(),
        Speed::default(),
        Forces::default(),
        GravityAffected,
        GroundSensor::default(),
        InputBuffer::default(),
        Jumper::default(),
        InputListenerBundle::input_map(),
    ));

    commands.spawn((
        PbrBundle {
            material: materials.add(Color::WHITE.into()),
            mesh: meshes.add(shape::Box::new(50.0, 0.5, 50.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -2.0, 0.0))
                .with_rotation(Quat::from_axis_angle(Vec3::Z, 30.0_f32.to_radians())),
            ..default()
        },
        Collider::cuboid(25.0, 0.25, 25.0),
        RigidBody::Fixed,
    ));
}
