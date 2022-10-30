use std::sync::{Arc, RwLock};

use crate::{voxels::{mesh::Mesh, chunk::Chunk}, tasks::TaskResult, gpu_manager::gpu_data::vertex_gpu_data::VertexGPUData};
use fundamentals::world_position::WorldPosition;

pub struct GenerateChunkMeshProcessor {}

impl GenerateChunkMeshProcessor {
    pub fn process_task(chunk_position: &WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<wgpu::Queue>>) -> TaskResult {
        let chunk_index = *vertex_gpu_data.read().unwrap().pos_to_gpu_index.get(chunk_position).unwrap() as u32;
        let mesh = Mesh::cull_ambient_occlusion(&chunk.read().unwrap(), chunk_index);
        vertex_gpu_data.write().unwrap().add_mesh_data_drain(mesh, chunk_position, queue);
        TaskResult::GenerateChunkMesh {  }
    }
}