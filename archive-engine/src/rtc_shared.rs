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

pub trait RtcClientSession {}

pub trait RtcServerHandle {
    type Session: RtcClientSession;
    type Error;
    fn rtc_connect(&self) -> SharedFuture<Result<Self::Session, Self::Error>>;
}
