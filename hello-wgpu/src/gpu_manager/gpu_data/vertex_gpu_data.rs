use std::collections::HashMap;
use cgmath::Point3;
use derivables::vertex::Vertex;
use fundamentals::world_position::WorldPosition;
use wgpu::{Device, util::DeviceExt, BufferUsages};

use crate::voxels::mesh::Mesh;

use super::vec_vertex_index_length_triple::VecVertexIndexLengthsTriple;

pub const NUM_BUCKETS: usize = (fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize) * fundamentals::consts::NUM_BUCKETS_PER_CHUNK;

pub struct BucketData {
    pub vertex_buffer_number: usize,
    pub vertex_offset: usize,
    pub index_buffer_number: usize,
    pub index_offset: usize,
    pub num_indices: usize
}

pub struct MeshBucketData {
    pub front_bucket_data_indices: Vec<u32>,
    pub back_bucket_data_indices: Vec<u32>,
    pub left_bucket_data_indices: Vec<u32>,
    pub right_bucket_data_indices: Vec<u32>,
    pub top_bucket_data_indices: Vec<u32>,
    pub bottom_bucket_data_indices: Vec<u32>,
}

pub struct VertexGPUData {
    pub data_front: VecVertexIndexLengthsTriple,
    pub data_back: VecVertexIndexLengthsTriple,
    pub data_left: VecVertexIndexLengthsTriple,
    pub data_right: VecVertexIndexLengthsTriple,
    pub data_top: VecVertexIndexLengthsTriple,
    pub data_bottom: VecVertexIndexLengthsTriple,
    pub chunk_index_array: Vec<WorldPosition>,
    pub chunk_index_buffer: wgpu::Buffer,
    pub chunk_index_bind_group_layout: wgpu::BindGroupLayout,
    pub chunk_index_bind_group: wgpu::BindGroup,
    pub pos_to_gpu_index: HashMap<WorldPosition, usize>,
    pub pos_to_loaded_index: HashMap<WorldPosition, usize>,
    pub loaded_chunks: Vec<WorldPosition>,
    //Vertex Pooling
    pub vertex_pool_buffers: Vec<wgpu::Buffer>,
    pub index_pool_buffers: Vec<wgpu::Buffer>,
    pub indirect_pool_buffers: Vec<wgpu::Buffer>,
    pub vertex_count_pool_buffers: Vec<wgpu::Buffer>,
    pub pool_position_to_mesh_bucket_data: HashMap<WorldPosition, MeshBucketData>,
}

impl VertexGPUData {
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
                    visibility: wgpu::ShaderStages::VERTEX,
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

        const MAX_BUFFER_SIZE: usize = 1073741824;

        let vertex_bucket_size = std::mem::size_of::<Vertex>() * fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize;
        let number_of_vertex_buckets_per_buffer = MAX_BUFFER_SIZE / vertex_bucket_size;
        let mut number_of_vertex_pool_buffers = NUM_BUCKETS / number_of_vertex_buckets_per_buffer;
        if NUM_BUCKETS % number_of_vertex_buckets_per_buffer != 0 {
            number_of_vertex_pool_buffers += 1;
        }

        // * 3/2 = 6/4 because for every 4 vertices there are 6 indices
        let index_bucket_size = std::mem::size_of::<i32>() * fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize * 3 / 2;
        let number_of_index_buckets_per_buffer = MAX_BUFFER_SIZE / index_bucket_size;
        let mut number_of_index_pool_buffers = NUM_BUCKETS / number_of_index_buckets_per_buffer;
        if NUM_BUCKETS % number_of_index_buckets_per_buffer != 0 {
            number_of_index_pool_buffers += 1;
        }

        let number_of_buckets_per_buffer = std::cmp::min(number_of_vertex_buckets_per_buffer, number_of_index_buckets_per_buffer);
        let number_of_buffers = std::cmp::max(number_of_vertex_pool_buffers, number_of_index_pool_buffers);

        let mut vertex_pool_buffers = Vec::new();
        let mut index_pool_buffers = Vec::new();
        let mut indirect_pool_buffers = Vec::new();
        let mut vertex_count_pool_buffers = Vec::new();

