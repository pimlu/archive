use super::*;
use crate::*;

use std::collections::{BTreeMap, BinaryHeap};

use hecs::*;

macro_rules! make_delta_diff {
    ($($kinds:tt),*) => {
        derive_delta! {
            enum DeltaDiff {
                $($kinds($kinds)),*
            }
        }
    }
}
macro_rules! make_delta_replace {
    ($($kinds:tt),*) => {
        derive_delta! {
            enum DeltaReplace {
                $($kinds($kinds)),*
            }
        }
    }
}

macro_rules! make_delta_remove {
    ($($kinds:tt),*) => {
        derive_delta! {
            enum DeltaRemove {
                $($kinds),*
            }
        }
    }
}

make_delta_diff!(Position, Rotation, Velocity, Health);
make_delta_replace!(Camera, Player, Bullet);
make_delta_remove!(Position, Rotation, Velocity, Camera, Player, Bullet, Health);

macro_rules! match_delta_helper {
    ($enum:ident, $diff:ident, |$c:ident| $expr:expr, $($kinds:tt),*) => {
        match $diff {
            $($enum::$kinds($c) => {
                type _Component = $kinds;
                $expr
            }),*
        }
    };
}
macro_rules! match_delta_diff {
    ($diff:ident, |$c:ident| $expr:expr) => {
        match_delta_helper!(
            DeltaDiff,
            $diff,
            |$c| $expr,
            Position,
            Rotation,
            Velocity,
            Health
        )
    };
}
macro_rules! match_delta_replace {
    ($diff:ident, |$c:ident| $expr:expr) => {
        match_delta_helper!(DeltaReplace, $diff, |$c| $expr, Camera, Player, Bullet)
    };
}

macro_rules! match_delta_remove_helper {
    ($enum:ident, $diff:ident, || $expr:expr, $($kinds:tt),*) => {
        match $diff {
            $($enum::$kinds => {
                type Struct = $kinds;
                $expr
            }),*
        }
    };
}

macro_rules! match_delta_remove {
    ($diff:ident, || $expr:expr) => {
        match_delta_remove_helper!(
            DeltaRemove,
            $diff,
            || $expr,
            Position,
            Rotation,
            Velocity,
            Camera,
            Player,
            Bullet,
            Health
        )
    };
}

// represents a single modification to a repl token.
derive_delta! {
    enum DeltaComponentPatch {
        DiffComponent(DeltaDiff),
        ReplaceComponent(DeltaReplace),
        RemoveComponent(DeltaRemove),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum DeltaEntityPatch {
    SpawnEntity(Vec<DeltaComponentPatch>),
    UpdateEntity(Vec<DeltaComponentPatch>),
    DespawnEntity,
}
// represents a set of modifications to a single repl token.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaAction {
    repl_key: ReplKey,
    ent_patch: DeltaEntityPatch,
}
// only exists as a wrapper for the priority heap
struct PrioritizedAction {
    action: DeltaAction,
    priority_accum: Priority,
}
impl PartialEq for PrioritizedAction {
    fn eq(&self, other: &Self) -> bool {
        self.priority_accum == other.priority_accum
    }
}
impl Eq for PrioritizedAction {}
impl Ord for PrioritizedAction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority_accum.cmp(&other.priority_accum)
    }
}
impl PartialOrd for PrioritizedAction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy)]
struct SingleDiff {
    old_token: Option<ReplToken>,
    new_token: Option<ReplToken>,
}
impl SingleDiff {
    fn newest_token(&self) -> ReplToken {
        self.new_token.or(self.old_token).unwrap()
    }
}

pub struct Delta {
    actions: Vec<DeltaAction>,
}

pub struct ServerDelta {
    inner: Delta,
    server_meta: ServerSnapshotMeta,
}
impl ServerDelta {
    // we want to check if entities were created/deleted, this joins
    // the old/new entity maps on their ReplKeys which lets us make
    // SingleDiffs for comparison purposes
    fn merge_diff_map(
        base: &mut ServerSnapshot,
        realm: &mut Realm,
    ) -> BTreeMap<ReplKey, SingleDiff> {
        // TODO include the map values in here as an optimization
        let mut token_map = BTreeMap::new();

        for &token in base.meta.token_map.values() {
            // I don't expect double tokens for the same key, but if it
            // somehow did happen, we intentionally overwrite here, I guess
            token_map.insert(
                token.key(),
                SingleDiff {
                    old_token: Some(token),
                    new_token: None,
                },
            );
        }
        for &token in realm.ent_map.left_values() {
            let key = token.key();
            if token_map.contains_key(&key) {
                token_map.get_mut(&key).unwrap().new_token = Some(token);
            } else {
                token_map.insert(
                    key,
                    SingleDiff {
                        old_token: None,
                        new_token: Some(token),
                    },
                );
            }
        }
        token_map
    }

