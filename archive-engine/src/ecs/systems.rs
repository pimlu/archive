use crate::*;

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Position {
    xy: P2,
    _zed: Zed,
}
#[derive(Component)]
pub struct Rotation {
    _rad: R,
}
#[derive(Component)]
pub struct Velocity {
    xy: V2,
}
#[derive(Component)]
pub struct Scale {
    _xy: V2,
}

pub fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query.iter_mut() {
        position.xy += velocity.xy;
    }
}

#[derive(Component)]
pub struct Camera {}

#[derive(Component)]
pub struct Player {
    _id: u8,
}
#[derive(Component)]
pub struct Input {
    _aim: V2,
    _movement: V2,
}
#[derive(Component)]
pub struct Bullet {
    _id: u16,
}
#[derive(Component)]
pub struct Health {
    value: u16,
}

pub fn death(mut commands: Commands, query: Query<(Entity, &Health), Changed<Health>>) {
    for (entity, health) in query.iter() {
        if health.value == 0 {
            commands.entity(entity).despawn();
        }
    }
}
