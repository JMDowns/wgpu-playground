use crate::{voxels::{world::World, mesh::Mesh, position::Position}, tasks::{TaskResult, Task}};

pub struct GenerateChunkMeshProcessor {}

impl GenerateChunkMeshProcessor {
    pub fn process_task(chunk_position: &Position, world: &World) -> TaskResult {
        match world.generate_mesh_at(&chunk_position) {
            Some(mesh) => TaskResult::GenerateChunkMesh { mesh },
            None => {
                println!("Failed to generate mesh at {} due to no chunk existing there", chunk_position);
                TaskResult::Requeue { task: Task::GenerateChunkMesh { chunk_position: *chunk_position } }
            }
        } 
    }
}