        for i in 0..number_of_buffers {
            let pool_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Vertex Pool Buffer {i}").as_str()),
                size: (number_of_buckets_per_buffer * vertex_bucket_size) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            let pool_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Index Pool Buffer {i}").as_str()),
                size: (number_of_buckets_per_buffer * index_bucket_size) as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            let pool_indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Indirect Pool Buffer {i}").as_str()),
                size: (number_of_buckets_per_buffer * (std::mem::size_of::<u32>() * 5)) as u64,
                usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            let pool_vertex_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Vertex Count Pool Buffer {i}").as_str()),
                size: (number_of_buckets_per_buffer * (std::mem::size_of::<u32>())) as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            vertex_pool_buffers.push(pool_vertex_buffer);
            index_pool_buffers.push(pool_index_buffer);
            indirect_pool_buffers.push(pool_indirect_buffer);
            vertex_count_pool_buffers.push(pool_vertex_count_buffer);
        }

        let pool_position_to_mesh_bucket_data = HashMap::new();

        Self {
            data_front: VecVertexIndexLengthsTriple::new(),
            data_back: VecVertexIndexLengthsTriple::new(),
            data_left: VecVertexIndexLengthsTriple::new(),
            data_right: VecVertexIndexLengthsTriple::new(),
            data_top: VecVertexIndexLengthsTriple::new(),
            data_bottom: VecVertexIndexLengthsTriple::new(),
            pos_to_gpu_index,
            pos_to_loaded_index: HashMap::new(),
            chunk_index_array,
            chunk_index_buffer,
            chunk_index_bind_group_layout,
            chunk_index_bind_group,
            loaded_chunks: Vec::new(),
            vertex_pool_buffers,
            index_pool_buffers,
            indirect_pool_buffers,
            vertex_count_pool_buffers,
            pool_position_to_mesh_bucket_data,
        }
    }

    pub fn add_mesh_data_drain(&mut self, mesh: Mesh, device: &Device, mesh_position: &WorldPosition) {
        let mesh_argument_arr = [
            (&mut self.data_front, &mesh.front, "front Mesh"),
            (&mut self.data_back, &mesh.back, "back Mesh"),
            (&mut self.data_left, &mesh.left, "left Mesh"),
            (&mut self.data_right, &mesh.right, "right Mesh"),
            (&mut self.data_top, &mesh.top, "top Mesh"),
            (&mut self.data_bottom, &mesh.bottom, "bottom Mesh"),
            ];
        for i in 0..6 {
            mesh_argument_arr[i].0.vertex_buffers.push(Mesh::generate_vertex_buffer(&mesh_argument_arr[i].1.0, device, mesh_argument_arr[i].2));
            mesh_argument_arr[i].0.index_buffers.push(Mesh::generate_index_buffer(&mesh_argument_arr[i].1.1, device, mesh_argument_arr[i].2));
            mesh_argument_arr[i].0.index_lengths.push(mesh_argument_arr[i].1.2);
        }
        self.pos_to_loaded_index.insert(*mesh_position, self.loaded_chunks.len());
        self.loaded_chunks.push(*mesh_position);
    }

    pub fn get_buffers_at_position(&self, pos: &WorldPosition) -> Option<[(&wgpu::Buffer, &wgpu::Buffer, u32); 6]> {
        match self.pos_to_loaded_index.get(pos) {
            Some(index) => self.get_buffers_at_index_i(*index),
            None => None
        }
        
    }

    pub fn get_buffers_at_index_i(&self, i: usize) -> Option<[(&wgpu::Buffer, &wgpu::Buffer, u32); 6]> {
        if i >= self.pos_to_loaded_index.len() {
            None
        } else {
            Some([
                (&self.data_front.vertex_buffers[i],
                &self.data_front.index_buffers[i],
                self.data_front.index_lengths[i]),
                (&self.data_back.vertex_buffers[i],
                &self.data_back.index_buffers[i],
                self.data_back.index_lengths[i]),
                (&self.data_left.vertex_buffers[i],
                &self.data_left.index_buffers[i],
                self.data_left.index_lengths[i]),
                (&self.data_right.vertex_buffers[i],
                &self.data_right.index_buffers[i],
                self.data_right.index_lengths[i]),
                (&self.data_top.vertex_buffers[i],
                &self.data_top.index_buffers[i],
                self.data_top.index_lengths[i]),
                (&self.data_bottom.vertex_buffers[i],
                &self.data_bottom.index_buffers[i],
                self.data_bottom.index_lengths[i]),
            ])
        }
       
    }
}