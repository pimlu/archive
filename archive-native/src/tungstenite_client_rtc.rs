use anyhow::{Context, Result};
use archive_engine::{
    rtc::{self, ArenaTicket, RtcSession},
    *,
};
use archive_server::session;
use futures::SinkExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub struct TungsteniteServerHandle {
    pub hostname: String,
}

impl TungsteniteServerHandle {
    async fn rtc_connect_raw(hostname: String) -> Result<session::MpscRtcSession> {
        let (mut ws_stream, _) = connect_async(hostname).await.context("Failed to connect")?;
        let ticket = ArenaTicket { arena_ukey: 0 };
        let ticket_bin = bincode::serialize(&ticket)?;
        ws_stream.send(Message::Binary(ticket_bin)).await?;
        session::MpscRtcSession::new_from_tungstenite(ws_stream).await
    }
}

impl rtc::RtcServerDescriptor for TungsteniteServerHandle {
    type Error = anyhow::Error;

    fn rtc_connect(&self) -> SharedFuture<Result<rtc::BoxedRtcSession, Self::Error>> {
        let hostname = self.hostname.clone();

        Box::pin(async move {
            let session = Self::rtc_connect_raw(hostname).await?;
            let boxed: Box<dyn RtcSession> = Box::new(session);
            Ok(boxed)
        })
    }
}
