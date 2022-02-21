use archive_engine::rtc;

use anyhow::{bail, Result};
use log::debug;
use std::cell::Cell;
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
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::{math_rand_alpha, RTCPeerConnection};

pub struct NativeRtcSession {
    pub peer_connection: Arc<RTCPeerConnection>,
    pub attempted_close: Cell<bool>,
    // done_tx: tokio::sync::mpsc::Sender<()>,
    // msg_rx: std::sync::mpsc::Receiver<Vec<u8>>
}

impl NativeRtcSession {
    pub fn new() -> Self {
        todo!();
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
        // since apparently close is async/can fail I don't want to trust it too hard
        // I would rather call this function only once
        if self.attempted_close.get() {
            return;
        }
        self.attempted_close.set(true);
        let peer_connection = self.peer_connection.clone();
        tokio::spawn(async move {
            if let Err(e) = peer_connection.close().await {
                // FIXME what do you even do here when this happens?
                panic!("failed to close: {}", e);
            }
        });
    }

    fn send(&self, msg: Vec<u8>) -> bool {
        todo!()
    }

    fn try_recv(&self) -> Result<Vec<u8>, std::sync::mpsc::TryRecvError> {
        todo!()
    }
}
