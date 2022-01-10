use archive_client::run;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowExtWebSys;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn start_loop() {
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
    wasm_bindgen_futures::spawn_local(run(event_loop, window));
}
