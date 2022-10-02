use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::texture;

pub struct TextureState {
    pub diffuse_bind_group_layout: wgpu::BindGroupLayout,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: texture::Texture,
    pub depth_texture: texture::Texture,
}

impl TextureState {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let diffuse_bytes = include_bytes!("../atlas.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "atlas.png").unwrap();

        let diffuse_bind_group_layout = 
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
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &diffuse_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        TextureState {
            diffuse_bind_group_layout,
            diffuse_bind_group,
            diffuse_texture,
            depth_texture,
        }
    }
}