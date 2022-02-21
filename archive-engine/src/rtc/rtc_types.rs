use std::sync::mpsc;

use crate::*;

use futures::{AsyncRead, AsyncWrite, Sink, Stream};
use serde::{Deserialize, Serialize};

const SNAPSHOT_CAP: usize = 256;
const SNAPSHOT_VCAP_BITS: u32 = 12;
const SNAPSHOT_VCAP: usize = 2usize.pow(SNAPSHOT_VCAP_BITS);

pub type SnapshotBuf<T> = containers::RollingBuf<T, SNAPSHOT_CAP, SNAPSHOT_VCAP>;

pub type ArenaUkey = u64;
pub type ClientId = u8;

#[derive(Serialize, Deserialize)]
pub struct ClientOffer {
    pub ticket: ArenaUkey,
    pub sdp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAnswer {
    pub client_id: ClientId,
    pub sdp: String,
}

pub enum SessionState {
    Connecting,
    Connected,
    Disconnected,
    Closed,
}
// implemented as either a wrapper around:
// 1. web_sys datachannel (in the browser)
// 2. webrtc crate datachannel (in native)
pub trait RtcSession {
    fn get_state(&self) -> SessionState;
    fn close(&self);
    fn send(&self, msg: Vec<u8>) -> bool;
    fn try_recv(&self) -> Result<Vec<u8>, mpsc::TryRecvError>;
}

pub type BoxedRtcSession = Box<dyn RtcSession>;

pub trait RtcServerDescriptor {
    type Session: RtcSession;
    type Error;
    fn rtc_connect(&self) -> SharedFuture<Result<Self::Session, Self::Error>>;
}
