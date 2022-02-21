use super::*;

use anyhow::{bail, Result};
use archive_engine::*;
use log::debug;
use std::sync::Arc;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

pub struct Negotiation {
    pub sdp: String,
    pub session: NativeRtcSession,
    pub done_rx: tokio::sync::mpsc::Receiver<()>,
}

// returns SDP string for warp to respond with.
// also creates a tokio task that awaits a connection
pub async fn negotiate<'a>(
    peer_connection: Arc<RTCPeerConnection>,
    client_offer: rtc::ClientOffer,
) -> Result<String> {
    debug!("parsing client SDP");
    // Wait for the offer to be pasted
    let mut offer = RTCSessionDescription::default();
    offer.sdp_type = RTCSdpType::Offer;
    offer.sdp = client_offer.sdp;
    //serde_json::from_str::<RTCSessionDescription>(&client_offer.sdp)?;

    // Set the remote SessionDescription
    peer_connection.set_remote_description(offer).await?;

    // Create an answer
    let answer = peer_connection.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = peer_connection.local_description().await {
        debug!("succeeded negotiation");
        Ok(local_desc.sdp)
    } else {
        bail!("failed to generate answer local description");
    }
}
