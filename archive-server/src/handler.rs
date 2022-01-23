use std::convert::Infallible;

use archive_engine::{ClientOffer, ServerAnswer};

pub async fn connect(offer: ClientOffer) -> Result<impl warp::Reply, Infallible> {
    let answer = ServerAnswer {
        sdp: offer.sdp + " world",
    };
    Ok(warp::reply::json(&answer))
}
