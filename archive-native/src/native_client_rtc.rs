// NOTE: this whole file just doesn't work.
// I independently ran into the same problem as https://github.com/webrtc-rs/webrtc/issues/115
// which seems unsolvable without using trickle ICE which I have no interest in right now.
// Going to create a websocket fallback instead.
use archive_engine::{rtc::*, *};
use archive_server::*;

use anyhow::{bail, Result};

use hyper::{Body, Client, Method, Request};

use log::info;
use webrtc::{
    data_channel::data_channel_init::RTCDataChannelInit,
    peer_connection::sdp::{sdp_type::RTCSdpType, session_description::RTCSessionDescription},
};

pub struct NativeServerHandle {
    pub hostname: String,
}

impl NativeServerHandle {
    async fn rtc_signal(
        hostname: String,
        client_offer: rtc::ClientOffer,
    ) -> Result<rtc::ServerAnswer> {
        let offer_serialized = serde_json::to_string(&client_offer)?;

        info!("offering '{offer_serialized}'");

        let client = Client::new();

        let req = Request::builder()
            .method(Method::POST)
            .uri(&format!("{hostname}/signal"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::from(offer_serialized))?;

        let resp = client.request(req).await?;
        if !resp.status().is_success() {
            bail!("bad signal status: {}", resp.status());
        }
        let bytes = hyper::body::to_bytes(resp.into_body()).await?;
        let server_answer: rtc::ServerAnswer = serde_json::from_slice(&bytes)?;
        Ok(server_answer)
    }
    async fn rtc_connect_raw(hostname: String) -> Result<session::NativeRtcSession> {
        let peer_connection = session::create_peer_connection().await?;

        let offer = peer_connection.create_offer(None).await?;

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection.set_local_description(offer).await?;

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        // Output the answer in base64 so we can paste it in browser
        let sdp = if let Some(local_desc) = peer_connection.local_description().await {
            local_desc.sdp
        } else {
            bail!("failed to generate offer local description");
        };

        let server_answer = Self::rtc_signal(
            hostname,
            rtc::ClientOffer {
                ticket: ArenaTicket { arena_ukey: 0 },
                sdp,
            },
        )
        .await?;

        let mut answer = RTCSessionDescription::default();
        answer.sdp_type = RTCSdpType::Answer;
        answer.sdp = server_answer.sdp;
        peer_connection.set_remote_description(answer).await?;

        peer_connection
            .create_data_channel(
                "udp",
                Some(RTCDataChannelInit {
                    ordered: Some(false),
                    max_packet_life_time: None,
                    max_retransmits: Some(0),
                    protocol: None,
                    negotiated: None,
                    id: None,
                }),
            )
            .await?;
        let session = session::NativeRtcSession::new(peer_connection).await?;
        Ok(session)
    }
}

impl rtc::RtcServerDescriptor for NativeServerHandle {
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
