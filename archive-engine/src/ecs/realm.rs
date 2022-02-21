use std::{collections::BTreeMap, time::Duration};

use super::*;
use crate::*;

use bimap::BiBTreeMap;

use hecs::*;

// 1 / 60
pub const TICK_RATE: Num = mk_num!(0.016666666666666666);
pub const TICK_DURATION: Duration =
    Duration::from_micros(conversions::num_to_umicros_cast(TICK_RATE));

#[derive(Default)]
pub struct Realm {
    // only ecs can access world because it has invariants to uphold
    pub(super) world: World,
    // pub(crate) input_query: PreparedQuery<InputQ>,
    // pub(crate) movement_query: PreparedQuery<MovementQ>,
    // pub(crate) health_query: PreparedQuery<HealthQ>,
    // pub(crate) death_query: PreparedQuery<DeadQ>,
    pub tick: u64,

    pub(crate) repl_token_pool: ReplPool,
    pub(crate) ent_map: BiBTreeMap<ReplToken, Entity>,
    pub(crate) player_map: BTreeMap<rtc::ClientId, Entity>,
}

impl Realm {
    pub fn new() -> Self {
        Realm::default()
    }
    pub fn run_systems(&mut self) {
        input_system(self);
        movement_system(self);
        health_system(self);
        // death_system(self);
    }
    // TODO add the player and make this relative
    pub(super) fn calc_priority_inc(&mut self, ent: Entity) -> Priority {
        // expects a replication component
        let repl_data = self.get_mut::<&Replicated>(ent).unwrap();

        if repl_data.blueprint.is_none() {
            return 1;
        }
        match repl_data.blueprint.unwrap() {
            Blueprint::Player => 1000,
            Blueprint::Bullet => 100,
            Blueprint::Static => 10,
        }
    }
    // idk
    pub(super) fn calc_deleted_priority_accum(&self) -> Priority {
        10
    }

    pub(super) fn entity_for_token(&self, token: &ReplToken) -> Entity {
        *self.ent_map.get_by_left(token).unwrap()
    }

    pub fn spawn(&mut self, bundle: impl DynamicBundle) -> Entity {
        let ent = self.world.spawn(bundle);
        if let Some(_) = self.get_mut::<&Replicated>(ent) {
            let token = self.repl_token_pool.alloc();
            self.ent_map.insert(token, ent);
        }
        ent
    }
    pub fn despawn(&mut self, ent: Entity) {
        if let Some(_) = self.get_mut::<&Replicated>(ent) {
            let token = self.ent_map.remove_by_right(&ent).unwrap().0;
            self.repl_token_pool.free(token);
        }
        self.world.despawn(ent).unwrap();
    }
    pub fn get_mut<Q: Query>(&mut self, ent: Entity) -> Option<QueryItem<Q>> {
        utils::world_get_mut::<Q>(&mut self.world, ent)
    }
    pub fn query_mut<Q: Query>(&mut self) -> QueryMut<Q> {
        self.world.query_mut()
    }
}
