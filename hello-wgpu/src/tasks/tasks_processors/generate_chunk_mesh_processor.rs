use std::sync::{Arc, RwLock};

use crate::{voxels::{mesh::Mesh, chunk::Chunk}, tasks::{TaskResult, Task}, gpu_manager::gpu_data::vertex_gpu_data::VertexGPUData};
use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide, logi};
use instant::Instant;

pub struct GenerateChunkMeshProcessor {}

impl GenerateChunkMeshProcessor {
    pub fn process_task(chunk_position: &WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<wgpu::Queue>>) -> TaskResult {
        let chunk_index = *vertex_gpu_data.read().unwrap().pos_to_gpu_index.get(chunk_position).unwrap() as u32;
        let now = Instant::now();
        let mesh = Mesh::greedy(&chunk.read().unwrap(), chunk_index);
        let after = Instant::now();
        let time = (after-now).as_millis();
        logi!("Greedy mesh took {} milliseconds", time);
        vertex_gpu_data.write().unwrap().add_mesh_data_drain(mesh, chunk_position, queue);
        TaskResult::GenerateChunkMesh {  }
    }
}

pub struct GenerateChunkSideMeshProcessor {}

impl GenerateChunkSideMeshProcessor {
    pub fn process_task(chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<wgpu::Queue>>, side: BlockSide) -> TaskResult {
        if vertex_gpu_data.read().unwrap().has_meshed_position(&chunk_position) {
            let chunk_index = *vertex_gpu_data.read().unwrap().pos_to_gpu_index.get(&chunk_position).unwrap() as u32;
            let (vertex_vec, index_vec, index_count) = Mesh::cull_side(&chunk.read().unwrap(), chunk_index, side);
            vertex_gpu_data.write().unwrap().update_side_mesh_data_drain(vertex_vec, index_vec, index_count, &chunk_position, queue, side);

            TaskResult::UpdateChunkSideMesh {  }
        } else {
            TaskResult::Requeue { task: Task::GenerateChunkSideMesh { chunk_position, chunk, vertex_gpu_data, queue, side } }
        }
        

        
    }
}