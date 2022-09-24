use bytemuck::{Zeroable, Pod};
use cgmath::Point3;
use fundamentals::world_position::WorldPosition;
use wgpu::{Device, Buffer, util::DeviceExt};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct FrustumBucketData {
    pub buffer_index: i32,
    pub bucket_index: i32,
    pub vertex_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct FrustumComputeData {
    pub world_position: WorldPosition,
    pub bucket_data: [FrustumBucketData; 12]
}

pub struct ComputeState {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub compute_input_buffer: wgpu::Buffer,
    pub compute_output_buffer: wgpu::Buffer,
    pub compute_staging_vec: Vec<u32>,
}

impl ComputeState {
    pub fn new(camera_position: Point3<f32>, device: &Device, camera_buffer: &Buffer) -> Self {
        let frustum_compute_data: Vec<FrustumComputeData> = fundamentals::consts::get_positions_around_player(WorldPosition::from(camera_position)).into_iter()
            .map(|pos| FrustumComputeData {
                world_position: pos,
                bucket_data: [FrustumBucketData {
                    buffer_index: -1,
                    bucket_index: -1,
                    vertex_count: 0
                }; 12]
            }).collect();

        let compute_input_buffer = device.create_buffer_init( &wgpu::util::BufferInitDescriptor {
            label: Some("Compute Input Buffer"),
            contents: bytemuck::cast_slice(&frustum_compute_data),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE
        });

        let compute_output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute Output Buffer"),
            size: std::mem::size_of::<u32>() as u64 * fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../frustum_compute.wgsl").into()),
        });

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
            ],
            label: Some("Compute Bind Group Layout")
        });

        let compute_bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: compute_input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: compute_output_buffer.as_entire_binding(),
                }
            ]
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[
                &compute_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main"
        });

        ComputeState {
            compute_bind_group,
            compute_input_buffer,
            compute_output_buffer,
            compute_pipeline,
            compute_staging_vec: vec![0; fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize]
        }
    }
}