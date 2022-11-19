use std::sync::{Arc, RwLock};

use crate::{voxels::{mesh::Mesh, chunk::Chunk}, tasks::{TaskResult, Task}, gpu_manager::gpu_data::vertex_gpu_data::VertexGPUData};
use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide, logi};
use instant::Instant;

pub struct GenerateChunkMeshProcessor {}

impl GenerateChunkMeshProcessor {
    pub fn process_task(chunk_position: &WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<wgpu::Queue>>) -> TaskResult {
        let chunk_index = *vertex_gpu_data.read().unwrap().pos_to_gpu_index.get(chunk_position).unwrap() as u32;
        
        let mesh = Mesh::greedy(&chunk.read().unwrap(), chunk_index);
        
        vertex_gpu_data.write().unwrap().add_mesh_data_drain(mesh, chunk_position, queue);
        TaskResult::GenerateChunkMesh {  }
    }
}

pub struct GenerateChunkSideMeshesProcessor {}

impl GenerateChunkSideMeshesProcessor {
    pub fn process_task(chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<wgpu::Queue>>, sides: Vec<BlockSide>) -> TaskResult {
        if vertex_gpu_data.read().unwrap().has_meshed_position(&chunk_position) {
            let chunk_index = *vertex_gpu_data.read().unwrap().pos_to_gpu_index.get(&chunk_position).unwrap() as u32;
            let mesh = Mesh::greedy_sided(&chunk.read().unwrap(), chunk_index, &sides);
            vertex_gpu_data.write().unwrap().update_side_mesh_data_drain(mesh, &chunk_position, queue, &sides);

            TaskResult::UpdateChunkSideMesh {  }
        } else {
            TaskResult::Requeue { task: Task::GenerateChunkSideMeshes { chunk_position, chunk, vertex_gpu_data, queue, sides } }
        }
        

        
    }
}