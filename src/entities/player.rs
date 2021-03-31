use crate::entities::common::Health;
use crate::entities::markers::Player;
use bevy::prelude::*;

#[derive(Debug)]
pub struct PlayerXp(pub f64);

pub struct PlayerName(pub String);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub xp: PlayerXp,
    pub name: PlayerName,
    pub health: Health,
    pub _p: Player,
}
