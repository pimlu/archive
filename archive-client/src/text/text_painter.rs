use crate::*;

use wgpu::CommandBuffer;
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder};

pub struct TextPainter {}

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
        TextPainter {}
    }

    pub fn render(
        &mut self,
        ctx: &GraphicsContext,
        view: &wgpu::TextureView,
        glyph_brush: &mut GlyphBrush<()>,
        text: &[wgpu_glyph::Section],
    ) -> CommandBuffer {
        let GraphicsContext { device, queue, .. } = ctx;
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for section in text {
            glyph_brush.queue(section);
        }

        let transform: [f32; 16] = bytemuck::cast(ctx.global.get_data().mvp);
        glyph_brush
            .draw_queued_with_transform(&device, queue, &mut encoder, view, transform)
            .unwrap();

        encoder.finish()
    }

    pub fn post_frame(&mut self, _ctx: &GraphicsContext) {}
}
