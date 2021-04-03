use crate::entities::markers::Wall;
use crate::plugins::player::PLAYER_START;
use crate::{AssetsLoading, GameState};
use bevy::asset::LoadState;
use bevy::prelude::*;
use noise::{NoiseFn, OpenSimplex, Seedable};
use rand::Rng;
use std::ops;

pub struct MainCamera;

#[derive(Default, Clone)]
pub struct WorldState {
    pub map_loaded: bool,
    pub collisions_loaded: bool,
    pub level: usize,
    pub requested_level: usize,
    pub world: Option<Entity>,
    pub collisions: Vec<Vec2>,
    pub world_noise: OpenSimplex,
    pub seed: Option<u32>,
}

pub struct WorldPlugin;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct GridLocation(pub i32, pub i32);

impl ops::Add<GridLocation> for GridLocation {
    type Output = GridLocation;

    fn add(self, rhs: GridLocation) -> Self::Output {
        GridLocation(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl From<Vec2> for GridLocation {
    fn from(vec: Vec2) -> Self {
        Self {
            0: vec.x.round() as i32,
            1: vec.y.round() as i32,
        }
    }
}

impl From<[f64; 2]> for GridLocation {
    fn from(coords: [f64; 2]) -> Self {
        Self {
            0: coords[0] as i32,
            1: coords[1] as i32,
        }
    }
}

impl From<GridLocation> for [f64; 2] {
    fn from(loc: GridLocation) -> Self {
        [loc.0 as f64, loc.1 as f64]
    }
}

const LERP_LAMBDA: f32 = 5.0;

fn render_grid_location_to_transform(
    time: Res<Time>,
    mut query: Query<(&GridLocation, &mut Transform)>,
) {
    for (grid_location, mut transform) in query.iter_mut() {
        let target_x = SPRITE_WIDTH * grid_location.0 as f32;
        transform.translation.x = transform.translation.x
            * (1.0 - LERP_LAMBDA * time.delta_seconds())
            + target_x * LERP_LAMBDA * time.delta_seconds();
        let target_y = SPRITE_WIDTH * grid_location.1 as f32;
        transform.translation.y = transform.translation.y
            * (1.0 - LERP_LAMBDA * time.delta_seconds())
            + target_y * LERP_LAMBDA * time.delta_seconds();
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(draw.system())
            .add_system(render_grid_location_to_transform.system());
    }
}

pub const SPRITE_WIDTH: f32 = 16.0;
pub const SPRITE_HEIGHT: f32 = 16.0;

#[derive(Bundle)]
struct WallBundle {
    pub grid_location: GridLocation,
    pub _wall: Wall,

    #[bundle]
    pub sprite: SpriteSheetBundle,
}

fn setup_wall(
    grid_location: GridLocation,
    game_state: &ResMut<GameState>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    textures: &Res<Assets<Texture>>,
) -> WallBundle {
    let texture_handle = game_state.asset_map.get("Sunnyland").unwrap();
    let texture: &Texture = textures.get(texture_handle.id).unwrap();
    let cols = texture.size.width / 16;
    let rows = texture.size.height / 16;
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle.clone(),
        Vec2::new(17.0, 17.0),
        cols as usize,
        rows as usize,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    WallBundle {
        grid_location,
        _wall: Wall,
        sprite: SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 48,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(
                SPRITE_WIDTH * grid_location.0 as f32,
                SPRITE_HEIGHT * grid_location.1 as f32,
                -1.0,
            )),
            texture_atlas: texture_atlas_handle,
            ..Default::default()
        },
    }
}

fn draw(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    textures: Res<Assets<Texture>>,
) {
    if game_state.world_state.map_loaded
        && game_state.world_state.level == game_state.world_state.requested_level
    {
        return;
    }

    let mut ready = true;
    for handle in loading.0.iter() {
        match server.get_load_state(handle) {
            LoadState::Failed => {
                ready = false;
            }
            LoadState::Loaded => {}
            _ => {
                ready = false;
            }
        }
    }

    if !ready {
        return;
    }

    if game_state.world_state.seed.is_none() {
        let mut rng = rand::thread_rng();
        game_state.world_state.seed = Some(rng.gen());
    }

    game_state.world_state.world_noise =
        OpenSimplex::new().set_seed(game_state.world_state.seed.unwrap());
    let noise = &game_state.world_state.world_noise;

    for chunk_y in -2..2 {
        for chunk_x in -2..2 {
            let chunk = generate_chunk(
                chunk_x,
                chunk_y,
                noise,
                &game_state,
                &mut texture_atlases,
                &textures,
            );
            commands.spawn_batch(chunk);
        }
    }

    game_state.world_state.level = game_state.world_state.requested_level;
    game_state.world_state.map_loaded = true;
}

fn generate_chunk(
    chunk_x: i32,
    chunk_y: i32,
    noise: &OpenSimplex,
    game_state: &ResMut<GameState>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    textures: &Res<Assets<Texture>>,
) -> Vec<WallBundle> {
    let mut bundles = Vec::new();
    for y in -8..8 {
        for x in -8..8 {
            let full_x = (chunk_x * 16) + x;
            let full_y = (chunk_y * 16) + y;
            let coord = GridLocation(full_x, full_y);

            if full_x == PLAYER_START.0 && full_y == PLAYER_START.1 {
                continue;
            }
            let f = noise.get([(full_x as f32 / 16.0) as f64, (full_y as f32 / 16.0) as f64]);
            let noise_value = f * 16.0 + (16.0 / 2.0);

            if noise_value > 4.8 {
                bundles.push(setup_wall(coord, game_state, texture_atlases, textures));
            }
        }
    }
    bundles
}
