use crate::GameState;
use bevy::prelude::*;
use bevy_ldtk::{LdtkMap, LdtkMapBundle, LdtkMapConfig};

pub struct MainCamera;

#[derive(Default, Clone)]
pub struct WorldState {
    pub map_loaded: bool,
    pub collisions_loaded: bool,
    pub level: usize,
    pub requested_level: usize,
    pub world: Option<Entity>,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(draw.system())
            .add_system(setup_collisions.system());
        // .add_system(coordinate_at_mouse.system());
    }
}

pub fn setup_collisions(
    map_query: Query<&Handle<LdtkMap>>,
    map_assets: Res<Assets<LdtkMap>>,
    game_state: ResMut<GameState>,
) {
    if game_state.world_state.collisions_loaded
        && game_state.world_state.level == game_state.world_state.requested_level
    {
        return;
    }
    // Loop through all of the maps
    for map_handle in map_query.iter() {
        // We have to `if let` here because asset server may not have finished loading
        // the map yet.
        if let Some(map) = map_assets.get(map_handle) {
            // Do something with map data
        }
    }
}

fn draw(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.world_state.map_loaded
        && game_state.world_state.level == game_state.world_state.requested_level
    {
        return;
    }

    if let Some(world) = game_state.world_state.world {
        commands.remove::<LdtkMapBundle>(world);
    }

    commands // Spawn a map bundle
        .spawn(LdtkMapBundle {
            // Specify the path to the map asset to load
            map: asset_server.load("map.ldtk"),
            config: LdtkMapConfig {
                // Automatically set the clear color to the LDtk background color
                set_clear_color: true,
                // You can specify a scale or leave it set to 1 for 1 to 1 pixel size
                scale: 1.0,
                // Set which level to load out of the map or leave it to 0 for the default level
                level: game_state.world_state.requested_level,
                // Tell the map to center around it's `Transform` instead of putting the top-left
                // corner of the map at the origin `Transform`.
                center_map: false,
            },
            ..Default::default()
        });

    game_state.world_state.world = commands.current_entity();

    game_state.world_state.level = game_state.world_state.requested_level;
    game_state.world_state.map_loaded = true;
}
