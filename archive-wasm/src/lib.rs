mod wasm_random;
mod wasm_rtc;
use wasm_rtc::*;
use web_sys::HtmlCanvasElement;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

use archive_client::*;
use archive_engine::rtc::{RtcServerDescriptor, BoxedRtcSession};
use archive_engine::*;
use js_sys::Reflect;
use wasm_random::WasmRandomBuilder;
use wasm_rtc::WasmClientSession;
use winit::event_loop::EventLoop;
use winit::platform::web::{WindowBuilderExtWebSys, WindowExtWebSys};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub struct WasmClient {
    tx: mpsc::Sender<client::ClientMessageFromApp>,
}

#[wasm_bindgen(js_name=startClient)]
pub async fn start_client() -> Result<JsValue, JsValue> {
    random::register(WasmRandomBuilder {});
    launch_config::register(launch_config::LaunchConfig { sample_count: 4 });

    let canvas: HtmlCanvasElement = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id("game"))
        .and_then(|elem| elem.dyn_into().ok())
        .expect("could not get canvas");

    let event_loop = EventLoop::new();

    let builder = winit::window::WindowBuilder::new();
    let builder = builder.with_canvas(Some(canvas));
    let window = builder.build(&event_loop).unwrap();

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    let (tx, rx) = mpsc::channel();
    let run = run_init(event_loop, window, rx).await;
    let run = Closure::once_into_js(run);

    let client = JsValue::from(WasmClient { tx });

    // annoying issue:
    // https://github.com/rustwasm/wasm-bindgen/issues/2486
    // which means we can't bundle run into our client object

    // return run to be called later, outside the executor, since panics break wasm-bindgen-futures

    let res = js_sys::Object::new();
    Reflect::set(&res, &JsValue::from("client"), &client)?;
    Reflect::set(&res, &JsValue::from("run"), &run)?;
    Ok(JsValue::from(res))
}

#[wasm_bindgen(js_name=useConnection)]
pub fn use_connection(
    wasm_client: &mut WasmClient,
    connection: WasmConnection,
) -> Result<(), JsValue> {
    let WasmConnection { session } = connection;
    let msg = client::ClientMessageFromApp::Connected(session);

    wasm_client.tx.send(msg).or_else(fmt_jserr)
}

#[wasm_bindgen]
pub struct WasmConnection {
    session: BoxedRtcSession,
}

#[wasm_bindgen]
pub async fn connect(hostname: String) -> Result<WasmConnection, JsValue> {
    let handle = WasmServerHandle { hostname };
    let session = handle.rtc_connect().await?;

    // let res = js_sys::Object::new();
    // let pc = JsValue::from(session.peer_connection);
    // let dc = JsValue::from(session.data_channel);
    // Reflect::set(&res, &JsValue::from("pc"), &pc)?;
    // Reflect::set(&res, &JsValue::from("dc"), &dc)?;
    // Ok(JsValue::from(res))
    Ok(WasmConnection { session })
}
