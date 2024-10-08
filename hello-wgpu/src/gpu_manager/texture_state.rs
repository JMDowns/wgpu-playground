use std::num::NonZeroU32;
use wgpu::{Device, Queue, SurfaceConfiguration, util::DeviceExt};
use crate::texture;

pub struct TextureState {
    //pub diffuse_bind_group_layout: wgpu::BindGroupLayout,
    //pub diffuse_bind_group: wgpu::BindGroup,
    pub depth_texture: Option<texture::Texture>,
}

impl TextureState {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        // let atlas_rgba_bytes = include_bytes!("../data.atl");

        // let mut block_texture_vec = Vec::new();
        // let mut block_texture_view_vec = Vec::new();
        // let mut block_texture_view_reference_vec = Vec::new();

        // let texture_size = wgpu::Extent3d {
        //     width: fundamentals::consts::TEXTURE_DIMENSION,
        //     height: fundamentals::consts::TEXTURE_DIMENSION,
        //     depth_or_array_layers: 1
        // };

        // let texture_descriptor = wgpu::TextureDescriptor {
        //     size: texture_size,
        //     mip_level_count: fundamentals::consts::MIP_LEVEL,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
        //     usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        //     label: None,
        //     view_formats: &[]
        // };

        // for i in 0..fundamentals::consts::NUM_TEXTURES {
        //     let block_texture = device.create_texture_with_data(
        //         queue,
        //         &wgpu::TextureDescriptor {
        //             label: Some(&format!("Texture {i}")),
        //             ..texture_descriptor
        //         }, 
        //         wgpu::util::TextureDataOrder::LayerMajor,
        //         &atlas_rgba_bytes[i*fundamentals::consts::TEXTURE_LENGTH_WITH_MIPMAPS*4..(i+1)*fundamentals::consts::TEXTURE_LENGTH_WITH_MIPMAPS*4]
        //     );

        //     let block_texture_view = block_texture.create_view(&wgpu::TextureViewDescriptor::default());

        //     block_texture_vec.push(block_texture);
        //     block_texture_view_vec.push(block_texture_view);
        // }

        // for block_texture_view in block_texture_view_vec.iter() {
        //     block_texture_view_reference_vec.push(block_texture_view);
        // }

        // let sampler = device.create_sampler(
        //     &wgpu::SamplerDescriptor {
        //         address_mode_u: wgpu::AddressMode::Repeat,
        //         address_mode_v: wgpu::AddressMode::Repeat,
        //         address_mode_w: wgpu::AddressMode::Repeat,
        //         mag_filter: wgpu::FilterMode::Nearest,
        //         min_filter: wgpu::FilterMode::Nearest,
        //         mipmap_filter: wgpu::FilterMode::Linear,
        //         ..Default::default()
        //     }
        // );

        // let texture_array_bind_group_layout = 
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Texture { 
        //                     multisampled: false,
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                 },
        //                 count: NonZeroU32::new(fundamentals::consts::NUM_TEXTURES as u32)
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 1,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 // This should match the filterable field of the corresponding Texture entry above.
        //                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //                 count: None,
        //             },
        //         ],
        //         label: Some("texture_bind_group_layout"),
        //     });

        // let texture_array_bind_group = device.create_bind_group(
        //     &wgpu::BindGroupDescriptor {
        //         layout: &texture_array_bind_group_layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: wgpu::BindingResource::TextureViewArray(&block_texture_view_reference_vec[..]),
        //             },
        //             wgpu::BindGroupEntry {
        //                 binding: 1,
        //                 resource: wgpu::BindingResource::Sampler(&sampler),
        //             }
        //         ],
        //         label: Some("diffuse_bind_group"),
        //     }
        // );

        #[cfg(not(target_family = "wasm"))]
        let depth_texture = Some(texture::Texture::create_depth_texture(&device, &config, "depth_texture"));

        #[cfg(target_family = "wasm")]
        let depth_texture = None;

        TextureState { 
           // diffuse_bind_group_layout: texture_array_bind_group_layout, 
           // diffuse_bind_group: texture_array_bind_group, 
            depth_texture
        }
    }
}