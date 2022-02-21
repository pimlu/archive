use std::sync::mpsc;

use log::{error, info};

use archive_engine::*;

#[derive(Default)]
pub struct Client {
    realm: ecs::Realm,
    snapshots: rtc::SnapshotBuf<ecs::Snapshot>,
    session: Option<rtc::BoxedRtcSession>,
}

// messages received "externally" to the client
pub enum ClientMessageFromApp {
    // TODO change this to be a server descriptor?
    // better long term but harder to report errors
    Connected(rtc::BoxedRtcSession),
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
    }
    pub fn frame(&mut self, dt: Num) {
        if self.session.is_none() {
            return;
        }
        loop {
            match self.session.as_ref().unwrap().try_recv() {
                Ok(msg) => {
                    info!("received: {:?}", msg);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    error!("session disconnected");
                    self.session = None;
                    return;
                }
            }
        }
    }
}
