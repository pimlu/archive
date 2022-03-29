use archive_engine::*;
use derive_more::From;

use super::*;

// the point of this is because dyn traits mess up optimization
// and this trait is an important/frequently used one.
// enum_dispatch only works within the same crate unfortunately.
#[derive(From)]
pub enum EnumRtcSession {
    Native(NativeRtcSession),
    Mpsc(MpscRtcSession),
}
use EnumRtcSession::*;
impl rtc::RtcSession for EnumRtcSession {
    fn get_state(&self) -> rtc::SessionState {
        match self {
            Native(s) => s.get_state(),
            Mpsc(s) => s.get_state(),
        }
    }

    fn close(&self) {
        match self {
            Native(s) => s.close(),
            Mpsc(s) => s.close(),
        }
    }

    fn send(&self, msg: Vec<u8>) -> SharedFuture<bool> {
        match self {
            Native(s) => s.send(msg),
            Mpsc(s) => s.send(msg),
        }
    }

    fn try_recv(&mut self) -> Result<Vec<u8>, std::sync::mpsc::TryRecvError> {
        match self {
            Native(s) => s.try_recv(),
            Mpsc(s) => s.try_recv(),
        }
    }
}

impl EnumRtcSession {
    pub async fn send_impl(&self, msg: Vec<u8>) -> bool {
        match self {
            Native(s) => s.send_impl(msg).await,
            Mpsc(s) => s.send_impl(msg).await,
        }
    }
}
