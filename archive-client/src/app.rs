use crate::*;
use archive_engine::random;

/// Example struct holds references to wgpu resources and frame persistent data
pub struct App {
    frame_counter: FrameCounter,
    sprite_painter: sprite::SpritePainter,
    sprite_texture: sprite::SpriteTexture,
    sprites: Vec<sprite::GpuSprite>,
    text_painter: text::TextPainter,
    inconsolata: wgpu_glyph::GlyphBrush<()>,
    last_fps: f64,
}

impl App {
    pub fn init(ctx: &GraphicsContext) -> Self {
        let GraphicsContext {
            device,
            queue,
            global,
            ..
        } = ctx;
        let frame_counter = FrameCounter::default();
        let sprite_painter = sprite::SpritePainter::init(ctx, &global.global_bind_group_layout);

        let texture_handle =
            sprite::TextureHandle::new(device, queue, include_asset!("textures/missing.png"));
        let sprite_texture = sprite::SpriteTexture::init(
            device,
            &sprite_painter.texture_bind_group_layout,
            texture_handle,
        );

        //let mut rng = random::new();

        let mut sprites = Vec::new();
        for i in 0..10 {
            sprites.push(sprite::GpuSprite {
                position: [i as f32 * 200. + 550., i as f32 * 100. + 550.], //[rng.gen32(), rng.gen32()],
                size: [800., 800.],
                rotation: i as f32 / 3.,
                color: 0xffffffff,
                ..Default::default()
            });
        }

        let text_painter = text::TextPainter::new(ctx, &global.global_bind_group_layout);
        let inconsolata =
            text::glyph_brush_from_font(ctx, include_asset!("fonts/Rubik-Regular.ttf").to_vec());

        App {
            frame_counter,
            sprite_painter,
            sprite_texture,
            sprites,
            text_painter,
            inconsolata,
            last_fps: 0.,
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
            self.last_fps = fps;
        }
        // update the viewport
        let global_data = Global {
            mvp: cgmath::ortho(
                0.0,
                ctx.config.width as f32,
                ctx.config.height as f32,
                0.0,
                -1.0,
                1.0,
            )
            .into(),
        };
        GlobalBuffer::write(ctx, global_data);

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        // no MSAA needed for clearing
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                    ..launch_config::color_attachment(ctx, view)
                }],
                depth_stencil_attachment: None,
            });

        let clear = encoder.finish();

        let sprites = self
            .sprite_painter
            .render(ctx, view, &self.sprite_texture, &self.sprites);

        let fps_str = format!("FPS: {}", self.last_fps.round() as i64);
        let fps_section = wgpu_glyph::Section {
            screen_position: (30.0, 30.0),
            bounds: (ctx.config.width as f32, ctx.config.height as f32),
            text: vec![wgpu_glyph::Text::new(&fps_str)
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(40.0)],
            ..Default::default()
        };
        let texts = self
            .text_painter
            .render(ctx, view, &mut self.inconsolata, &[fps_section]);
        ctx.queue.submit([clear, sprites, texts]);
    }

    pub fn post_frame(&mut self, ctx: &GraphicsContext) {
        self.text_painter.post_frame(ctx);
    }
}
