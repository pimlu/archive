use std::collections::BTreeMap;

use crate::*;
use anyhow::{Context, Result};
use archive_engine::{rtc::RtcSession, *};

use log::error;
use webrtc::peer_connection::math_rand_alpha;

// TODO change this based off of features for tests
// the point of this is because dyn traits mess up optimization
// and this trait is an important/frequently used one.
pub type ImplRtcSession = session::NativeRtcSession;

#[derive(Default)]
pub struct Arena {
    pub(super) realm: ecs::Realm,
    pub(super) clients: BTreeMap<rtc::ClientId, ClientHandle>,
}
impl Arena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {}
    pub async fn tick_async(&mut self) {
        for (client_id, handle) in self.clients.iter_mut() {
            if handle.session.is_none() {
                continue;
            }
            let session = handle.session.as_mut().unwrap();

            let session_state = session.get_state();
            // TODO add a small grace period
            if session_state == rtc::SessionState::Disconnected {
                // just kick them for now
                session.close();
            }
            if session_state != rtc::SessionState::Connected {
                // TODO add some grace period/kick them here
                continue;
            }
            let message = math_rand_alpha(15);
            let send_ok = session.send_impl(Vec::from(message.as_bytes())).await;
            if !send_ok {
                error!("failed to send to client #{client_id}");
            }
        }
    }

    pub fn alloc_client(&mut self, client_id: rtc::ClientId) {
        self.clients.insert(client_id, ClientHandle::default());
    }
    pub fn process_client_session(
        &mut self,
        client_id: rtc::ClientId,
        session: ImplRtcSession,
    ) -> Result<()> {
        let handle = self
            .clients
            .get_mut(&client_id)
            .context("unreachable: missing client")?;
        // TODO check if a session already exists?
        handle.session = Some(session);
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct ClientHandle {
    // session == None if they are not connected
    session: Option<ImplRtcSession>,
    snapshots: rtc::SnapshotBuf<ecs::ServerSnapshot>,
}
