use super::*;
use crate::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blueprint {
    Player,
    Bullet,
    Static,
}
derive_math_components! {
    pub struct Position {
        pub xy: V2,
        pub zed: Zed,
    }
    pub struct Rotation {
        pub rad: R,
    }
    pub struct Velocity {
        pub xy: V2,
    }
    pub struct Scale {
        pub xy: V2,
    }
}

pub type MovementQ = (&'static mut Position, &'static Velocity);

pub fn movement_system(realm: &mut Realm) {
    let Realm {
        world,
        movement_query,
        ..
    } = realm;
    for (_id, (pos, vel)) in movement_query.query_mut(world) {
        pos.xy += vel.xy;
    }
}

derive_components! {
    pub struct Camera {}

    pub struct Player {
        _id: u8,
    }
    // replicated player inputs
    pub struct Input {
        movement: V2,
        aim: R,
    }
}

pub type InputQ = (&'static mut Velocity, &'static mut Rotation, &'static Input);

pub fn input_system(realm: &mut Realm) {
    let Realm {
        world, input_query, ..
    } = realm;
    for (_id, (vel, rot, input)) in input_query.query_mut(world) {
        vel.xy += input.movement;
        rot.rad = input.aim;
    }
}

derive_components! {
    pub struct Bullet {}
    pub struct Dead {}
}
pub(super) type HealthVal = std::num::Wrapping<u16>;
derive_math_components! {
    pub struct Health {
        value: HealthVal,
    }
}
impl Health {
    pub fn new(value: u16) -> Self {
        Health {
            value: std::num::Wrapping(value),
        }
    }
}

pub type HealthQ = &'static Health;

pub fn health_system(realm: &mut Realm) {
    let Realm {
        world,
        health_query,
        ..
    } = realm;
    let mut to_despawn = Vec::new();
    for (id, health) in health_query.query_mut(world) {
        if health.value.0 == 0 {
            to_despawn.push(id);
        }
    }
    for id in to_despawn {
        world.insert_one(id, Dead {}).unwrap();
    }
}

pub type DeadQ = &'static Dead;
pub fn death_system(realm: &mut Realm) {
    let Realm {
        world, death_query, ..
    } = realm;
    let mut to_remove = Vec::new();
    for (id, _) in death_query.query_mut(world) {
        to_remove.push(id);
    }
    for id in to_remove {
        world.despawn(id).unwrap();
    }
}
