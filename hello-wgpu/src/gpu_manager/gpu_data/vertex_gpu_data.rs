use std::{collections::HashMap, hash::Hash, num::NonZeroUsize, sync::{Arc, RwLock}};
use cgmath::{Point3, Vector3, Deg};
use derivables::{subvoxel_vertex::{generate_cube_at_center, SubvoxelVertex}, vertex::Vertex};
use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide, consts};
use lru::LruCache;
use wgpu::{Device, util::DeviceExt, BufferUsages, Queue};

use crate::{gpu_manager::chunk_index_state::{self, ChunkIndexState}, voxels::{chunk, mesh::Mesh}};

pub const NUM_BUCKETS: usize = (fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize) * fundamentals::consts::NUM_BUCKETS_PER_CHUNK;

#[derive(PartialEq, Eq)]
pub struct MemoryInfo {
    pub buckets_total: usize
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct BucketPosition {
    pub buffer_number: i32,
    pub bucket_number: i32
}

#[derive(Debug)]
pub struct MeshBucketData {
    pub front_bucket_data_vertices: Vec<BucketPosition>,
    pub front_bucket_data_indices: Vec<BucketPosition>,
    pub back_bucket_data_vertices: Vec<BucketPosition>,
    pub back_bucket_data_indices: Vec<BucketPosition>,
    pub left_bucket_data_vertices: Vec<BucketPosition>,
    pub left_bucket_data_indices: Vec<BucketPosition>,
    pub right_bucket_data_vertices: Vec<BucketPosition>,
    pub right_bucket_data_indices: Vec<BucketPosition>,
    pub top_bucket_data_vertices: Vec<BucketPosition>,
    pub top_bucket_data_indices: Vec<BucketPosition>,
    pub bottom_bucket_data_vertices: Vec<BucketPosition>,
    pub bottom_bucket_data_indices: Vec<BucketPosition>,
}

pub struct VertexGPUData {
    pub vertex_pool_buffers: Vec<wgpu::Buffer>,
    pub index_pool_buffers: Vec<wgpu::Buffer>,
    pub indirect_pool_buffers: Vec<wgpu::Buffer>,
    pub visibility_buffer: wgpu::Buffer,
    pub visibility_bind_group_layout: wgpu::BindGroupLayout,
    pub visibility_bind_group: wgpu::BindGroup,
    pub occlusion_cube_vertex_buffer: wgpu::Buffer,
    pub occlusion_cube_index_buffer: wgpu::Buffer,
    pub pool_position_to_mesh_bucket_data: HashMap<WorldPosition, MeshBucketData>,
    pub lru_vertex_buffer_bucket_index: LruCache<BucketPosition, u32>,
    pub lru_index_buffer_bucket_index: LruCache<BucketPosition, u32>,
    pub number_of_buckets_per_buffer: usize,
    pub vertex_bucket_size: usize,
    pub index_bucket_size: usize,
    pub frustum_bucket_data_to_update: Vec<(WorldPosition, BlockSide, u32, BucketPosition)>,
    pub frustum_bucket_data_to_clear: Vec<(WorldPosition, BlockSide, u32)>,
    pub vertex_buckets_used: usize,
    pub vertex_buckets_total: usize,
    pub chunk_index_state: Arc<RwLock<ChunkIndexState>>
}

impl VertexGPUData {
    pub fn new(device: &Device, chunk_index_state: Arc<RwLock<ChunkIndexState>>) -> Self {
        let buffer_size_fn_return= fundamentals::buffer_size_function::return_bucket_buffer_size_and_amount_information(std::mem::size_of::<Vertex>());

        let mut vertex_pool_buffers = Vec::new();
        let mut index_pool_buffers = Vec::new();
        let mut indirect_pool_buffers = Vec::new();

        for i in 0..buffer_size_fn_return.num_initial_buffers {
            let pool_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Vertex Pool Buffer {i}").as_str()),
                size: (buffer_size_fn_return.number_of_buckets_per_buffer * buffer_size_fn_return.vertex_bucket_size) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            let pool_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Index Pool Buffer {i}").as_str()),
                size: (buffer_size_fn_return.number_of_buckets_per_buffer * buffer_size_fn_return.index_bucket_size) as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            let mut indirect_commands = Vec::new();

            for i in 0..buffer_size_fn_return.number_of_buckets_per_buffer as u32 {
                let indirect_command = wgpu::util::DrawIndexedIndirectArgs {
                    index_count: buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    instance_count: 0,
                    first_index: i * buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    base_vertex: i as i32 * buffer_size_fn_return.vertex_bucket_size as i32 / std::mem::size_of::<Vertex>() as i32,
                    first_instance: 0
                };

                let indirect_command_arr = [
                    indirect_command.index_count, 
                    indirect_command.instance_count, 
                    indirect_command.first_index, 
                    indirect_command.base_vertex as u32, 
                    indirect_command.first_instance
                    ];

                indirect_commands.push(indirect_command_arr);
            }

            let pool_indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Indirect Pool Buffer {i}").as_str()),
                contents: bytemuck::cast_slice(&indirect_commands),
                usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            });

