use crate::{voxels::{world::World}, tasks::{TaskResult, Task}};
use fundamentals::world_position::WorldPosition;

pub struct GenerateChunkMeshProcessor {}

impl GenerateChunkMeshProcessor {
    pub fn process_task(chunk_position: &WorldPosition, world: &World, chunks_pos_to_index: &std::collections::HashMap<WorldPosition, u32>) -> TaskResult {
        match world.generate_mesh_at(&chunk_position, *(chunks_pos_to_index.get(chunk_position).unwrap())) {
            Some(mesh) => TaskResult::GenerateChunkMesh { mesh },
            None => {
                println!("Failed to generate mesh at {} due to no chunk existing there", chunk_position);
                TaskResult::Requeue { task: Task::GenerateChunkMesh { chunk_position: *chunk_position } }
            }
        } 
    }
}