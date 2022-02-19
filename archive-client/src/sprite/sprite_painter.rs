use super::*;
use crate::*;
use std::{borrow::Cow, mem};

use bytemuck::{Pod, Zeroable};
use wgpu::CommandBuffer;

pub struct SpritePainter {
    instance_buffer: wgpu::Buffer,

    render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

const MAX_SPRITES: usize = 512;
// 256 bit minimum alignment imposed by nvidia or something. there is also
// min_uniform_buffer_offset_alignment which basically means the GPU could
// in theory do better, but I don't want to mess with that at runtime
#[repr(C, align(32))]
#[derive(Clone, Copy, Pod, Zeroable, Default, Debug)]
pub struct GpuSprite {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: u32,
    pub _pad: [u32; 2],
}

impl GpuSprite {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GpuSprite>() as wgpu::BufferAddress,
            // this is the magic that gives you instance buffers
            step_mode: wgpu::VertexStepMode::Instance,
            // TODO use offset_of when const macro is stabilized
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0, //offset!(position),
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // size
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // rotation
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                // color
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

impl SpritePainter {
    pub fn init(
        graphics: &GraphicsContext,
        global_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let GraphicsContext { config, device, .. } = graphics;
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("sprite_shader.wgsl"))),
        });

        let local_size = MAX_SPRITES * mem::size_of::<GpuSprite>();
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: local_size as _,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[global_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[GpuSprite::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[config.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: launch_config::multisample_state(),
            multiview: None,
        });

        SpritePainter {
            instance_buffer,
            render_pipeline,
            texture_bind_group_layout,
        }
    }

    pub fn render(
        &mut self,
        ctx: &GraphicsContext,
        view: &wgpu::TextureView,
        sprite_texture: &SpriteTexture,
        sprites: &[GpuSprite],
    ) -> CommandBuffer {
        let GraphicsContext { queue, device, .. } = ctx;
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(sprites));
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &ctx.global.global_group, &[]);
            render_pass.set_bind_group(1, &sprite_texture.bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            render_pass.draw(0..4 as u32, 0..sprites.len() as u32);
        }

        encoder.finish()
    }
}
