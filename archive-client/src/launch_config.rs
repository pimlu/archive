use crate::*;

use log::warn;
use once_cell::sync::OnceCell;
use wgpu::{MultisampleState, RenderPassColorAttachment};

pub struct LaunchConfig {
    pub sample_count: u32,
}

static LAUNCH_CONFIG: OnceCell<LaunchConfig> = OnceCell::new();
fn get() -> &'static LaunchConfig {
    LAUNCH_CONFIG.get().expect("no reigstered config")
}

// sets the global builder for RNGs.
pub fn register(launch_config: LaunchConfig) {
    let res = LAUNCH_CONFIG.set(launch_config);
    if res.is_err() {
        warn!("already registered config");
    }
}

pub fn color_attachment<'a>(
    ctx: &'a GraphicsContext,
    view: &'a wgpu::TextureView,
) -> RenderPassColorAttachment<'a> {
    let cfg = get();
    if cfg.sample_count == 1 {
        RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }
    } else {
        RenderPassColorAttachment {
            view: ctx.multisampled_fb.as_ref().unwrap(),
            resolve_target: Some(view),
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }
    }
}
pub fn multisample_state() -> MultisampleState {
    let cfg = get();
    MultisampleState {
        count: cfg.sample_count,
        ..Default::default()
    }
}

pub fn multisampled_framebuffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> Option<wgpu::TextureView> {
    let cfg = get();
    if cfg.sample_count == 1 {
        return None;
    }
    let multisampled_texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        mip_level_count: 1,
        sample_count: cfg.sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    };

    let view = device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default());
    Some(view)
}