            vertex_pool_buffers.push(pool_vertex_buffer);
            index_pool_buffers.push(pool_index_buffer);
            indirect_pool_buffers.push(pool_indirect_buffer);
        }

        for i in buffer_size_fn_return.num_initial_buffers..buffer_size_fn_return.num_max_buffers {
            let mut indirect_commands = Vec::new();

            for i in 0..buffer_size_fn_return.number_of_buckets_per_buffer as u32 {
                let indirect_command = wgpu::util::DrawIndexedIndirectArgs {
                    index_count: buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    instance_count: 0,
                    first_index: i * buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    base_vertex: i as i32 * buffer_size_fn_return.vertex_bucket_size as i32 / std::mem::size_of::<Vertex>() as i32,
                    first_instance: 0
                };

                let indirect_command_arr = [
                    indirect_command.index_count, 
                    indirect_command.instance_count, 
                    indirect_command.first_index, 
                    indirect_command.base_vertex as u32, 
                    indirect_command.first_instance
                    ];

                indirect_commands.push(indirect_command_arr);
            }

            let pool_indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Indirect Pool Buffer {i}").as_str()),
                contents: bytemuck::cast_slice(&indirect_commands),
                usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            });
            indirect_pool_buffers.push(pool_indirect_buffer);
        }

        let pool_position_to_mesh_bucket_data = HashMap::new();

        let mut vertex_buckets_total = 0;

        let mut lru_vertex_buffer_bucket_index = LruCache::new(NonZeroUsize::new(NUM_BUCKETS).unwrap());
        let mut lru_index_buffer_bucket_index = LruCache::new(NonZeroUsize::new(NUM_BUCKETS).unwrap());
        for i in 0..(buffer_size_fn_return.num_initial_buffers) as i32 {
            for j in 0..buffer_size_fn_return.number_of_buckets_per_buffer as i32 {
                lru_vertex_buffer_bucket_index.push(BucketPosition { buffer_number: i, bucket_number: j }, 0);
                lru_index_buffer_bucket_index.push(BucketPosition { buffer_number: i, bucket_number: j }, 0);
                vertex_buckets_total += 1;
            }
        }

        let occlusion_cube_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Occlusion Cube Vertex Buffer").as_str()),
            size: (consts::NUMBER_OF_CHUNKS_AROUND_PLAYER * 24 * std::mem::size_of::<Vertex>() as u32) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let occlusion_cube_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Occlusion Cube Index Buffer").as_str()),
            size: (consts::NUMBER_OF_CHUNKS_AROUND_PLAYER * 36 * std::mem::size_of::<u32>() as u32) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let visibility_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Chunk Visibility Buffer").as_str()),
            size: (consts::NUMBER_OF_CHUNKS_AROUND_PLAYER * std::mem::size_of::<u32>() as u32) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let visibility_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None
                }
            ],
            label: Some("visibility_bind_group_layout")
        });

        let visibility_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &visibility_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: visibility_buffer.as_entire_binding()
                }
            ],
            label: Some("visibility_bind_group")
        });

        Self {
            vertex_bucket_size: buffer_size_fn_return.vertex_bucket_size,
            index_bucket_size: buffer_size_fn_return.index_bucket_size,
            number_of_buckets_per_buffer: buffer_size_fn_return.number_of_buckets_per_buffer,
            vertex_pool_buffers,
            index_pool_buffers,
            indirect_pool_buffers,
            visibility_buffer,
            visibility_bind_group_layout,
            visibility_bind_group,
            frustum_bucket_data_to_update: Vec::new(),
            frustum_bucket_data_to_clear: Vec::new(),
            pool_position_to_mesh_bucket_data,
            lru_vertex_buffer_bucket_index,
            lru_index_buffer_bucket_index,
            vertex_buckets_used: 0,
            vertex_buckets_total,
            occlusion_cube_vertex_buffer,
            occlusion_cube_index_buffer,
            chunk_index_state
        }
    }

    pub fn return_frustum_bucket_data_to_update_and_empty_counts(&mut self) -> Vec<(WorldPosition, BlockSide, u32, BucketPosition)> {
        self.frustum_bucket_data_to_update.drain(..).collect()
    }

    pub fn return_frustum_bucket_data_to_clear_and_empty_counts(&mut self) -> Vec<(WorldPosition, BlockSide, u32)> {
        self.frustum_bucket_data_to_clear.drain(..).collect()
    }

    pub fn add_vertex_vec(&mut self, vertex_vec: &Vec<Vertex>, queue: &Arc<RwLock<Queue>>, side: BlockSide,  mesh_position: &WorldPosition) -> Vec<BucketPosition> {
        let vertex_buckets = vertex_vec.chunks(fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize);
        let mut vertex_chunks_len = 0;
        if vertex_vec.len() > 0 {
            vertex_chunks_len = 1 + (vertex_vec.len() - 1) / (fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize);
        }
        let mut lru_buckets = Vec::new();
        let buckets_to_use = match self.pool_position_to_mesh_bucket_data.get_mut(mesh_position) {
            Some(mesh_bucket_data) => {
                match side {
                    BlockSide::FRONT => &mut mesh_bucket_data.front_bucket_data_vertices,
                    BlockSide::BACK => &mut mesh_bucket_data.back_bucket_data_vertices,
                    BlockSide::LEFT => &mut mesh_bucket_data.left_bucket_data_vertices,
                    BlockSide::RIGHT => &mut mesh_bucket_data.right_bucket_data_vertices,
                    BlockSide::TOP => &mut mesh_bucket_data.top_bucket_data_vertices,
                    BlockSide::BOTTOM => &mut mesh_bucket_data.bottom_bucket_data_vertices,
                }
            },
            None => {
                for _ in 0..vertex_chunks_len {
                    let lru_bucket = *self.lru_vertex_buffer_bucket_index.peek_lru().unwrap().0;
                    lru_buckets.push(lru_bucket);
                    self.lru_vertex_buffer_bucket_index.get(&lru_bucket);
                }
                self.vertex_buckets_used += vertex_chunks_len;
                &mut lru_buckets
            }
        };
        
        if vertex_vec.len() != 0 {
            for _ in buckets_to_use.len()..vertex_chunks_len {
                let lru_bucket = *self.lru_vertex_buffer_bucket_index.peek_lru().unwrap().0;
                buckets_to_use.push(lru_bucket);
                self.lru_vertex_buffer_bucket_index.get(&lru_bucket);
                self.vertex_buckets_used += 1;
            }
            for (i, vertex_bucket) in vertex_buckets.enumerate() {
                let bucket = buckets_to_use[i];
                queue.read().unwrap().write_buffer(&self.vertex_pool_buffers[bucket.buffer_number as usize], (bucket.bucket_number as usize * self.vertex_bucket_size) as u64, bytemuck::cast_slice(vertex_bucket));
                self.lru_vertex_buffer_bucket_index.get(&bucket);
            }
        }

        buckets_to_use.to_vec()
    }

    pub fn add_index_vec_and_update_index_count_vec(&mut self, index_vec: &Vec<u32>, queue: &Arc<RwLock<Queue>>, side: BlockSide, mesh_position: &WorldPosition) -> Vec<BucketPosition> {
        let index_buckets = index_vec.chunks(fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize * 3 / 2);
        let mut index_chunks_len = 0;
        if index_vec.len() > 0 {
            index_chunks_len = 1 + (index_vec.len() - 1) / (fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize * 3 / 2);
        }
        let mut lru_buckets = Vec::new();
        let buckets_to_use = match self.pool_position_to_mesh_bucket_data.get_mut(mesh_position) {
            Some(mesh_bucket_data) => {
                match side {
                    BlockSide::FRONT => &mut mesh_bucket_data.front_bucket_data_indices,
                    BlockSide::BACK => &mut mesh_bucket_data.back_bucket_data_indices,
                    BlockSide::LEFT => &mut mesh_bucket_data.left_bucket_data_indices,
                    BlockSide::RIGHT => &mut mesh_bucket_data.right_bucket_data_indices,
                    BlockSide::TOP => &mut mesh_bucket_data.top_bucket_data_indices,
                    BlockSide::BOTTOM => &mut mesh_bucket_data.bottom_bucket_data_indices,
                }
            },
            None => {
                for _ in 0..index_chunks_len {
                    let lru_bucket = *self.lru_index_buffer_bucket_index.peek_lru().unwrap().0;
                    lru_buckets.push(lru_bucket);
                    self.lru_index_buffer_bucket_index.get(&lru_bucket);
                }
                &mut lru_buckets
            }
        };

        if index_vec.len() == 0 {
            for (i, bucket_position) in buckets_to_use.iter().enumerate() {
                self.frustum_bucket_data_to_clear.push((*mesh_position, side, i as u32));
                Self::update_indirect_index_count(&self.indirect_pool_buffers, bucket_position.buffer_number as usize, bucket_position.bucket_number as usize, 0, queue);
            }
        } else {
            for _ in buckets_to_use.len()..index_chunks_len {
                let lru_bucket = *self.lru_index_buffer_bucket_index.peek_lru().unwrap().0;
                buckets_to_use.push(lru_bucket);
                self.lru_index_buffer_bucket_index.get(&lru_bucket);
            }
            for (i, index_bucket) in index_buckets.enumerate() {
                let bucket_position = buckets_to_use[i];
                queue.read().unwrap().write_buffer(&self.index_pool_buffers[bucket_position.buffer_number as usize], (bucket_position.bucket_number as usize * self.index_bucket_size) as u64, bytemuck::cast_slice(index_bucket));
                Self::update_indirect_index_count(&self.indirect_pool_buffers, bucket_position.buffer_number as usize, bucket_position.bucket_number as usize, index_bucket.len(), queue);
                self.frustum_bucket_data_to_update.push((*mesh_position, side, i as u32, bucket_position));
                self.lru_index_buffer_bucket_index.get(&bucket_position);
            }
            for i in index_chunks_len..buckets_to_use.len() {
                let bucket_position = &buckets_to_use[i];
                self.frustum_bucket_data_to_clear.push((*mesh_position, side, i as u32));
                Self::update_indirect_index_count(&self.indirect_pool_buffers, bucket_position.buffer_number as usize, bucket_position.bucket_number as usize, 0, queue);
            }
        }
        buckets_to_use.to_vec()
    }

    fn update_indirect_index_count(indirect_pool_buffers: &Vec<wgpu::Buffer>, buffer_number: usize, bucket_number: usize, index_count: usize, queue: &Arc<RwLock<Queue>>) {
        queue.read().unwrap().write_buffer(&indirect_pool_buffers[buffer_number as usize], (bucket_number as usize * std::mem::size_of::<i32>()*5) as u64, bytemuck::cast_slice(&[index_count]))
    }   

    pub fn add_mesh_data_drain(&mut self, mesh: Mesh, occlusion_cube_mesh: Mesh, mesh_position: &WorldPosition, queue: Arc<RwLock<Queue>>) {
        let front_bucket_data_vertices = self.add_vertex_vec(&mesh.front.0, &queue, BlockSide::FRONT, mesh_position);
        let front_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.front.1, &queue, BlockSide::FRONT, mesh_position);
        let back_bucket_data_vertices = self.add_vertex_vec(&mesh.back.0, &queue, BlockSide::BACK, mesh_position);
        let back_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.back.1, &queue, BlockSide::BACK, mesh_position);
        let left_bucket_data_vertices = self.add_vertex_vec(&mesh.left.0, &queue, BlockSide::LEFT, mesh_position);
        let left_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.left.1, &queue, BlockSide::LEFT, mesh_position);
        let right_bucket_data_vertices = self.add_vertex_vec(&mesh.right.0, &queue, BlockSide::RIGHT, mesh_position);
        let right_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.right.1, &queue, BlockSide::RIGHT, mesh_position);
        let top_bucket_data_vertices = self.add_vertex_vec(&mesh.top.0, &queue, BlockSide::TOP, mesh_position);
        let top_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.top.1, &queue, BlockSide::TOP, mesh_position);
        let bottom_bucket_data_vertices = self.add_vertex_vec(&mesh.bottom.0, &queue, BlockSide::BOTTOM, mesh_position);
        let bottom_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.bottom.1, &queue, BlockSide::BOTTOM, mesh_position);
        self.pool_position_to_mesh_bucket_data.insert(*mesh_position, MeshBucketData { front_bucket_data_vertices, front_bucket_data_indices, back_bucket_data_vertices, back_bucket_data_indices, left_bucket_data_vertices, left_bucket_data_indices, right_bucket_data_vertices, right_bucket_data_indices, top_bucket_data_vertices, top_bucket_data_indices, bottom_bucket_data_vertices, bottom_bucket_data_indices });

        let gpu_index = *self.chunk_index_state.read().unwrap().pos_to_gpu_index.get(mesh_position).unwrap();
        let occlusion_vertices = [
            occlusion_cube_mesh.front.0,
            occlusion_cube_mesh.back.0,
            occlusion_cube_mesh.left.0,
            occlusion_cube_mesh.right.0,
            occlusion_cube_mesh.top.0,
            occlusion_cube_mesh.bottom.0,
        ].concat();
        let occlusion_indices = [
            occlusion_cube_mesh.front.1.iter().map(|v| *v + 24*(gpu_index as u32)).collect::<Vec<u32>>(),
            occlusion_cube_mesh.back.1.iter().map(|v| *v + 24*(gpu_index as u32)).collect(),
            occlusion_cube_mesh.left.1.iter().map(|v| *v + 24*(gpu_index as u32)).collect(),
            occlusion_cube_mesh.right.1.iter().map(|v| *v + 24*(gpu_index as u32)).collect(),
            occlusion_cube_mesh.top.1.iter().map(|v| *v + 24*(gpu_index as u32)).collect(),
            occlusion_cube_mesh.bottom.1.iter().map(|v| *v + 24*(gpu_index as u32)).collect(),
        ].concat();
        queue.read().unwrap().write_buffer(&self.occlusion_cube_vertex_buffer, (gpu_index * std::mem::size_of::<Vertex>()*24 as usize) as u64, bytemuck::cast_slice(occlusion_vertices.as_slice()));
        queue.read().unwrap().write_buffer(&self.occlusion_cube_index_buffer, (gpu_index * std::mem::size_of::<u32>()*36 as usize) as u64, bytemuck::cast_slice(occlusion_indices.as_slice()));
    }

    pub fn enough_memory_for_mesh(&self, mesh: &Mesh, mesh_position: &WorldPosition) -> bool {
        let number_of_vertices = mesh.front.0.len() + mesh.back.0.len() + mesh.left.0.len() + mesh.right.0.len() + mesh.top.0.len() + mesh.bottom.0.len();
        if (number_of_vertices == 0) {
            return true;
        }
        let buckets_needed = (1 + (number_of_vertices - 1) / (fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize)) as i32;

        let mut buckets_used = 0;

        match self.pool_position_to_mesh_bucket_data.get(mesh_position) {
            Some(mesh) => {
                buckets_used = (mesh.front_bucket_data_vertices.len() + mesh.back_bucket_data_vertices.len() + mesh.left_bucket_data_vertices.len() + mesh.right_bucket_data_vertices.len() + mesh.top_bucket_data_vertices.len() + mesh.bottom_bucket_data_vertices.len()) as i32;
            }

            None => {}
        }
        
        let enough_memory = ((self.vertex_buckets_used as i32 + buckets_needed - buckets_used) as usize) < self.vertex_buckets_total;

        if !enough_memory {
            let vertex_buckets_used = self.vertex_buckets_used;
            let new_buckets_needed = buckets_needed - buckets_used;
        }

        enough_memory
    }

    pub fn update_side_mesh_data_drain(&mut self, mesh: Mesh, mesh_position: &WorldPosition, queue: Arc<RwLock<Queue>>, sides: &Vec<BlockSide>) {
        for side in sides {
            match side {
                BlockSide::FRONT => {
                    let front_bucket_data_vertices = self.add_vertex_vec(&mesh.front.0, &queue, BlockSide::FRONT, mesh_position);
                    let front_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.front.1, &queue, BlockSide::FRONT, mesh_position);
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().front_bucket_data_vertices = front_bucket_data_vertices;
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().front_bucket_data_indices = front_bucket_data_indices;
                },
                BlockSide::BACK => {
                    let back_bucket_data_vertices = self.add_vertex_vec(&mesh.back.0, &queue, BlockSide::BACK, mesh_position);
                    let back_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.back.1, &queue, BlockSide::BACK, mesh_position);
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().back_bucket_data_vertices = back_bucket_data_vertices;
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().back_bucket_data_indices = back_bucket_data_indices;
                },
                BlockSide::LEFT => {
                    let left_bucket_data_vertices = self.add_vertex_vec(&mesh.left.0, &queue, BlockSide::LEFT, mesh_position);
                    let left_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.left.1, &queue, BlockSide::LEFT, mesh_position);
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().left_bucket_data_vertices = left_bucket_data_vertices;
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().left_bucket_data_indices = left_bucket_data_indices;
                },
                BlockSide::RIGHT => {
                    let right_bucket_data_vertices = self.add_vertex_vec(&mesh.right.0, &queue, BlockSide::RIGHT, mesh_position);
                    let right_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.right.1, &queue, BlockSide::RIGHT, mesh_position);
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().right_bucket_data_vertices = right_bucket_data_vertices;
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().right_bucket_data_indices = right_bucket_data_indices;
                },
                BlockSide::TOP => {
                    let top_bucket_data_vertices = self.add_vertex_vec(&mesh.top.0, &queue, BlockSide::TOP, mesh_position);
                    let top_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.top.1, &queue, BlockSide::TOP, mesh_position);
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().top_bucket_data_vertices = top_bucket_data_vertices;
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().top_bucket_data_indices = top_bucket_data_indices;
                },
                BlockSide::BOTTOM => {
                    let bottom_bucket_data_vertices = self.add_vertex_vec(&mesh.bottom.0, &queue, BlockSide::BOTTOM, mesh_position);
                    let bottom_bucket_data_indices = self.add_index_vec_and_update_index_count_vec(&mesh.bottom.1, &queue, BlockSide::BOTTOM, mesh_position);
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().bottom_bucket_data_vertices = bottom_bucket_data_vertices;
                    self.pool_position_to_mesh_bucket_data.get_mut(mesh_position).unwrap().bottom_bucket_data_indices = bottom_bucket_data_indices;
                }
            }
        }
    }

    pub fn should_allocate_new_buffer(&self) -> bool {
        self.vertex_buckets_used > (self.vertex_buckets_total * 3 / 4)
    }

    pub fn allocate_new_buffer(&mut self, device: Arc<RwLock<Device>>) {
        let buffer_size_fn_return= fundamentals::buffer_size_function::return_bucket_buffer_size_and_amount_information(std::mem::size_of::<Vertex>());

        let buffer_num = self.vertex_pool_buffers.len() + 1;

        if buffer_num > buffer_size_fn_return.num_max_buffers {
            panic!("Ran out of memory!");
        }

        let device = device.read().unwrap();

        let pool_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Vertex Pool Buffer {buffer_num}").as_str()),
            size: (buffer_size_fn_return.number_of_buckets_per_buffer * buffer_size_fn_return.vertex_bucket_size) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let pool_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Index Pool Buffer {buffer_num}").as_str()),
            size: (buffer_size_fn_return.number_of_buckets_per_buffer * buffer_size_fn_return.index_bucket_size) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        for i in 0..buffer_size_fn_return.number_of_buckets_per_buffer as i32 {
            let bucket_position = BucketPosition { buffer_number: self.vertex_pool_buffers.len() as i32, bucket_number: i };
            self.lru_vertex_buffer_bucket_index.push(bucket_position, 0);
            self.lru_vertex_buffer_bucket_index.demote(&bucket_position);
            self.lru_index_buffer_bucket_index.push(bucket_position, 0);
            self.lru_index_buffer_bucket_index.demote(&bucket_position);
            self.vertex_buckets_total += 1;
        }

        self.vertex_pool_buffers.push(pool_vertex_buffer);
        self.index_pool_buffers.push(pool_index_buffer);
    }

    pub fn has_meshed_position(&self, mesh_position: &WorldPosition) -> bool {
        self.pool_position_to_mesh_bucket_data.contains_key(mesh_position)
    }

    pub fn get_memory_info(&self) -> MemoryInfo {
        MemoryInfo { buckets_total: self.vertex_buckets_total }
    }
}