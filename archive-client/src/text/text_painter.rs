use crate::*;

use wgpu::CommandBuffer;
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder};

pub struct TextPainter {
    pub(super) staging_belt: wgpu::util::StagingBelt,
    pub(super) local_pool: futures::executor::LocalPool,
}

pub fn glyph_brush_from_font(ctx: &GraphicsContext, font: Vec<u8>) -> GlyphBrush<()> {
    let font_arc = ab_glyph::FontArc::try_from_vec(font).unwrap();

    let glyph_brush = GlyphBrushBuilder::using_font(font_arc).build(&ctx.device, ctx.config.format);
    glyph_brush
}

impl TextPainter {
    pub fn new(
        _graphics: &GraphicsContext,
        _global_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = futures::executor::LocalPool::new();

        TextPainter {
            staging_belt,
            local_pool,
        }
    }

    pub fn render(
        &mut self,
        ctx: &GraphicsContext,
        view: &wgpu::TextureView,
        glyph_brush: &mut GlyphBrush<()>,
        text: &[wgpu_glyph::Section],
    ) -> CommandBuffer {
        let GraphicsContext { device, .. } = ctx;
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for section in text {
            glyph_brush.queue(section);
        }

        let transform: [f32; 16] = bytemuck::cast(ctx.global.get_data().mvp);
        glyph_brush
            .draw_queued_with_transform(
                &device,
                &mut self.staging_belt,
                &mut encoder,
                view,
                transform,
            )
            .unwrap();

        self.staging_belt.finish();
        encoder.finish()
    }

    pub fn post_frame(&mut self, _ctx: &GraphicsContext) {
        use futures::task::SpawnExt;
        let local_spawner = self.local_pool.spawner();
        local_spawner.spawn(self.staging_belt.recall()).unwrap();

        self.local_pool.run_until_stalled();
    }
}