    // "loops" through each component and actually makes the delta.
    fn patches_for_components(
        old_world: &mut World,
        old_ent: Option<Entity>,
        new_world: &mut World,
        new_ent: Entity,
    ) -> Vec<DeltaComponentPatch> {
        // TODO use velocity for better position prediction
        let mut patches = Vec::new();

        assert!(old_ent.is_none() || old_world.contains(old_ent.unwrap()));
        assert!(new_world.contains(new_ent));

        macro_rules! query_both {
            ($type: ty) => {
                (
                    old_ent
                        .and_then(|ent| old_world.query_one_mut::<&$type>(ent).ok())
                        .copied(),
                    new_world.query_one_mut::<&$type>(new_ent).ok().copied(),
                )
            };
        }

        macro_rules! standard_diff {
            ($($kinds:tt),*) => {
                $(
                    let (old, new) = query_both!($kinds);

                    if old != new {
                        if let Some(new) = new {
                            let old = old.unwrap_or_default();
                            patches.push(DeltaComponentPatch::DiffComponent(DeltaDiff::$kinds(new - old)));
                        } else {
                            // old must not be None
                            patches.push(DeltaComponentPatch::RemoveComponent(DeltaRemove::$kinds));
                        }
                    }
                )*
            };
        }
        // TODO replace this with some kind of Diff trait with associated items?
        standard_diff!(Position, Rotation, Velocity, Health);

        macro_rules! standard_replace {
            ($($kinds:tt),*) => {
                $(
                    let (old, new) = query_both!($kinds);

                    if old != new {
                        if let Some(new) = new {
                            patches.push(DeltaComponentPatch::ReplaceComponent(DeltaReplace::$kinds(new)));
                        } else {
                            // old must not be None
                            patches.push(DeltaComponentPatch::RemoveComponent(DeltaRemove::$kinds));
                        }
                    }
                )*
            };
        }
        standard_replace!(Camera, Player, Bullet);

        patches
    }

    // constructs a DeltaEntityPatch, if there is a difference.
    fn patch_for_entity(
        base: &mut ServerSnapshot,
        realm: &mut Realm,
        single_diff: SingleDiff,
    ) -> Option<DeltaEntityPatch> {
        let SingleDiff {
            old_token,
            new_token,
        } = single_diff;
        assert_ne!([old_token, new_token], [None, None], "can't be both None");

        if new_token.is_none() {
            return Some(DeltaEntityPatch::DespawnEntity);
        }

        let old_ent = old_token.map(|tok| base.entity_for_token(&tok)).flatten();
        let new_ent = realm.entity_for_token(&new_token.unwrap());

        let update = ServerDelta::patches_for_components(
            &mut base.inner.world,
            old_ent,
            &mut realm.world,
            new_ent,
        );
        if update.is_empty() {
            None
        } else if old_token == new_token {
            Some(DeltaEntityPatch::UpdateEntity(update))
        } else {
            Some(DeltaEntityPatch::SpawnEntity(update))
        }
    }

    fn prioritized_total_diff(
        base: &mut ServerSnapshot,
        realm: &mut Realm,
    ) -> (Vec<PrioritizedAction>, ReplTokenMap) {
        let mut result = Vec::new();

        // construct a map which lets us look simultaneously at the "old" and "new"
        // values for a ReplKey "slot".
        let merged_token_map = ServerDelta::merge_diff_map(base, realm);

        for (&repl_key, &single_diff) in &merged_token_map {
            let SingleDiff {
                old_token,
                new_token,
            } = single_diff;

            let old_priority_accum = old_token
                .and_then(|tok| base.priority_accum_for_token(&tok))
                .unwrap_or_default();

            let new_entity = new_token.map(|tok| realm.entity_for_token(&tok));
            let new_priority_inc = new_entity.map_or(realm.calc_deleted_priority_accum(), |ent| {
                realm.calc_priority_inc(ent)
            });

            let priority_accum = old_priority_accum.saturating_add(new_priority_inc);

            // only if there is something to do, generate an action
            if let Some(ent_patch) = ServerDelta::patch_for_entity(base, realm, single_diff) {
                result.push(PrioritizedAction {
                    priority_accum,
                    action: DeltaAction {
                        repl_key,
                        ent_patch,
                    },
                });
            }
        }

        let token_map: ReplTokenMap = merged_token_map
            .into_iter()
            .map(|(repl_key, single_diff)| (repl_key, single_diff.newest_token()))
            .collect();
        (result, token_map)
    }

    pub(super) fn diff(base: &mut ServerSnapshot, realm: &mut Realm) -> Self {
        let (total_diff, token_map) = ServerDelta::prioritized_total_diff(base, realm).into();
        let mut queue: BinaryHeap<_> = total_diff.into();

        let mut actions = Vec::new();

        let mut priority_map = BTreeMap::new();
        while let Some(prioritized) = queue.pop() {
            actions.push(prioritized.action);

            // FIXME arbitrary limit to test priority
            if actions.len() >= 10 {
                break;
            }
        }

        for batch in queue.drain() {
            let PrioritizedAction {
                action,
                priority_accum,
                ..
            } = batch;
            priority_map.insert(action.repl_key, priority_accum);
        }

        let inner = Delta { actions };
        // NOTE: this token_map has too many tokens, (it has all the old tokens from merge_diff_map)
        // so the server_augment() function will strip unused tokens later
        let server_meta = ServerSnapshotMeta {
            token_map,
            priority_map,
        };

        ServerDelta { inner, server_meta }
    }

