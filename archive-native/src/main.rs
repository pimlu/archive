mod native_random;

use std::sync::mpsc;

use archive_client::{launch_config, run_init};
use archive_engine::random;
use native_random::NativeRandomBuilder;

fn main() {
    random::register(NativeRandomBuilder {});
    launch_config::register(launch_config::LaunchConfig { sample_count: 1 });

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    env_logger::init();
    let (_tx, rx) = mpsc::channel();
    let run = pollster::block_on(run_init(event_loop, window, rx));
    run();
}
