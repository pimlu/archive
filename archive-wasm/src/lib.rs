mod wasm_random;
mod wasm_rtc;

use archive_client::{launch_config, run_init};
use js_sys::Reflect;
use wasm_random::WasmRandomBuilder;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowExtWebSys;

use archive_engine::{random, RtcServerHandle};

use wasm_bindgen::prelude::*;

use crate::wasm_rtc::WasmServerHandle;

#[wasm_bindgen]
pub async fn start_loop() -> JsValue {
    random::register(WasmRandomBuilder {});
    launch_config::register(launch_config::LaunchConfig { sample_count: 4 });

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

#[wasm_bindgen]
pub async fn connect(hostname: String) -> Result<JsValue, JsValue> {
    let handle = WasmServerHandle { hostname };
    let session = handle.rtc_connect().await?;

    let res = js_sys::Object::new();
    let pc = JsValue::from(session.peer_connection);
    let dc = JsValue::from(session.data_channel);
    Reflect::set(&res, &JsValue::from("pc"), &pc)?;
    Reflect::set(&res, &JsValue::from("dc"), &dc)?;
    Ok(JsValue::from(res))
}
