use crate::*;

use bevy_ecs::prelude::*;

#[derive(Component)]
struct Sprite {
    color: [u8; 4],
    id: u16,
    z: u8,
}