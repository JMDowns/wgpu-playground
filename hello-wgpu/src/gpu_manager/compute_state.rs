use std::collections::HashMap;

use bytemuck::{Zeroable, Pod};
use cgmath::Point3;
use fundamentals::enums::block_side::BlockSide;
use fundamentals::world_position::WorldPosition;
use wgpu::{Device, Buffer, util::DeviceExt};

use super::compute_state_generated_helper;

use super::gpu_data::vertex_gpu_data::BucketPosition;

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct FrustumBucketData {
    pub buffer_index: i32,
    pub bucket_index: i32,
    pub side: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct FrustumComputeData {
    pub world_position: WorldPosition,
    pub bucket_data: [FrustumBucketData; fundamentals::consts::NUM_BUCKETS_PER_CHUNK]
}

pub struct ComputeState {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub compute_indirect_bind_groups: Vec<wgpu::BindGroup>,
    pub compute_bucket_data_buffer: wgpu::Buffer,
    pub compute_staging_vec: Vec<u32>,
    pub world_position_to_compute_data_offset: HashMap<WorldPosition, usize>
}

impl ComputeState {
    pub fn new(camera_position: Point3<f32>, device: &Device, camera_buffer: &Buffer, indirect_buffers: &Vec<Buffer>, visibility_buffer: &Buffer) -> Self {
        let frustum_compute_data: Vec<FrustumComputeData> = fundamentals::consts::get_positions_around_player(WorldPosition::from(camera_position)).into_iter()
            .map(|pos| FrustumComputeData {
                world_position: pos,
                bucket_data: [FrustumBucketData {
                    buffer_index: -1,
                    bucket_index: -1,
                    side: 0,
                }; fundamentals::consts::NUM_BUCKETS_PER_CHUNK]
            }).collect();

        let mut world_position_to_compute_data_offset = HashMap::new();
        for (i, pos) in fundamentals::consts::get_positions_around_player(WorldPosition::from(camera_position)).into_iter().enumerate() {
            world_position_to_compute_data_offset.insert(pos, i * std::mem::size_of::<FrustumComputeData>());
        }

        let compute_bucket_data_buffer = device.create_buffer_init( &wgpu::util::BufferInitDescriptor {
            label: Some("Compute Input Buffer"),
            contents: bytemuck::cast_slice(&frustum_compute_data),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../gpu_manager/shaders/frustum_compute.wgsl").into()),
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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                }
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
                    resource: compute_bucket_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: visibility_buffer.as_entire_binding(),
                },
                
            ]
        });

        let (compute_indirect_bind_groups, indirect_bind_group_layouts) = compute_state_generated_helper::return_bind_group_and_layout(device, indirect_buffers);

        let mut compute_pipeline_bind_group_layouts = vec![&compute_bind_group_layout];
        for layout in indirect_bind_group_layouts.iter() {
            compute_pipeline_bind_group_layouts.push(layout);
        }

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &compute_pipeline_bind_group_layouts,
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
            compute_indirect_bind_groups,
            compute_bucket_data_buffer,
            compute_pipeline,
            compute_staging_vec: vec![0; fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize],
            world_position_to_compute_data_offset
        }
    }

    pub fn update_frustum_bucket_data(&mut self, mesh_position: WorldPosition, side: BlockSide, side_offset: u32, bucket_position: BucketPosition, queue: &wgpu::Queue) {
        let data_beginning_offset = *self.world_position_to_compute_data_offset.get(&mesh_position).unwrap();
        let offset = (data_beginning_offset as usize + std::mem::size_of::<WorldPosition>() + std::mem::size_of::<FrustumBucketData>() * (side as u32 * fundamentals::consts::NUM_BUCKETS_PER_SIDE + side_offset) as usize) as u64;
        queue.write_buffer(&self.compute_bucket_data_buffer, offset, bytemuck::cast_slice(&[bucket_position.buffer_number, bucket_position.bucket_number, side as i32]));
    }

    pub fn clear_frustum_bucket_data(&mut self, mesh_position: WorldPosition, side: BlockSide, side_offset: u32, queue: &wgpu::Queue) {
        let data_beginning_offset = *self.world_position_to_compute_data_offset.get(&mesh_position).unwrap();
        let offset = (data_beginning_offset as usize + std::mem::size_of::<WorldPosition>() + std::mem::size_of::<FrustumBucketData>() * (side as u32 * fundamentals::consts::NUM_BUCKETS_PER_SIDE + side_offset) as usize) as u64;
        queue.write_buffer(&self.compute_bucket_data_buffer, offset, bytemuck::cast_slice(&[-1, -1, side as i32]));
    }
}