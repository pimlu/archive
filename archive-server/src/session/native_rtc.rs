use archive_engine::{rtc::RtcSession, *};

use anyhow::{bail, Context, Result};
use futures::Future;
use log::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;

use bytes::Bytes;

use super::map_try_recv_to_std;

// FIXME add logic to boot old clients when a double handshake happens
// either that, or rate limit it in warp instead
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);

// Buffer up to a dozen messages from the client, using a bounded channel
// because we don't trust them. Why a dozen? 12 is a cool number
pub const MAX_MSG_BUF: usize = 12;

pub async fn create_peer_connection() -> Result<Arc<RTCPeerConnection>> {
    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    debug!("creating peer connection");
    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    Ok(peer_connection)
}
pub struct NativeRtcSession {
    pub peer_connection: Arc<RTCPeerConnection>,
    pub data_channel: Arc<RTCDataChannel>,
    pub attempted_close: AtomicBool,
    done_tx: tokio::sync::mpsc::Sender<()>,
    msg_rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}

impl NativeRtcSession {
    // waits for data channel opening on an RTCPeerConnection and then returns
    // a NativeRtcSession wrapping the whole thing. The peer connection should
    // already have been set up with SDP and such.
    // Agnostic to server/client side. Has a timeout.
    pub async fn new(peer_connection: Arc<RTCPeerConnection>) -> Result<Self> {
        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

        let peer_connection_done = peer_connection.clone();
        tokio::spawn(async move {
            done_rx.recv().await;
            debug!("received done signal");
            // FIXME what do you even do when this fails?
            peer_connection_done.close().await.unwrap();
        });

        let timeout = tokio::time::sleep(HANDSHAKE_TIMEOUT);
        tokio::pin!(timeout);

        let get_session = Self::finish_new(peer_connection, done_tx.clone());

        tokio::select! {
            _ = timeout.as_mut() => {
                let _ = done_tx.try_send(());
                bail!("handshake timed out");
            }
            session = get_session => session
        }
    }

    async fn finish_new(
        peer_connection: Arc<RTCPeerConnection>,
        done_tx: tokio::sync::mpsc::Sender<()>,
    ) -> Result<Self> {
        let done_tx_fail = done_tx.clone();
        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    println!("Peer Connection has gone to failed exiting");
                    let _ = done_tx_fail.try_send(());
                }

                Box::pin(async {})
            }))
            .await;

        // dc channel is used to "trampoline" the datachannel out of the event handler.
        let (dc_tx, mut dc_rx) = tokio::sync::mpsc::channel::<Arc<RTCDataChannel>>(1);
        let (msg_tx, msg_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(MAX_MSG_BUF);
        let dc_set = Arc::new(AtomicBool::new(false));

        let done_tx_dc = done_tx.clone();
        // Register data channel creation handling
        peer_connection
            .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
                let d_label = d.label().to_owned();
                let d_id = d.id();
                println!("New DataChannel {} {}", d_label, d_id);

                // only allow at most one onDataChannel event using atomic booleans
                let dc_already_set = dc_set.swap(true, Ordering::Relaxed);
                if dc_already_set || d.ordered() || d.max_retransmits() != 0 {
                    println!("dc doesn't pass sanity checks, closing");
                    let _ = done_tx_dc.try_send(());
                }

                // borrow checker makes us clone these for the async
                let dc_tx = dc_tx.clone();
                let done_tx_dc = done_tx_dc.clone();
                let msg_tx = msg_tx.clone();

                // Register channel opening handling
                Box::pin(async move {
                    let d2 = Arc::clone(&d);
                    let d_label2 = d_label.clone();
                    let d_id2 = d_id;
                    d.on_open(Box::new(move || {
                        println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", d_label2, d_id2);

                        if let Err(_) = dc_tx.try_send(d2) {
                            println!("dc_tx send fail");
                            let _ = done_tx_dc.try_send(());
                        }

                        Box::pin(async {})
                    })).await;

                    // Register text message handling
                    d.on_message(Box::new(move |msg: DataChannelMessage| {
                        println!("Message from DataChannel '{}'", d_label);
                        let msg = msg.data.to_vec();

                        if let Err(_) = msg_tx.try_send(msg) {
                            println!("msg send fail");
                            // FIXME tally up overflow packets here for client misbehavior
                        }

                        Box::pin(async {})
                    })).await;

                })
            }))
            .await;

        let data_channel = dc_rx.recv().await.context("dc hangup")?;

        Ok(NativeRtcSession {
            peer_connection,
            data_channel,
            attempted_close: false.into(),
            done_tx,
            msg_rx,
        })
    }

    pub fn send_impl(&self, msg: Vec<u8>) -> impl Future<Output = bool> {
        let data_channel = self.data_channel.clone();
        async move {
            match data_channel.send(&Bytes::from(msg)).await {
                Ok(_) => true,
                Err(e) => {
                    error!("send error {e}");
                    false
                }
            }
        }
    }
}

impl rtc::RtcSession for NativeRtcSession {
    fn get_state(&self) -> rtc::SessionState {
        match self.peer_connection.connection_state() {
            RTCPeerConnectionState::Unspecified => rtc::SessionState::Disconnected,
            RTCPeerConnectionState::New => rtc::SessionState::Connecting,
            RTCPeerConnectionState::Connecting => rtc::SessionState::Connecting,
            RTCPeerConnectionState::Connected => rtc::SessionState::Connected,
            RTCPeerConnectionState::Disconnected => rtc::SessionState::Connected,
            RTCPeerConnectionState::Failed => rtc::SessionState::Disconnected,
            RTCPeerConnectionState::Closed => rtc::SessionState::Closed,
        }
    }

    fn close(&self) {
        // I would rather call this function only once
        // or at least know when I call it twice
        if self.attempted_close.swap(true, Ordering::Relaxed) {
            info!("already closed");
        } else {
            let _ = self.done_tx.try_send(());
        }
    }

    fn send(&self, msg: Vec<u8>) -> SharedFuture<bool> {
        Box::pin(self.send_impl(msg))
    }

    fn try_recv(&mut self) -> Result<Vec<u8>, std::sync::mpsc::TryRecvError> {
        self.msg_rx.try_recv().map_err(map_try_recv_to_std)
    }
}
