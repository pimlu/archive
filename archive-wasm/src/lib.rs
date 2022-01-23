use std::fmt::Display;

use archive_client::run_init;
use log::info;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowExtWebSys;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use archive_engine::{ClientOffer, ServerAnswer};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn start_loop() -> JsValue {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    // On wasm, append the canvas to the document body
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");
    let run = run_init(event_loop, window).await;
    // return run to be called later, outside the executor, since panics break wasm-bindgen-futures
    Closure::once_into_js(run)
}

fn fmt_jserr<T>(err: impl Display) -> Result<T, JsValue> {
    Err(JsValue::from(format!("{err}")))
}

#[wasm_bindgen]
pub async fn connect(hostname: String) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let offer = ClientOffer {
        sdp: String::from("hello"),
    };
    let offer_serialized = serde_json::to_string(&offer).or_else(fmt_jserr)?;
    opts.body(Some(&JsValue::from_str(&offer_serialized)));

    let url = format!("{hostname}/connect");

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let headers = request.headers();
    headers.set("Content-Type", "application/json")?;
    headers.set("Accept", "application/json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into()?;

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Use serde to parse the JSON into a struct.
    let branch_info: ServerAnswer = json.into_serde().or_else(fmt_jserr)?;

    // Send the `Branch` struct back to JS as an `Object`.
    Ok(JsValue::from_serde(&branch_info).unwrap())
}
