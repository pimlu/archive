mod native_random;
mod tungstenite_client_rtc;

use std::sync::mpsc;

use archive_client::*;
use archive_engine::{rtc::RtcServerDescriptor, *};
use log::error;
use native_random::NativeRandomBuilder;

#[tokio::main]
async fn main() {
    random::register(NativeRandomBuilder {});
    launch_config::register(launch_config::LaunchConfig { sample_count: 1 });

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    env_logger::init();
    let (tx, rx) = mpsc::channel();

    let server_handle = tungstenite_client_rtc::TungsteniteServerHandle {
        hostname: "ws://localhost:8080".into(),
    };
    match server_handle.rtc_connect().await {
        Ok(session) => {
            tx.send(client::ClientMessageFromApp::Connected(session))
                .unwrap();
        }
        Err(e) => {
            error!("failed to connect: {e}");
        }
    };

    let run = run_init(event_loop, window, rx).await;
    run();
}
