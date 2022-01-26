use crate::*;
use std::{borrow::Cow, mem};

use bytemuck::{Zeroable, Pod};
use wgpu::util::DeviceExt;

pub struct SpritePainter {
    global_group: wgpu::BindGroup,
    local_buffer: wgpu::Buffer,
    local_group: wgpu::BindGroup,

    render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    mvp: [[f32; 4]; 4],
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
    // NOTE: you MUST include this in wgsl to fix a metal stride bug
    pub _pad: [u32; 2]
}


impl SpritePainter {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("sprite_shader.wgsl"))),
        });

        // TODO resize this thing when thw window resizes, or more likely pass in a camera matrix
        let globals = Globals {
            mvp: cgmath::ortho(
                0.0,
                config.width as f32,
                0.0,
                config.height as f32,
                -1.0,
                1.0,
            )
            .into(),
        };
        let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(mem::size_of::<Globals>() as _),
                        },
                        count: None,
                    },
                    ],
                    label: None,
                });
        
        let global_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let local_size = MAX_SPRITES * mem::size_of::<GpuSprite>();
        let local_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: local_size as _,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(local_size as _),
                    },
                    count: None,
                }],
                label: None,
            });
        
        let local_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &local_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &local_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(local_size as _),
                    }),
                },
            ],
            label: None,
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
            bind_group_layouts: &[&global_bind_group_layout, &local_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        SpritePainter {
            global_group,
            local_buffer,
            local_group,
            render_pipeline,
            texture_bind_group_layout,
        }
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sprite_texture: &SpriteTexture,
        sprites: &[GpuSprite]
    ) {
        queue.write_buffer(&self.local_buffer, 0, bytemuck::cast_slice(sprites));
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.global_group, &[]);
            render_pass.set_bind_group(1, &self.local_group, &[]);
            render_pass.set_bind_group(2, &sprite_texture.bind_group, &[]);
            render_pass.draw(0..4 as u32, 0..sprites.len() as u32);
        }
        // done
        queue.submit(Some(encoder.finish()));
    }
}
