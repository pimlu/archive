use crate::*;
use archive_engine::random;
use log::info;

/// Example struct holds references to wgpu resources and frame persistent data
pub struct App {
    frame_counter: FrameCounter,
    sprite_painter: SpritePainter,
    sprite_texture: SpriteTexture,
    sprites: Vec<GpuSprite>
}

impl App {
    pub fn init(graphics: &GraphicsContext) -> Self {
        let GraphicsContext {
            device,
            queue,
            global,
            ..
        } = graphics;
        let frame_counter = FrameCounter::default();
        let sprite_painter = SpritePainter::init(graphics, &global.global_bind_group_layout);

        let texture_handle = TextureHandle::init(device, queue, include_bytes!("missing.png"));
        let sprite_texture = SpriteTexture::init(
            device,
            &sprite_painter.texture_bind_group_layout,
            texture_handle,
        );

        //let mut rng = random::new();

        let mut sprites = Vec::new();
        for i in 0..10 {
            sprites.push(GpuSprite {
                position: [i as f32 * 200. + 550., i as f32 * 100. + 550.],//[rng.gen32(), rng.gen32()],
                size: [800., 800.],
                rotation: i as f32 / 3.,
                color: 0xffffffff,
                ..Default::default()
            });
        }
        App {
            frame_counter,
            sprite_painter,
            sprite_texture,
            sprites
        }
    }

    /// update is called for any WindowEvent not handled by the framework
    fn _update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    /// resize is called on WindowEvent::Resized events
    fn _resize(
        &mut self,
        _sc_desc: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        //empty
    }

    pub fn render(&mut self, ctx: &GraphicsContext, view: &wgpu::TextureView) {
        let (_last_frametime, fps_opt) = self.frame_counter.tick();
        if let Some(fps) = fps_opt {
            info!("fps: {}", fps);
        }
        // update the viewport
        let global_data = Global {
            mvp: cgmath::ortho(
                0.0,
                ctx.config.width as f32,
                0.0,
                ctx.config.height as f32,
                -1.0,
                1.0,
            )
            .into(),
        };
        GlobalBuffer::write(ctx, &global_data);

        self.sprite_painter
            .render(ctx, view, &self.sprite_texture, &self.sprites);
    }
}
