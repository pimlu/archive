use anyhow::Result;
use archive_engine::*;
use log::*;
use warp::reject::Reject;

use crate::*;

async fn rtc_signal_anyhow(
    client_offer: rtc::ClientOffer,
    arena_map: arena::ArenaMapLock,
) -> Result<impl warp::Reply> {
    let (client_id, arena_lock) =
        arena::process_client_offer(&client_offer, arena_map.clone()).await?;
    debug!("attempting negotiation");

    let peer_connection = session::create_peer_connection().await?;

    let sdp = session::negotiate(peer_connection.clone(), client_offer).await?;

    // warp is going to respond with the SDP credentials, and this task
    // will wait expecting the client to connect using the information
    // the server just provided
    tokio::spawn(async move {
        // log errors but let them pass through, so that the task ends
        match session::NativeRtcSession::new(peer_connection).await {
            Ok(session) => {
                let mut arena = arena_lock.write().await;
                if let Err(e) = arena.process_client_session(client_id, session) {
                    error!("failed to process client session: {e}");
                }
            }
            Err(e) => error!("failed to set up rtc session: {e}"),
        };
    });

    let server_answer = rtc::ServerAnswer { sdp, client_id };
    Ok(warp::reply::json(&server_answer))
}

#[derive(Debug)]
struct AnyhowReject {
    // used by the derive
    #[allow(dead_code)]
    error: anyhow::Error,
}
impl Reject for AnyhowReject {}
fn error_to_reject(error: anyhow::Error) -> warp::Rejection {
    warp::reject::custom(AnyhowReject { error })
}

pub async fn rtc_signal(
    client_offer: rtc::ClientOffer,
    arena_map: arena::ArenaMapLock,
) -> Result<impl warp::Reply, warp::Rejection> {
    rtc_signal_anyhow(client_offer, arena_map)
        .await
        .map_err(error_to_reject)
}
