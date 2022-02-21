use std::{collections::BTreeMap, sync::Arc};

use super::*;
use archive_engine::*;

use anyhow::{bail, Result};
use log::*;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct ArenaMap {
    arena_map: BTreeMap<rtc::ArenaUkey, ArenaLock>,
}
pub type ArenaMapLock = Arc<RwLock<ArenaMap>>;

pub type ArenaLock = Arc<RwLock<Arena>>;
impl ArenaMap {
    pub fn get_or_insert_default(&mut self, arena_ukey: rtc::ArenaUkey) -> ArenaLock {
        if let Some(arena) = self.arena_map.get(&arena_ukey) {
            arena.clone()
        } else {
            let arena = ArenaLock::default();
            self.arena_map.insert(arena_ukey, arena.clone());
            self.start_poll_task(arena.clone());
            arena
        }
    }
    fn start_poll_task(&self, arena_strong: ArenaLock) {
        let arena_weak = Arc::downgrade(&arena_strong);
        std::mem::drop(arena_strong);

        tokio::spawn(async move {
            loop {
                if let Some(arena_strong) = arena_weak.upgrade() {
                    let mut arena = arena_strong.write().await;
                    arena.tick();

                    // use tokio/async when sending because https://github.com/quinn-rs/quinn/issues/867
                    // FIXME this introduces error into the tickrate
                    // calculate a duration for sleep instead
                    arena.tick_async().await;
                } else {
                    break;
                }

                // FIXME is there a better way to make tickrate guarantees?
                // sleep has 1ms precision
                let timeout = tokio::time::sleep(ecs::TICK_DURATION);
                tokio::pin!(timeout);
                let _ = timeout.as_mut().await;
            }
            // TODO better instrumentation
            info!("dropped arena");
        });
    }
}

pub async fn process_client_offer(
    client_offer: &rtc::ClientOffer,
    arena_map: ArenaMapLock,
) -> Result<(rtc::ClientId, ArenaLock)> {
    // TODO process their ticket using diesel
    let arena_ukey: rtc::ArenaUkey = client_offer.ticket;

    // first lock arena_map briefly to get access to the corresponding arena
    let arena_lock = {
        let mut arena_map = arena_map.write().await;
        arena_map.get_or_insert_default(arena_ukey)
    };

    let client_id = {
        // then lock the arena itself to add the client
        let mut arena = arena_lock.write().await;

        // TODO actual tickets/client_ids
        if arena.clients.len() == rtc::ClientId::MAX as usize {
            bail!("max clients reached");
        }
        let client_id = arena.clients.len() as rtc::ClientId;
        arena.alloc_client(client_id);
        client_id
    };

    Ok((client_id as _, arena_lock))
}
