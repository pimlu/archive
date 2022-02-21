use anyhow::Result;
use archive_engine::*;
use log::debug;
use warp::reject::Reject;

use crate::*;

async fn rtc_signal_anyhow(
    client_offer: rtc::ClientOffer,
    arena_map: arena::ArenaMapLock,
) -> Result<impl warp::Reply> {
    let client_id = arena::process_client_offer(&client_offer, arena_map.clone()).await?;
    debug!("attempting negotiation");
    let session::Negotiation {
        sdp,
        session,
        mut done_rx,
    } = session::negotiate(client_offer).await?;

    let server_answer = rtc::ServerAnswer { sdp, client_id };

    tokio::spawn(async move {
        done_rx.recv().await;
        debug!("received done signal");
        session.peer_connection.close().await.unwrap();
    });

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
