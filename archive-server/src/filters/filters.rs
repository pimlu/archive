use std::collections::BTreeMap;

use super::*;
use crate::*;

use tokio::sync::RwLock;
use warp::Filter;

pub async fn _handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    Ok(warp::reply::json(&format!("{:?}", err)))
}

pub async fn warp_main() {
    env_logger::init();

    let arena_map = arena::ArenaMapLock::default();

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
        .and_then(rtc_signal)
        .recover(_handle_rejection)
        .with(cors);

    warp::serve(signal).run(([127, 0, 0, 1], 3030)).await
}

pub fn add_map_filter(
    arena_map: arena::ArenaMapLock,
) -> impl warp::Filter<Extract = (arena::ArenaMapLock,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || arena_map.clone())
}
