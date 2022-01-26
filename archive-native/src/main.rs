mod native_random;

use archive_client::run_init;
use archive_engine::random;
use native_random::NativeRandomBuilder;

fn main() {
    random::register(NativeRandomBuilder {});

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    env_logger::init();
    let run = pollster::block_on(run_init(event_loop, window));
    run();
}
