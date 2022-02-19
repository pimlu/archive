use crate::*;

use archive_engine::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct GraphicsContext {
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub global: GlobalBuffer,
    pub multisampled_fb: Option<wgpu::TextureView>,
}

// this function is mainly playing along with the winit event loop and
// setting up wgpu constructs. But it also passes through client_rx which
// is critical for setting up the client e.g. with connections to the server
pub async fn run_init(
    event_loop: EventLoop<()>,
    window: Window,
    client_rx: client::ClientReceiver,
) -> impl FnOnce() {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &config);

    let global = GlobalBuffer::new(&device);

    let multisampled_fb = launch_config::multisampled_framebuffer(&device, &config);

    let mut ctx = GraphicsContext {
        config,
        adapter,
        device,
        queue,
        global,
        multisampled_fb,
    };

    // TODO I don't like this, we should have a separate function that returns ctx,
    // pass that into App::init, and then another function which does the event loop
    let mut app = App::init(&ctx, client_rx);

    // return a FnOnce so we can "escape" wasm-bindgen-futures and run this
    // call which panics later. (This is what generates the "shouldSuppress"
    // error in App.tsx.) This is done because wasm-bindgen-futures doesn't
    // support panics (the executor stops working after a panic)
    move || {
        event_loop.run(move |event, _, control_flow| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = &instance;

            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    // Reconfigure the surface with the new size
                    ctx.config.width = size.width;
                    ctx.config.height = size.height;
                    surface.configure(&ctx.device, &ctx.config);
                    ctx.multisampled_fb =
                        launch_config::multisampled_framebuffer(&ctx.device, &ctx.config);
                }
                Event::RedrawRequested(_) => {
                    let frame = surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    app.render(&mut ctx, &view);
                    frame.present();
                    app.post_frame(&ctx);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::RedrawEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
                }
                _ => {}
            }
        })
    }
}
