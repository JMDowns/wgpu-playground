use std::num::NonZeroU32;
use wgpu::{Device, Queue, SurfaceConfiguration};
use crate::texture;

pub struct TextureState {
    pub diffuse_bind_group_layout: wgpu::BindGroupLayout,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub depth_texture: texture::Texture,
}

impl TextureState {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let atlas_rgba_bytes = include_bytes!("../data.atl");

        let mut block_texture_vec = Vec::new();
        let mut block_texture_view_vec = Vec::new();
        let mut block_texture_view_reference_vec = Vec::new();

        let texture_size = wgpu::Extent3d {
            width: fundamentals::consts::TEXTURE_DIMENSION,
            height: fundamentals::consts::TEXTURE_DIMENSION,
            depth_or_array_layers: 1
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
        };

        let tex_dim_squared = (fundamentals::consts::TEXTURE_DIMENSION * fundamentals::consts::TEXTURE_DIMENSION) as usize;

        for i in 0..fundamentals::consts::NUM_TEXTURES {
            let block_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture 1"),
                ..texture_descriptor
            });

            let block_texture_view = block_texture.create_view(&wgpu::TextureViewDescriptor::default());

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &block_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &atlas_rgba_bytes[(4*i*tex_dim_squared)..((4*(i+1))*tex_dim_squared)], 
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4*texture_size.width),
                    rows_per_image: NonZeroU32::new(texture_size.height),
                }, 
                texture_size
            );

            block_texture_vec.push(block_texture);
            block_texture_view_vec.push(block_texture_view);
        }

        for block_texture_view in block_texture_view_vec.iter() {
            block_texture_view_reference_vec.push(block_texture_view);
        }

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        let texture_array_bind_group_layout = 
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
                        count: NonZeroU32::new(fundamentals::consts::NUM_TEXTURES as u32)
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

        let texture_array_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_array_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&block_texture_view_reference_vec[..]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        TextureState { 
            diffuse_bind_group_layout: texture_array_bind_group_layout, 
            diffuse_bind_group: texture_array_bind_group, 
            depth_texture: depth_texture 
        }
    }
}