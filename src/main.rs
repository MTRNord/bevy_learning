use crate::plugins::{
    world::{MainCamera, WorldState},
    PlayerPlugin, WorldPlugin,
};
use bevy::render::camera::{OrthographicProjection, ScalingMode};
use bevy::{prelude::*, window::WindowMode};
use bevy_ldtk::*;

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
            vsync: false,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        //.add_plugin(PrintDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //.add_system(PrintDiagnosticsPlugin::print_diagnostics_system.system())
        //.add_plugin(KurinjiPlugin::default())
        .add_startup_system(setup.system())
        .add_plugin(PlayerPlugin)
        .add_plugin(WorldPlugin)
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
            ..OrthographicCameraBundle::new_2d()
        })
        .with(MainCamera);

    /*// TODO move to file
    kurinji
        .bind_keyboard_pressed(KeyCode::A, "MOVE_LEFT")
        .bind_keyboard_pressed(KeyCode::D, "MOVE_RIGHT")
        .bind_keyboard_pressed(KeyCode::Space, "JUMP")
        .bind_keyboard_pressed(KeyCode::LShift, "SHIFT_MODIFIER")
        .bind_keyboard_pressed(KeyCode::RShift, "SHIFT_MODIFIER");*/
}
