use crate::*;
use log::info;

/// Example struct holds references to wgpu resources and frame persistent data
pub struct App {
    frame_counter: FrameCounter,
    sprite_painter: SpritePainter,
    sprite_texture: SpriteTexture,
}

impl App {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let frame_counter = FrameCounter::default();
        let sprite_painter = SpritePainter::init(config, adapter, device, queue);

        let texture_handle = TextureHandle::init(device, queue, include_bytes!("missing.png"));
        let sprite_texture = SpriteTexture::init(
            device,
            &sprite_painter.texture_bind_group_layout,
            texture_handle,
        );

        App {
            frame_counter,
            sprite_painter,
            sprite_texture,
        }
    }

    /// update is called for any WindowEvent not handled by the framework
    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    /// resize is called on WindowEvent::Resized events
    fn resize(
        &mut self,
        _sc_desc: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        //empty
    }

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let (_last_frametime, fps_opt) = self.frame_counter.tick();
        if let Some(fps) = fps_opt {
            info!("fps: {}", fps);
        }
        self.sprite_painter
            .render(view, device, queue, &self.sprite_texture);
    }
}
