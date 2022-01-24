mod handler;
mod server_rtc;

use warp::Filter;

async fn _handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    Ok(warp::reply::json(&format!("{:?}", err)))
}

#[tokio::main]
async fn main() {
    env_logger::init();
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

    let signal = warp::post()
        .and(warp::path("signal"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(handler::signal)
        .recover(_handle_rejection)
        .with(cors);

    warp::serve(signal).run(([127, 0, 0, 1], 3030)).await
}
