use super::*;
use crate::{
    containers::RollingBuf,
    ecs::{Realm, ServerSnapshot, Snapshot},
};

use hecs::*;

pub struct Server {
    realm: Realm,
    clients: [Option<ClientHandle>; ClientId::MAX as _],
}

struct ClientHandle {
    // session == None if they are not connected
    session: Option<ImplRtcClientSession>,
    snapshots: SnapshotBuf<ServerSnapshot>,
    ent: Entity,
}
