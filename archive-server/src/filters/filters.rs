use std::collections::HashMap;

use super::*;
use crate::*;

use futures::{FutureExt, StreamExt};
use warp::Filter;

pub async fn _handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    Ok(warp::reply::json(&format!("{:?}", err)))
}

pub async fn warp_serve(arena_map: arena::ArenaMapLock) {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Content-Type",
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
        ])
        .allow_methods(vec!["POST", "GET"]);

    let add_map = add_map_filter(arena_map);

    let signal = warp::post()
        .and(warp::path("signal"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(add_map)
        .and_then(handle_rtc_signal)
        .recover(_handle_rejection)
        .with(cors);

    let ws = warp::path("ws")
        .and(warp::query::<HashMap<String, String>>())
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .and_then(handle_ws);

    warp::serve(signal.or(ws)).run(([127, 0, 0, 1], 3030)).await
}

pub fn add_map_filter(
    arena_map: arena::ArenaMapLock,
) -> impl warp::Filter<Extract = (arena::ArenaMapLock,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || arena_map.clone())
}
