use crate::GameState;
use bevy::prelude::*;
use bevy_ldtk::{LdtkMap, LdtkMapBundle, LdtkMapConfig};
use rayon::prelude::*;

pub struct MainCamera;

#[derive(Default, Clone)]
pub struct WorldState {
    pub map_loaded: bool,
    pub collisions_loaded: bool,
    pub level: usize,
    pub requested_level: usize,
    pub world: Option<Entity>,
    pub collisions: Vec<Vec2>,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(draw.system())
            .add_system(setup_collisions.system());
    }
}

pub fn setup_collisions(
    map_query: Query<&Handle<LdtkMap>>,
    map_assets: Res<Assets<LdtkMap>>,
    mut game_state: ResMut<GameState>,
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
            let level_idx = game_state.world_state.level;

            // Get the level from the project
            let level = &map.project.levels[level_idx];

            // Find the collision layer
            let collision_layer = level
                .layer_instances
                .as_ref() // get a reference to the layer instances
                .unwrap() // Unwrap the option ( this could be None, if there are no layers )
                .iter() // Iterate over the layers
                .find(|&x| x.__identifier == "Collisions") // Filter on the one we want
                .unwrap(); // Unwrap it ( would be None if it could not find a layer "Collisions" )

            // Calculate collider center coordinates
            game_state.world_state.collisions = collision_layer
                .int_grid_csv
                .clone()
                .into_par_iter()
                .enumerate()
                .filter(|(_, object)| *object != 2 && *object != 0)
                .map(|(i, _)| {
                    one_d_to_two_d_coordinate(i as f32, collision_layer.__c_wid as f32, 16.0, 16.0)
                })
                .collect();
            game_state.world_state.collisions_loaded = true;
        }
    }
}

fn one_d_to_two_d_coordinate(
    coordinate: f32,
    row_length: f32,
    tile_width: f32,
    tile_height: f32,
) -> Vec2 {
    Vec2::new(
        ((coordinate % row_length).round() * tile_width + 11.0).round() as f32,
        (-((coordinate / row_length).round() * tile_height + 8.0)).round() as f32,
    )
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
