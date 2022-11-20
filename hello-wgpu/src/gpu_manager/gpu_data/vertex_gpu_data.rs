use std::{collections::HashMap, num::NonZeroUsize, sync::{Arc, RwLock}};
use cgmath::Point3;
use derivables::vertex::Vertex;
use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide};
use lru::LruCache;
use wgpu::{Device, util::DeviceExt, BufferUsages, Queue};

use crate::voxels::mesh::Mesh;

pub const NUM_BUCKETS: usize = (fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize) * fundamentals::consts::NUM_BUCKETS_PER_CHUNK;

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
    pub pool_position_to_mesh_bucket_data: HashMap<WorldPosition, MeshBucketData>,
    pub lru_vertex_buffer_bucket_index: LruCache<BucketPosition, u32>,
    pub lru_index_buffer_bucket_index: LruCache<BucketPosition, u32>,
    pub number_of_buckets_per_buffer: usize,
    pub number_of_buckets_in_last_buffer: usize,
    pub vertex_bucket_size: usize,
    pub index_bucket_size: usize,
    pub frustum_bucket_data_to_update: Vec<(WorldPosition, BlockSide, u32, BucketPosition, i32)>
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

        let buffer_size_fn_return= fundamentals::buffer_size_function::return_bucket_buffer_size_and_amount_information(std::mem::size_of::<Vertex>());

        let mut vertex_pool_buffers = Vec::new();
        let mut index_pool_buffers = Vec::new();
        let mut indirect_pool_buffers = Vec::new();
        let mut index_count_pool_buffers = Vec::new();

        for i in 0..buffer_size_fn_return.number_of_buffers-1 {
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
                let indirect_command = wgpu::util::DrawIndexedIndirect {
                    vertex_count: buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    instance_count: 1,
                    base_index: i * buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    vertex_offset: i as i32 * buffer_size_fn_return.vertex_bucket_size as i32 / std::mem::size_of::<Vertex>() as i32,
                    base_instance: 0
                };

                let indirect_command_arr = [
                    indirect_command.vertex_count, 
                    indirect_command.instance_count, 
                    indirect_command.base_index, 
                    indirect_command.vertex_offset as u32, 
                    indirect_command.base_instance
                    ];

                indirect_commands.push(indirect_command_arr);
            }

            let pool_indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Indirect Pool Buffer {i}").as_str()),
                contents: bytemuck::cast_slice(&indirect_commands),
                usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            });

            let pool_vertex_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Vertex Count Pool Buffer {i}").as_str()),
                size: (buffer_size_fn_return.number_of_buckets_per_buffer * (std::mem::size_of::<u32>())) as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            vertex_pool_buffers.push(pool_vertex_buffer);
            index_pool_buffers.push(pool_index_buffer);
            indirect_pool_buffers.push(pool_indirect_buffer);
            index_count_pool_buffers.push(pool_vertex_count_buffer);
        }
        {
            let last_buffer_num = buffer_size_fn_return.number_of_buffers-1;

            let pool_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Vertex Pool Buffer {last_buffer_num}").as_str()),
                size: (buffer_size_fn_return.number_of_buckets_in_last_buffer * buffer_size_fn_return.vertex_bucket_size) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            let pool_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Index Pool Buffer {last_buffer_num}").as_str()),
                size: (buffer_size_fn_return.number_of_buckets_in_last_buffer * buffer_size_fn_return.index_bucket_size) as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            let mut indirect_commands = Vec::new();

            for i in 0..buffer_size_fn_return.number_of_buckets_in_last_buffer as u32 {
                let indirect_command = wgpu::util::DrawIndexedIndirect {
                    vertex_count: buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    instance_count: 1,
                    base_index: i * buffer_size_fn_return.index_bucket_size as u32 / std::mem::size_of::<i32>() as u32,
                    vertex_offset: i as i32 * buffer_size_fn_return.vertex_bucket_size as i32 / std::mem::size_of::<Vertex>() as i32,
                    base_instance: 0
                };

                let indirect_command_arr = [
                    indirect_command.vertex_count, 
                    indirect_command.instance_count, 
                    indirect_command.base_index, 
                    indirect_command.vertex_offset as u32, 
                    indirect_command.base_instance
                    ];

                indirect_commands.push(indirect_command_arr);
            }

            let pool_indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Indirect Pool Buffer {last_buffer_num}").as_str()),
                contents: bytemuck::cast_slice(&indirect_commands),
                usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            });

            let pool_vertex_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("Vertex Count Pool Buffer {last_buffer_num}").as_str()),
                size: (buffer_size_fn_return.number_of_buckets_per_buffer * (std::mem::size_of::<u32>())) as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false
            });

            vertex_pool_buffers.push(pool_vertex_buffer);
            index_pool_buffers.push(pool_index_buffer);
            indirect_pool_buffers.push(pool_indirect_buffer);
            index_count_pool_buffers.push(pool_vertex_count_buffer);
        }

        let pool_position_to_mesh_bucket_data = HashMap::new();

        let mut lru_vertex_buffer_bucket_index = LruCache::new(NonZeroUsize::new(NUM_BUCKETS).unwrap());
        let mut lru_index_buffer_bucket_index = LruCache::new(NonZeroUsize::new(NUM_BUCKETS).unwrap());
        for i in 0..(buffer_size_fn_return.number_of_buffers-1) as i32 {
            for j in 0..buffer_size_fn_return.number_of_buckets_per_buffer as i32 {
                lru_vertex_buffer_bucket_index.push(BucketPosition { buffer_number: i, bucket_number: j }, 0);
                lru_index_buffer_bucket_index.push(BucketPosition { buffer_number: i, bucket_number: j }, 0);
            }
        }
        for j in 0..buffer_size_fn_return.number_of_buckets_in_last_buffer as i32 {
            lru_vertex_buffer_bucket_index.push(BucketPosition { buffer_number: (buffer_size_fn_return.number_of_buffers-1) as i32, bucket_number: j }, 0);
            lru_index_buffer_bucket_index.push(BucketPosition { buffer_number: (buffer_size_fn_return.number_of_buffers-1) as i32, bucket_number: j }, 0);
        }

        Self {
            pos_to_gpu_index,
            pos_to_loaded_index: HashMap::new(),
            chunk_index_array,
            chunk_index_buffer,
            chunk_index_bind_group_layout,
            chunk_index_bind_group,
            loaded_chunks: Vec::new(),
            vertex_bucket_size: buffer_size_fn_return.vertex_bucket_size,
            index_bucket_size: buffer_size_fn_return.index_bucket_size,
            number_of_buckets_per_buffer: buffer_size_fn_return.number_of_buckets_per_buffer,
            number_of_buckets_in_last_buffer: buffer_size_fn_return.number_of_buckets_in_last_buffer,
            vertex_pool_buffers,
            index_pool_buffers,
            indirect_pool_buffers,
            frustum_bucket_data_to_update: Vec::new(),
            pool_position_to_mesh_bucket_data,
            lru_vertex_buffer_bucket_index,
            lru_index_buffer_bucket_index
        }
    }

    pub fn return_frustum_bucket_data_to_update_and_empty_counts(&mut self) -> Vec<(WorldPosition, BlockSide, u32, BucketPosition, i32)> {
        self.frustum_bucket_data_to_update.drain(..).collect()
    }

    pub fn add_vertex_vec(&mut self, vertex_vec: &Vec<Vertex>, queue: &Arc<RwLock<Queue>>, side: BlockSide,  mesh_position: &WorldPosition) -> Vec<BucketPosition> {
        let vertex_buckets = vertex_vec.chunks(fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize);
        let mut vertex_chunks_len = 0;
        if vertex_vec.len() > 0 {
            vertex_chunks_len = 1 + (vertex_vec.len() - 1) / (fundamentals::consts::NUM_VERTICES_IN_BUCKET as usize);
        }
        let mut lru_buckets = Vec::new();
        let buckets_to_use = match self.pool_position_to_mesh_bucket_data.get(mesh_position) {
            Some(mesh_bucket_data) => {
                match side {
                    BlockSide::FRONT => &mesh_bucket_data.front_bucket_data_vertices,
                    BlockSide::BACK => &mesh_bucket_data.back_bucket_data_vertices,
                    BlockSide::LEFT => &mesh_bucket_data.left_bucket_data_vertices,
                    BlockSide::RIGHT => &mesh_bucket_data.right_bucket_data_vertices,
                    BlockSide::TOP => &mesh_bucket_data.top_bucket_data_vertices,
                    BlockSide::BOTTOM => &mesh_bucket_data.bottom_bucket_data_vertices,
                }
            },
            None => {
                for _ in 0..vertex_chunks_len {
                    let lru_bucket = *self.lru_vertex_buffer_bucket_index.peek_lru().unwrap().0;
                    lru_buckets.push(lru_bucket);
                    self.lru_vertex_buffer_bucket_index.get(&lru_bucket);
                }
                &lru_buckets
            }
        };
        
        //TODO: Remove logic around getting bucket once bucket requesting is implemented - self.lru_vertex_buffer_bucket_index.get(&bucket);
        if vertex_vec.len() == 0 {
            for bucket in buckets_to_use.iter() {
                self.lru_vertex_buffer_bucket_index.get(bucket);
            }
        } else {
            if buckets_to_use.len() < vertex_chunks_len {
                panic!("There are more chunks than vertex buckets to use! Need to implement requesting a bucket.");
            }
            for (i, vertex_bucket) in vertex_buckets.enumerate() {
                let bucket = buckets_to_use[i];
                queue.read().unwrap().write_buffer(&self.vertex_pool_buffers[bucket.buffer_number as usize], (bucket.bucket_number as usize * self.vertex_bucket_size) as u64, bytemuck::cast_slice(vertex_bucket));
                self.lru_vertex_buffer_bucket_index.get(&bucket);
            }
            for i in vertex_chunks_len..buckets_to_use.len() {
                let bucket = buckets_to_use[i];
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
        let buckets_to_use = match self.pool_position_to_mesh_bucket_data.get(mesh_position) {
            Some(mesh_bucket_data) => {
                match side {
                    BlockSide::FRONT => &mesh_bucket_data.front_bucket_data_indices,
                    BlockSide::BACK => &mesh_bucket_data.back_bucket_data_indices,
                    BlockSide::LEFT => &mesh_bucket_data.left_bucket_data_indices,
                    BlockSide::RIGHT => &mesh_bucket_data.right_bucket_data_indices,
                    BlockSide::TOP => &mesh_bucket_data.top_bucket_data_indices,
                    BlockSide::BOTTOM => &mesh_bucket_data.bottom_bucket_data_indices,
                }
            },
            None => {
                for _ in 0..index_chunks_len {
                    let lru_bucket = *self.lru_index_buffer_bucket_index.peek_lru().unwrap().0;
                    lru_buckets.push(lru_bucket);
                    self.lru_index_buffer_bucket_index.get(&lru_bucket);
                }
                &lru_buckets
            }
        };

        if index_vec.len() == 0 {
            for (i, bucket) in buckets_to_use.iter().enumerate() {
                //Basically free up the index bucket (which frees the bucket pair from vertices indirectly)
                self.frustum_bucket_data_to_update.push((*mesh_position, side, i as u32, *bucket, 0));
                self.lru_index_buffer_bucket_index.get(bucket);
            }
        } else {
            if buckets_to_use.len() < index_chunks_len {
                panic!("There are more chunks than index buckets to use! Need to implement requesting a bucket.");
            }
            for (i, index_bucket) in index_buckets.enumerate() {
                let bucket = buckets_to_use[i];
                queue.read().unwrap().write_buffer(&self.index_pool_buffers[bucket.buffer_number as usize], (bucket.bucket_number as usize * self.index_bucket_size) as u64, bytemuck::cast_slice(index_bucket));
                self.frustum_bucket_data_to_update.push((*mesh_position, side, i as u32, bucket, index_bucket.len() as i32));
                self.lru_index_buffer_bucket_index.get(&bucket);
            }
            for i in index_chunks_len..buckets_to_use.len() {
                let bucket = buckets_to_use[i];
                //Basically free up the index bucket (which frees the bucket pair from vertices indirectly)
                self.frustum_bucket_data_to_update.push((*mesh_position, side, i as u32, bucket, 0));
                self.lru_index_buffer_bucket_index.get(&bucket);
            }
        }
        buckets_to_use.to_vec()
    }

    pub fn add_mesh_data_drain(&mut self, mesh: Mesh, mesh_position: &WorldPosition, queue: Arc<RwLock<Queue>>) {
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

    pub fn has_meshed_position(&self, mesh_position: &WorldPosition) -> bool {
        self.pool_position_to_mesh_bucket_data.contains_key(mesh_position)
    }
}