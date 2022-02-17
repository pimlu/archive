use crate::{containers::CircularBuf, ecs::Snapshot};

const NUM_SNAPSHOTS: usize = 32;

struct ClientSnapshotHistory {
    latest_id: usize,
    history: CircularBuf<Snapshot, NUM_SNAPSHOTS>,
}

impl ClientSnapshotHistory {}
