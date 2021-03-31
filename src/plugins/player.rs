use crate::entities::common::Health;
use crate::entities::markers::{Movable, Player, Wall};
use crate::entities::player::{PlayerBundle, PlayerName, PlayerXp};
use crate::plugins::world::{GridLocation, MainCamera, SPRITE_HEIGHT, SPRITE_WIDTH};
use crate::GameState;
use bevy::prelude::*;
use std::collections::HashMap;

pub const PLAYER_START: GridLocation = GridLocation(2, -2);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(player_movement.system())
            .add_system(setup_player.system())
            .add_startup_system(setup_player.system());
    }
}

fn setup_player(
    mut commands: &mut Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    game_state: ResMut<GameState>,
) {
    if !game_state.spawned {
        setup_player_internal(
            PLAYER_START,
            &mut commands,
            asset_server,
            texture_atlases,
            game_state,
        );
    }
}

fn setup_player_internal(
    grid_location: GridLocation,
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut game_state: ResMut<GameState>,
) {
    let texture_handle = asset_server.load("tilesets/0x72_DungeonTilesetII_v1.3.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 32, 32);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // Spawn player
    commands
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 362,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(
                SPRITE_WIDTH * grid_location.0 as f32,
                SPRITE_HEIGHT * grid_location.1 as f32,
                0.0,
            )),
            texture_atlas: texture_atlas_handle,
            ..Default::default()
        })
        .with(grid_location)
        .with(Movable)
        .with(PlayerBundle {
            xp: PlayerXp(0.0),
            name: PlayerName("Player 1".into()),
            health: Health { hp: 100.0 },
            _p: Player,
        });
    game_state.spawned = true;
}

#[allow(clippy::type_complexity)]
fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut wall_query: Query<(Entity, &Wall, &GridLocation)>,
    mut set: QuerySet<(
        Query<(Entity, &Movable, &mut GridLocation)>,
        Query<(Entity, &PlayerBundle, &GridLocation)>,
    )>,
    mut camera: Query<&mut GridLocation, With<MainCamera>>,
) {
    let _shift = keyboard_input.pressed(KeyCode::LShift) || keyboard_input.pressed(KeyCode::RShift);
    let _ctrl =
        keyboard_input.pressed(KeyCode::LControl) || keyboard_input.pressed(KeyCode::RControl);

    let delta = {
        let mut delta = GridLocation(0, 0);
        if keyboard_input.just_pressed(KeyCode::A) {
            delta = GridLocation(-1, 0);
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            delta = GridLocation(1, 0);
        }
        if keyboard_input.just_pressed(KeyCode::S) {
            delta = GridLocation(0, -1);
        }
        if keyboard_input.just_pressed(KeyCode::W) {
            delta = GridLocation(0, 1);
        }
        if delta == GridLocation(0, 0) {
            return;
        }
        delta
    };

    let immovables: HashMap<GridLocation, Entity> = {
        let mut tmp = HashMap::new();
        for (wall_entity, _wall, wall_grid_location) in wall_query.iter_mut() {
            tmp.insert(
                GridLocation(wall_grid_location.0, wall_grid_location.1),
                wall_entity,
            );
        }
        tmp
    };

    let movables: HashMap<GridLocation, Entity> = {
        let mut tmp = HashMap::new();
        for (movable_entity, _movable, grid_location) in set.q0_mut().iter_mut() {
            tmp.insert(
                GridLocation(grid_location.0, grid_location.1),
                movable_entity,
            );
        }
        tmp
    };

    let mut to_move: Vec<Entity> = vec![];
    let mut last_pos = None;

    for (_player_entity, _player, player_grid_location) in set.q1().iter() {
        let mut tmp_to_move = vec![];

        let mut current_loc = *player_grid_location;
        //prevent block skips
        if let Some(pos) = last_pos {
            if pos == current_loc {
                continue;
            }
        }

        while let Some(movable) = movables.get(&current_loc) {
            tmp_to_move.push(*movable);
            current_loc = current_loc + delta;
        }
        if let Some(_immovable) = immovables.get(&current_loc) {
            continue;
        }
        last_pos = Some(current_loc);
        to_move.append(&mut tmp_to_move);
    }

    let mut camera_grid_location = camera.iter_mut().next().unwrap();
    for loc in to_move {
        let mut grid_location: Mut<GridLocation> = set.q0_mut().get_component_mut(loc).unwrap();
        *grid_location = *grid_location + delta;
        *camera_grid_location = *camera_grid_location + delta;
    }
}
