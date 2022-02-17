use crate::containers::CircularBuf;

const NUM_SNAPSHOTS: usize = 32;

struct SnapshotHistory<S> {
    latest_id: usize,
    history: CircularBuf<S, NUM_SNAPSHOTS>,
}
