use crate::entities::common::Health;
use crate::entities::markers::Player;
use crate::entities::player::{PlayerBundle, PlayerName, PlayerXp};
use crate::plugins::world::MainCamera;
use crate::GameState;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_ldtk::LdtkMap;
use std::convert::TryInto;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(player_movement.system())
            .add_system(spawn_player.system());
    }
}

fn spawn_player(
    mut game_state: ResMut<GameState>,
    commands: &mut Commands,
    map_query: Query<&Handle<LdtkMap>>,
    map_assets: Res<Assets<LdtkMap>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if game_state.spawned {
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

            // Find the entities layer
            let entities_layer = level
                .layer_instances
                .as_ref() // get a reference to the layer instances
                .unwrap() // Unwrap the option ( this could be None, if there are no layers )
                .iter() // Iterate over the layers
                .find(|&x| x.__identifier == "Entities") // Filter on the one we want
                .unwrap(); // Unwrap it ( would be None if it could not find a layer "Entities" )

            // Get the specific entity you want
            let player_start = entities_layer
                .entity_instances
                .iter() // Iterate over our entities in the layer
                .find(|x| x.__identifier == "Player") // Find the one we want
                .unwrap(); // Unwrap it

            // Get the number of layers in the map and add one to it: this is how high we need to
            // spawn the player so that he is on top of all the maps
            let player_z = level.layer_instances.as_ref().unwrap().len() as f32 + 1.0;

            // Do something with map data
            let entities = &mut map.project.defs.entities.iter();
            let tilesets = &mut map.project.defs.tilesets.iter();

            let player_position = Vec3::new(
                // The player x position is the entity's x position from the map data
                player_start.px[0] as f32,
                // The player y position is the entity's y position from the map data, but
                // multiplied by negative one because in the LDtk map +y means down and not up.
                -(player_start.px[1] as f32 - (player_start.height as f32 / 2.0)),
                // Spawn the player with the z value we determined earlier
                player_z,
            );

            // Move camera initially
            // assuming there is exactly one main camera entity, so this is OK
            let mut camera_transform = camera.iter_mut().next().unwrap();
            camera_transform.translation = player_position;

            // Find player assets
            if let Some(player_entity) = entities.find(|x| x.identifier == "Player") {
                if let Some(tileset_id) = player_entity.tileset_id {
                    if let Some(tileset) = tilesets.find(|x| x.uid == tileset_id) {
                        if let Some(tileset_texture) = map.tile_sets.get(&tileset.identifier) {
                            let tile_id = player_entity.tile_id;
                            let texture_atlas = TextureAtlas::from_grid(
                                (*tileset_texture).clone(),
                                Vec2::new(
                                    tileset.tile_grid_size as f32,
                                    tileset.tile_grid_size as f32,
                                ),
                                (tileset.px_wid / tileset.tile_grid_size)
                                    .try_into()
                                    .unwrap(),
                                (tileset.px_hei / tileset.tile_grid_size)
                                    .try_into()
                                    .unwrap(),
                            );

                            let texture_atlas_handle = texture_atlases.add(texture_atlas);
                            commands
                                .spawn(SpriteSheetBundle {
                                    sprite: TextureAtlasSprite {
                                        index: tile_id.unwrap() as u32,
                                        ..Default::default()
                                    },
                                    texture_atlas: texture_atlas_handle,
                                    transform: Transform::from_translation(player_position),
                                    ..Default::default()
                                })
                                .with(PlayerBundle {
                                    xp: PlayerXp(0),
                                    name: PlayerName("Player1".into()),
                                    health: Health { hp: 100.0 },
                                    _p: Player,
                                });
                            game_state.spawned = true;
                        } else {
                            println!("Error: Cannot find tileset as asset!");
                        }
                    } else {
                        println!("Error: Cannot find tilset with wanted id!");
                    }
                } else {
                    println!("Error: Cannot find player tileset!");
                }
            } else {
                println!("Error: Cannot find player in map!");
            }
        }
    }
}

