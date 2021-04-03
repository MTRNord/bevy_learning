use crate::plugins::world::{SPRITE_HEIGHT, SPRITE_WIDTH};
use crate::plugins::{
    player::PLAYER_START,
    world::{MainCamera, WorldState},
    PlayerPlugin, WorldPlugin,
};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::camera::{OrthographicProjection, ScalingMode};
use bevy::{prelude::*, window::WindowMode};
use std::collections::HashMap;

mod entities;
mod plugins;

#[derive(Default)]
pub struct AssetsLoading(Vec<HandleUntyped>);

#[derive(Default, Clone)]
pub struct GameState {
    pub spawned: bool,
    pub world_state: WorldState,
    pub asset_map: HashMap<String, Handle<Texture>>,
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Random test game".to_string(),
            width: 1024.,
            height: 720.,
            vsync: false,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .init_resource::<GameState>()
        .init_resource::<AssetsLoading>()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system_to_stage(StartupStage::PreStartup, setup.system())
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerPlugin)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut loading: ResMut<AssetsLoading>,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle {
            orthographic_projection: OrthographicProjection {
                far: 1024.0,  // This gives us 1024 layers,
                scale: 200.0, // How many pixels high in the game
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
        .insert(PLAYER_START)
        .insert(MainCamera);

    let sunny_texture_handle = asset_server.load("tilesets/SunnyLand_by_Ansimuz-extended.png");
    loading.0.push(sunny_texture_handle.clone_untyped());
    game_state
        .asset_map
        .insert("Sunnyland".into(), sunny_texture_handle);
    let dungeon_tileset_texture_handle =
        asset_server.load("tilesets/0x72_DungeonTilesetII_v1.3.png");
    loading
        .0
        .push(dungeon_tileset_texture_handle.clone_untyped());
    game_state
        .asset_map
        .insert("DungeonTileset".into(), dungeon_tileset_texture_handle);
}
