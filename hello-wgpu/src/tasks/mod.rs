use crate::voxels::{mesh::Mesh, chunk::Chunk};
use fundamentals::world_position::WorldPosition;

pub mod tasks_processors;

pub enum Task {
    GenerateChunkMesh { chunk_position: WorldPosition },
    GenerateChunk { chunk_position: WorldPosition },
}

pub enum TaskResult {
    Requeue { task: Task },
    GenerateChunkMesh { mesh: Mesh },
    GenerateChunk { chunk: Chunk },
}