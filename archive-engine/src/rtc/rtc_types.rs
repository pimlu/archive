use std::sync::mpsc;

use crate::*;

use serde::{Deserialize, Serialize};

const SNAPSHOT_CAP: usize = 256;
const SNAPSHOT_VCAP_BITS: u32 = 12;
const SNAPSHOT_VCAP: usize = 2usize.pow(SNAPSHOT_VCAP_BITS);

pub type SnapshotBuf<T> = containers::RollingBuf<T, SNAPSHOT_CAP, SNAPSHOT_VCAP>;

pub type ArenaUkey = u64;
pub type ClientId = u8;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ArenaTicket {
    pub arena_ukey: ArenaUkey,
}

#[derive(Serialize, Deserialize)]
pub struct ClientOffer {
    pub ticket: ArenaTicket,
    pub sdp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAnswer {
    pub client_id: ClientId,
    pub sdp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Connecting,
    Connected,
    Disconnected,
    Closed,
}
// implemented as a wrapper around:
// 1. web_sys datachannel (in the browser)
// 2. webrtc crate datachannel (in native)
// 3. web_sys websocket (in the browser)
// 4. tungsten websocket (in native)
pub trait RtcSession {
    fn get_state(&self) -> SessionState;
    fn close(&self);
    fn send(&self, msg: Vec<u8>) -> SharedFuture<bool>;
    fn try_recv(&mut self) -> Result<Vec<u8>, mpsc::TryRecvError>;
}

// archive client can't really know about all the runtime types since
// crates depend on it and not the other way around. So I use this on
// the client for now which kind of sucks but w/e
pub type BoxedRtcSession = Box<dyn RtcSession>;

pub trait RtcServerDescriptor {
    type Error;
    fn rtc_connect(&self) -> SharedFuture<Result<BoxedRtcSession, Self::Error>>;
}
