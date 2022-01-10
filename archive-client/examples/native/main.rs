use archive_client::run;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    env_logger::init();
    pollster::block_on(run(event_loop, window));
}
