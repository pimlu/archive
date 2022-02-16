use super::*;
use crate::*;

use std::collections::BTreeMap;

use hecs::*;

pub struct Snapshot {
    pub(super) world: World,
    // map from repl keys to entities that exist in this snapshot specifically
    pub(super) ent_map: BTreeMap<ReplKey, Entity>,
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            world: World::new(),
            ent_map: BTreeMap::new(),
        }
    }

    pub(super) fn entity_for_token(&self, token: &ReplToken) -> Option<Entity> {
        self.ent_map.get(&token.key()).map(|e| *e)
    }

    pub(super) fn server_augment(self, meta: ServerSnapshotMeta) -> ServerSnapshot {
        let ServerSnapshotMeta {
            mut token_map,
            priority_map,
        } = meta;

        // the token_map is too large on its own, and contains unnecessary
        // tokens from merge_diff_map. strip unused tokens.
        // TODO use drain_filter when it is stabilized
        let mut to_remove = Vec::new();

        for &repl_key in token_map.keys() {
            if !self.ent_map.contains_key(&repl_key) && !priority_map.contains_key(&repl_key) {
                to_remove.push(repl_key);
            }
        }

        for repl_key in to_remove {
            token_map.remove(&repl_key).unwrap();
        }

        ServerSnapshot {
            inner: self,
            meta: ServerSnapshotMeta {
                token_map,
                priority_map,
            },
        }
    }

    fn clone_builder(world: &mut World, ent: Entity) -> EntityBuilder {
        let mut builder = EntityBuilder::new();
        map_all_components!(|Struct| {
            if let Ok(comp) = world.get_mut::<Struct>(ent) {
                builder.add::<Struct>(*comp);
            }
        });
        builder
    }

    pub fn clone_mut(&mut self) -> Self {
        let mut new_world = World::new();
        // TODO I think there is a faster way using archetypes() to clone this

        let mut entities = Vec::new();

        for (ent, ()) in self.world.query_mut::<()>() {
            entities.push(ent);
        }

        for ent in entities {
            let mut builder = Snapshot::clone_builder(&mut self.world, ent);
            new_world.spawn(builder.build());
        }

        Snapshot {
            world: new_world,
            ent_map: self.ent_map.clone(),
        }
    }

    pub fn get_mut<Q: Query>(&mut self, ent: Entity) -> Option<QueryItem<Q>> {
        utils::world_get_mut::<Q>(&mut self.world, ent)
    }
}

pub(super) type ReplTokenMap = BTreeMap<ReplKey, ReplToken>;
#[derive(Clone)]
pub struct ServerSnapshotMeta {
    // allows you to recover tokens from ReplKeys
    pub(super) token_map: ReplTokenMap,
    // for any tokens that have accumulated priority, they are added here
    // if the client is fast/cooperative, this should be normally empty
    pub(super) priority_map: BTreeMap<ReplKey, Priority>,
}
impl ServerSnapshotMeta {
    pub fn new() -> Self {
        Self {
            token_map: BTreeMap::new(),
            priority_map: BTreeMap::new(),
        }
    }
}
pub struct ServerSnapshot {
    pub(super) inner: Snapshot,
    pub(super) meta: ServerSnapshotMeta,
}

impl ServerSnapshot {
    pub fn new() -> Self {
        Self {
            inner: Snapshot::new(),
            meta: ServerSnapshotMeta::new(),
        }
    }

    pub(super) fn entity_for_token(&self, token: &ReplToken) -> Option<Entity> {
        self.inner.entity_for_token(token)
    }
    pub(super) fn priority_accum_for_token(&self, token: &ReplToken) -> Option<Priority> {
        self.meta.priority_map.get(&token.key()).copied()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_mut() {
        let mut snapshot = Snapshot::new();
        // add a single health component
        let orig = Health::new(27);
        snapshot.world.spawn((orig,));

        let mut clone = snapshot.clone_mut();

        let mut matches = Vec::new();
        for (_, &health) in clone.world.query_mut::<&Health>() {
            matches.push(health);
        }
        // expect to see it come back in the clone
        assert_eq!(matches, vec![orig]);
    }
}
