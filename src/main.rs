use crate::plugins::world::{SPRITE_HEIGHT, SPRITE_WIDTH};
use crate::plugins::{
    player::PLAYER_START,
    world::{MainCamera, WorldState},
    PlayerPlugin, WorldPlugin,
};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::camera::{OrthographicProjection, ScalingMode};
use bevy::{prelude::*, window::WindowMode};

mod entities;
mod plugins;

#[derive(Default, Clone)]
pub struct GameState {
    pub spawned: bool,
    pub world_state: WorldState,
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Random test game".to_string(),
            width: 1024.,
            height: 720.,
            vsync: true,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system_to_stage(StartupStage::PreStartup, setup.system())
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerPlugin)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn setup(commands: &mut Commands /*mut kurinji: ResMut<Kurinji>*/) {
    commands
        // Spawn a camera
        .spawn(OrthographicCameraBundle {
            orthographic_projection: OrthographicProjection {
                far: 1024.0,  // This gives us 1024 layers,
                scale: 150.0, // How many pixels high in the game
                scaling_mode: ScalingMode::FixedVertical,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(
                SPRITE_WIDTH * PLAYER_START.0 as f32,
                SPRITE_HEIGHT * PLAYER_START.1 as f32,
                0.0,
            )),
            ..OrthographicCameraBundle::new_2d()
        })
        .with(PLAYER_START)
        .with(MainCamera);
}
