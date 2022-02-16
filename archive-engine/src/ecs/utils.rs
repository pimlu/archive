use hecs::*;

pub fn world_get_mut<Q: Query>(world: &mut World, ent: Entity) -> Option<QueryItem<Q>> {
    match world.query_one_mut::<Q>(ent) {
        Ok(res) => Some(res),
        Err(QueryOneError::Unsatisfied) => None,
        Err(QueryOneError::NoSuchEntity) => panic!("missing ent"),
    }
}
