use std::sync::mpsc;

use crate::*;

use futures::{AsyncRead, AsyncWrite, Sink, Stream};
use serde::{Deserialize, Serialize};

const SNAPSHOT_CAP: usize = 256;
const SNAPSHOT_VCAP_BITS: u32 = 12;
const SNAPSHOT_VCAP: usize = 2usize.pow(SNAPSHOT_VCAP_BITS);

pub type SnapshotBuf<T> = containers::RollingBuf<T, SNAPSHOT_CAP, SNAPSHOT_VCAP>;

// on the server side, we know who is implementing this
pub type ImplRtcClientSession = ();

pub type ClientId = u8;

#[derive(Serialize, Deserialize)]
pub struct ClientOffer {
    pub sdp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAnswer {
    pub sdp: String,
}

// implemented as either a wrapper around:
// 1. web_sys datachannel (in the browser)
// 2. webrtc crate datachannel (in native)
pub trait RtcClientSession {
    fn try_recv(&self) -> Result<Vec<u8>, mpsc::TryRecvError>;
}

pub type BoxRtcClientSession = Box<dyn RtcClientSession>;

pub trait RtcServerDescriptor {
    type Session: RtcClientSession;
    type Error;
    fn rtc_connect(&self) -> SharedFuture<Result<Self::Session, Self::Error>>;
}
