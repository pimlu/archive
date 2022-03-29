use std::collections::BTreeMap;

use crate::*;
use anyhow::{Context, Result};
use archive_engine::{
    rtc::{ClientId, RtcSession},
    *,
};

use log::{error, info};
use webrtc::peer_connection::math_rand_alpha;

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
        let mut to_drop = Vec::<ClientId>::new();
        for (client_id, handle) in self.clients.iter_mut() {
            if handle.session.is_none() {
                continue;
            }
            let session = handle.session.as_mut().unwrap();

            match session.get_state() {
                // just kick them for now
                rtc::SessionState::Disconnected => session.close(),
                rtc::SessionState::Closed => {
                    to_drop.push(*client_id);
                    continue;
                }
                // TODO add some grace period/kick them here
                rtc::SessionState::Connecting => continue,
                rtc::SessionState::Connected => (),
            };
            let message = math_rand_alpha(15);
            let send_ok = session.send_impl(Vec::from(message.as_bytes())).await;
            if !send_ok {
                error!("failed to send to client #{client_id}");
            }
        }
        // drop session handles that have been closed
        for client_id in to_drop {
            info!("dropping disconnected client #{client_id}");
            self.clients.remove(&client_id);
        }
    }

    pub fn alloc_client(&mut self, client_id: rtc::ClientId) {
        self.clients.insert(client_id, ClientHandle::default());
    }
    pub fn process_client_session(
        &mut self,
        client_id: rtc::ClientId,
        session: session::EnumRtcSession,
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
    session: Option<session::EnumRtcSession>,
    snapshots: rtc::SnapshotBuf<ecs::ServerSnapshot>,
}
