use crate::*;
use std::mem;

use wgpu::util::DeviceExt;


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Global {
    pub mvp: [[f32; 4]; 4],
}

pub struct GlobalBuffer {
    global_buffer: wgpu::Buffer,
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_group: wgpu::BindGroup
}


impl GlobalBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let global_data = Global {
            mvp: Default::default(),
        };
        let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&global_data),
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
                            min_binding_size: wgpu::BufferSize::new(mem::size_of::<Global>() as _),
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
        GlobalBuffer {
            global_buffer,
            global_bind_group_layout,
            global_group
        }
    }
    pub fn write(ctx: &GraphicsContext, global_data: &Global) {
        let GraphicsContext {
            queue,
            global,
            ..
        } = ctx;
        queue.write_buffer(&global.global_buffer, 0, bytemuck::bytes_of(global_data));
    }
}