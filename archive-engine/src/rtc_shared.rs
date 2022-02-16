use futures::{Sink, Stream};
use serde::{Deserialize, Serialize};

use crate::SharedFuture;

#[derive(Serialize, Deserialize)]
pub struct ClientOffer {
    pub sdp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAnswer {
    pub sdp: String,
}

pub trait RawChannel: Stream<Item = Vec<u8>> + Sink<Vec<u8>> {}

pub trait RtcClientSession {
    type Channel: RawChannel;
    fn channels(&mut self) -> &mut [Self::Channel];
    fn reliability(&self) -> &[bool];
}

pub trait RtcServerHandle {
    type Session: RtcClientSession;
    type Error;
    fn rtc_connect(&self) -> SharedFuture<Result<Self::Session, Self::Error>>;
}
