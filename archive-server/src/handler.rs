use anyhow::Result;
use archive_engine::rtc::ClientOffer;
use log::debug;
use warp::reject::Reject;

use crate::server_rtc::{negotiate, Negotiation};

async fn signal_anyhow(client_offer: ClientOffer) -> Result<impl warp::Reply> {
    debug!("attempting negotiation");
    let Negotiation {
        server_answer,
        peer_connection,
        mut done_rx,
    } = negotiate(client_offer).await?;

    tokio::spawn(async move {
        done_rx.recv().await;
        debug!("received done signal");
        // FIXME
        peer_connection.close().await.unwrap();
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

pub async fn signal(client_offer: ClientOffer) -> Result<impl warp::Reply, warp::Rejection> {
    signal_anyhow(client_offer).await.map_err(error_to_reject)
}
