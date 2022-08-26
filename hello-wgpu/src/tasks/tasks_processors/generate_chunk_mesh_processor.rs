use std::sync::{Arc, RwLock};

use crate::{voxels::{world::World}, tasks::{TaskResult, Task}, gpu_data::vertex_gpu_data::VertexGPUData};
use fundamentals::world_position::WorldPosition;

pub struct GenerateChunkMeshProcessor {}

impl GenerateChunkMeshProcessor {
    pub fn process_task(chunk_position: &WorldPosition, world: Arc<RwLock<World>>, chunk_index: u32, vertex_gpu_data: Arc<RwLock<VertexGPUData>>) -> TaskResult {
        let mesh_option = world.read().unwrap().generate_mesh_at(&chunk_position, chunk_index);
        match mesh_option {
            Some(mesh) => {
                vertex_gpu_data.write().unwrap().add_gpu_data_drain(&mut mesh.get_gpu_data());
                TaskResult::GenerateChunkMesh { }
            },
            None => {
                println!("Failed to generate mesh at {} due to no chunk existing there", chunk_position);
                TaskResult::Requeue { task: Task::GenerateChunkMesh { chunk_position: *chunk_position, world: world.clone(), chunk_index, vertex_gpu_data: vertex_gpu_data.clone() } }
            }
        } 
    }
}