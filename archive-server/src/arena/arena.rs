use crate::*;
use archive_engine::*;

use hecs::*;

// TODO change this based off of features for tests
// the point of this is because dyn traits mess up optimization
// and this trait is an important/frequently used one.
pub type ImplRtcSession = ();

#[derive(Default)]
pub struct Arena {
    pub(super) realm: ecs::Realm,
    pub(super) clients: Vec<Option<ClientHandle>>,
}
impl Arena {
    const NONE: Option<ClientHandle> = None;
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {}
}

pub(super) struct ClientHandle {
    // session == None if they are not connected
    session: Option<ImplRtcSession>,
    snapshots: rtc::SnapshotBuf<ecs::ServerSnapshot>,
    ent: Entity,
}