    // &mut is only used for an exclusive reference to the World for performance
    pub(super) fn apply_server(&self, to: &mut ServerSnapshot) -> ServerSnapshot {
        let new_snapshot = self.inner.apply(&mut to.inner);

        new_snapshot.server_augment(self.server_meta.clone())
    }
}

impl Delta {
    // &mut is only used for an exclusive reference to the World for performance
    pub(super) fn apply(&self, onto: &mut Snapshot) -> Snapshot {
        let mut result = onto.clone_mut();
        for action in &self.actions {
            let DeltaAction {
                repl_key,
                ent_patch,
            } = action;
            let ent = result.ent_map.get(&repl_key).copied();
            match ent_patch {
                DeltaEntityPatch::SpawnEntity(patches) => {
                    if let Some(ent) = ent {
                        result.world.despawn(ent).unwrap();
                    }
                    let mut builder = Self::build_spawn(patches);
                    let ent = result.world.spawn(builder.build());
                    result.ent_map.insert(*repl_key, ent);
                }
                DeltaEntityPatch::UpdateEntity(patches) => {
                    Self::apply_component_patches(&mut result, ent.unwrap(), patches);
                }
                DeltaEntityPatch::DespawnEntity => {
                    result.world.despawn(ent.unwrap()).unwrap();
                    result.ent_map.remove(repl_key).unwrap();
                }
            }
        }
        result
    }

    fn build_spawn(patches: &Vec<DeltaComponentPatch>) -> EntityBuilder {
        let mut builder = EntityBuilder::new();

        for &patch in patches {
            use DeltaComponentPatch::*;
            match patch {
                DiffComponent(diff) => match_delta_diff!(diff, |c| builder.add(c)),
                ReplaceComponent(replace) => match_delta_replace!(replace, |c| builder.add(c)),
                RemoveComponent(_) => unreachable!(),
            };
        }

        builder
    }
    fn apply_component_patches(
        target: &mut Snapshot,
        ent: Entity,
        patches: &Vec<DeltaComponentPatch>,
    ) {
        for &patch in patches {
            use DeltaComponentPatch::*;

            match patch {
                DiffComponent(diff) => match_delta_diff!(diff, |c| {
                    let dest = target.get_mut::<&mut _Component>(ent);
                    if let Some(dest) = dest {
                        *dest += c;
                    } else {
                        // add component if it isn't there already
                        target.world.insert_one(ent, c).unwrap();
                    }
                }),
                ReplaceComponent(replace) => match_delta_replace!(replace, |c| {
                    let dest = target.get_mut::<&mut _Component>(ent);
                    if let Some(dest) = dest {
                        *dest = c;
                    } else {
                        // add component if it isn't there already
                        target.world.insert_one(ent, c).unwrap();
                    }
                }),
                RemoveComponent(remove) => match_delta_remove!(remove, || {
                    target.world.remove_one::<Struct>(ent).unwrap();
                }),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R_PLAYER: Replicated = Replicated {
        blueprint: Some(Blueprint::Player),
    };

    const V_A: V2 = mk_v2!(1.2, -1.6);
    const V_B: V2 = mk_v2!(0.1, 0);

    #[test]
    fn test_empty_diff() {
        let mut realm = Realm::new();
        let mut base = ServerSnapshot::new();
        let diff = ServerDelta::diff(&mut base, &mut realm);
        assert!(diff.inner.actions.is_empty());
    }

    #[test]
    fn test_spawn_despawn() {
        let mut realm = Realm::new();
        let mut base = ServerSnapshot::new();

        let pos_a = Position {
            xy: V_A,
            zed: mk_zed(2),
        };

        let ent = realm.spawn((pos_a, R_PLAYER));

        let mut midpoint = {
            let diff = ServerDelta::diff(&mut base, &mut realm);

            assert_eq!(diff.inner.actions.len(), 1, "has a single action");

            let ent_patch = &diff.inner.actions[0].ent_patch;
            assert_eq!(
                *ent_patch,
                DeltaEntityPatch::SpawnEntity(vec![DeltaComponentPatch::DiffComponent(
                    DeltaDiff::Position(pos_a)
                )]),
                "spawns in a position entity"
            );

            diff.apply_server(&mut base)
        };

        {
            let diff = ServerDelta::diff(&mut midpoint, &mut realm);
            assert_eq!(diff.inner.actions.len(), 0, "snapshot is up to date");
        }

        realm.despawn(ent);

        {
            let diff = ServerDelta::diff(&mut base, &mut realm);
            assert_eq!(
                diff.inner.actions.len(),
                0,
                "diff to base is blank for empty realm"
            );
        }

        {
            let diff = ServerDelta::diff(&mut midpoint, &mut realm);

            assert_eq!(diff.inner.actions.len(), 1, "has a single action");
            assert_eq!(
                diff.inner.actions[0].ent_patch,
                DeltaEntityPatch::DespawnEntity
            );
        }
    }
}
