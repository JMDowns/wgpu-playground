use itertools::Chunk;
use fundamentals::world_position::WorldPosition;
use std::collections::HashMap;
use cgmath::Point3;
use wgpu::Device;
use wgpu::util::DeviceExt;

pub struct ChunkIndexState {
    pub chunk_index_array: Vec<WorldPosition>,
    pub chunk_index_buffer: wgpu::Buffer,
    pub chunk_index_bind_group_layout: wgpu::BindGroupLayout,
    pub chunk_index_bind_group: wgpu::BindGroup,
    pub pos_to_gpu_index: HashMap<WorldPosition, usize>,
}

impl ChunkIndexState {
    pub fn new(camera_position: Point3<f32>, device: &Device) -> Self {
        let chunk_index_array = fundamentals::consts::get_positions_around_player(WorldPosition::from(camera_position));
        let mut pos_to_gpu_index = HashMap::new();
        for (i, chunk_position) in chunk_index_array.iter().enumerate() {
            pos_to_gpu_index.insert(*chunk_position, i);
        }
        let chunk_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Index Buffer"),
            contents: bytemuck::cast_slice(&chunk_index_array),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let chunk_index_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None
                }
            ],
            label: Some("chunk_offset_bind_group_layout")
        });

        let chunk_index_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &chunk_index_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: chunk_index_buffer.as_entire_binding()
                }
            ],
            label: Some("chunk_index_bind_group")
        });

        ChunkIndexState {
            chunk_index_array,
            chunk_index_buffer,
            chunk_index_bind_group,
            chunk_index_bind_group_layout,
            pos_to_gpu_index
        }
    }
}