fn player_movement(
    time: Res<Time>,
    game_state: ResMut<GameState>,
    map_query: Query<&Handle<LdtkMap>>,
    map_assets: Res<Assets<LdtkMap>>,
    input: Res<Input<KeyCode>>,
    mut player: Query<(&PlayerBundle, &mut Transform)>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    let shift = input.pressed(KeyCode::LShift) || input.pressed(KeyCode::RShift);
    let _ctrl = input.pressed(KeyCode::LControl) || input.pressed(KeyCode::RControl);
    // TODO make sure we queue inputs and do all of them

    let mut direction = Vec3::zero();

    if shift {
        if input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0) * 5.;
        }

        if input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0) * 5.;
        }
    } else {
        if input.pressed(KeyCode::A) {
            direction -= Vec3::new(2.5, 0.0, 0.0);
        }

        if input.pressed(KeyCode::D) {
            direction += Vec3::new(2.5, 0.0, 0.0);
        }
    }

    if input.pressed(KeyCode::Space) {
        direction += Vec3::new(0.0, 1.0, 0.0);
    }

    if input.pressed(KeyCode::S) {
        direction -= Vec3::new(0.0, 1.0, 0.0);
    }

    if direction != Vec3::zero() {
        let translation = time.delta_seconds() * direction * 40.;

        // assuming there is exactly one player entity, so this is OK
        if let Some((_, mut player_transform)) = player.iter_mut().next() {
            if can_move_to_requested_coordinate(
                &map_query,
                &map_assets,
                &game_state,
                player_transform.translation + translation,
                16,
                16,
            ) {
                player_transform.translation += translation;
            }
        }
        // assuming there is exactly one camera entity, so this is OK
        if let Some(mut camera_transform) = camera.iter_mut().next() {
            if can_move_to_requested_coordinate(
                &map_query,
                &map_assets,
                &game_state,
                camera_transform.translation + translation,
                16,
                16,
            ) {
                camera_transform.translation += translation;
            }
        }
    }
}

fn can_move_to_requested_coordinate(
    map_query: &Query<&Handle<LdtkMap>>,
    map_assets: &Res<Assets<LdtkMap>>,
    game_state: &ResMut<GameState>,
    coordinate: Vec3,
    width: i32,
    height: i32,
) -> bool {
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

            // Do a bounding box check. Bounding boxes are 16*16 from center of the object
            for (i, object) in collision_layer.int_grid_csv.iter().enumerate() {
                // Skip ladders
                if *object == 2 || *object == 0 {
                    continue;
                }

                // Check moves
                let object_coordinate =
                    one_d_to_two_d_coordinate(i as f32, collision_layer.__c_wid as f32, 16.0, 16.0);

                if check_intersection(
                    coordinate.xy(),
                    object_coordinate,
                    Vec2::new(width as f32, height as f32),
                    Vec2::new(16.0, 16.0),
                ) {
                    // We found an collision
                    return false;
                }
            }
        }
    }
    true
}

fn check_intersection(a: Vec2, b: Vec2, a_size: Vec2, b_size: Vec2) -> bool {
    ((a.x - b.x).abs() * 2.0 < (a_size.x + b_size.x))
        && ((a.y - b.y).abs() * 2.0 < (a_size.y + b_size.y))
}

fn two_d_to_one_d_coordinate(coordinate: Vec3, row_length: f32) -> f32 {
    (coordinate.y * row_length) + (-coordinate.x)
}

fn one_d_to_two_d_coordinate(
    coordinate: f32,
    row_length: f32,
    tile_width: f32,
    tile_height: f32,
) -> Vec2 {
    Vec2::new(
        (((coordinate % row_length) * tile_width) + (tile_width / 2.0)).round() as f32,
        ((-(coordinate / row_length * tile_height)) - (tile_height / 2.0)).round() as f32,
    )
}
