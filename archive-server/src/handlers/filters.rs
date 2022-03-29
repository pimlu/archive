use std::{collections::HashMap, convert::Infallible, net::SocketAddr};

use super::*;
use crate::*;

use anyhow::bail;
use futures::{StreamExt, FutureExt};
use hyper::{Request, Body, Response, service::{make_service_fn, service_fn}, Server};
use warp::{Filter, reply};

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

async fn root_handler(_req: Request<Body>, arena_map: arena::ArenaMapLock) -> Result<Response<Body>, Infallible> {

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Try POSTing data to /echo");
        },
        (&Method::POST, "/echo") => {
            // we'll be back
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        },
    };
    Ok(Response::new("Hello, World".into()))
}
pub async fn hyper_main() {
    env_logger::init();

    let arena_map = arena::ArenaMapLock::default();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(move |_conn| {
        let arena_map_clone = arena_map.clone();
        async {
            // service_fn converts our function into a `Service`
            Ok::<_, Infallible>(service_fn(move |req|
                root_handler(req, arena_map_clone.clone()))
            )
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}