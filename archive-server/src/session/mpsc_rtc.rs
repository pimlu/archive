use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{Context, Result};
use archive_engine::rtc;
use futures::{Future, SinkExt, StreamExt};
use log::{error, info, log, Level};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::{
    net::TcpStream,
    select,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::session::MAX_MSG_BUF;

use super::map_try_recv_to_std;

pub struct MpscRtcSession {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    is_open: Arc<AtomicBool>,
    done_tx: Sender<()>,
}

impl rtc::RtcSession for MpscRtcSession {
    fn get_state(&self) -> rtc::SessionState {
        if self.is_open.load(Ordering::Relaxed) {
            rtc::SessionState::Connected
        } else {
            rtc::SessionState::Closed
        }
    }

    fn close(&self) {
        let _ = self.done_tx.try_send(());
    }

    fn send(&self, msg: Vec<u8>) -> archive_engine::SharedFuture<bool> {
        Box::pin(self.send_impl(msg))
    }

    fn try_recv(&mut self) -> Result<Vec<u8>, std::sync::mpsc::TryRecvError> {
        self.rx.try_recv().map_err(map_try_recv_to_std)
    }
}
impl MpscRtcSession {
    pub fn send_impl(&self, msg: Vec<u8>) -> impl Future<Output = bool> {
        let success = match self.tx.try_send(msg) {
            Ok(()) => true,
            Err(err) => {
                error!("failed to send over ws: {err}");
                false
            }
        };
        async move { success }
    }

    // S could be either TcpStream or MaybeTlsStream, idc which, I just want to forward
    // it to mpsc channels and stop considering it
    pub async fn new_from_tungstenite<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        ws_stream: WebSocketStream<S>,
    ) -> Result<MpscRtcSession> {
        let (mut write, mut read) = ws_stream.split();

        let (write_tx, mut write_rx) = mpsc::channel::<Vec<u8>>(1);
        let (read_tx, read_rx) = mpsc::channel::<Vec<u8>>(MAX_MSG_BUF);

        let (done_tx, mut done_rx) = mpsc::channel::<()>(1);

        let is_open = Arc::new(AtomicBool::new(true));
        let is_open_copy = is_open.clone();

        let session = MpscRtcSession {
            tx: write_tx,
            rx: read_rx,
            done_tx,
            is_open,
        };

        let msg_loop = {
            let write_loop = async move {
                while let Some(msg) = write_rx.recv().await {
                    write.send(Message::Binary(msg)).await?;
                }
                Ok(()) as Result<()>
            };
            let read_loop = async move {
                while let Some(msg) = read.next().await {
                    let msg = msg?.into_data();
                    read_tx.send(msg).await?;
                }
                Ok(()) as Result<()>
            };

            async move {
                let done_interrupt = done_rx.recv();
                let result;
                select! {
                    res = write_loop => {
                        result = res;
                    }
                    res = read_loop => {
                        result = res;
                    }
                    option = done_interrupt => {
                        result = option.context("done_tx dropped");
                    }
                }
                let level = if result.is_ok() {
                    Level::Info
                } else {
                    Level::Error
                };
                log!(level, "ws connection closed: {:?}", result);
                is_open_copy.store(false, Ordering::Relaxed);
            }
        };

        tokio::spawn(msg_loop);

        Ok(session)
    }
}
