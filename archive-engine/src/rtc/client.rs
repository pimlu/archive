use std::sync::mpsc;

use log::info;

use super::*;
use crate::*;

#[derive(Default)]
pub struct Client {
    realm: ecs::Realm,
    snapshots: SnapshotBuf<ecs::Snapshot>,
    session: Option<BoxRtcClientSession>,
}

// messages received "externally" to the client
pub enum ClientMessageFromApp {
    Connected(BoxRtcClientSession),
}

pub type ClientReceiver = mpsc::Receiver<ClientMessageFromApp>;

impl Client {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn recv_from_app(&mut self, msg: ClientMessageFromApp) {
        use ClientMessageFromApp::*;
        match msg {
            Connected(session) => self.session = Some(session),
        }
        // for debug
        info!("received a thing");
    }
}
