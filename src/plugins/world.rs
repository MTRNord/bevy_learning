use crate::entities::markers::Wall;
use crate::GameState;
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

fn setup_wall(
    grid_location: GridLocation,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
            transform: Transform::from_translation(Vec3::new(
                SPRITE_WIDTH * grid_location.0 as f32,
                SPRITE_HEIGHT * grid_location.1 as f32,
                -1.0,
            )),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH, SPRITE_HEIGHT)),
            ..Default::default()
        })
        .with(grid_location)
        .with(Wall);
}

fn draw(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if game_state.world_state.map_loaded
        && game_state.world_state.level == game_state.world_state.requested_level
    {
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
            for y in -8..8 {
                for x in -8..8 {
                    let full_x = (chunk_x * 16) + x;
                    let full_y = (chunk_y * 16) + y;
                    let coord = GridLocation(full_x, full_y);
                    /*
                                    // Get noise value:
                    f = noise1d (x/width);
                    h = f * height + (height/2);  // scale and center at screen-center
                                     */
                    if full_x == 0 && full_y == 0 {
                        continue;
                    }
                    let f =
                        noise.get([(full_x as f32 / 16.0) as f64, (full_y as f32 / 16.0) as f64]);
                    let noise_value = f * 16.0 + (16.0 / 2.0);

                    // scale and center at screen-center
                    // TODO see https://stackoverflow.com/a/10225718
                    if noise_value > 3.0 {
                        setup_wall(coord, commands, &mut materials);
                    }
                }
            }
        }
    }

    game_state.world_state.level = game_state.world_state.requested_level;
    game_state.world_state.map_loaded = true;
}
