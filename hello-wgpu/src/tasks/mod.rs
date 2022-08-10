use crate::voxels::{position::Position, mesh::Mesh, chunk::Chunk};

pub mod tasks_processors;

pub enum Task {
    GenerateChunkMesh { chunk_position: Position },
    GenerateChunk { chunk_position: Position },
}

pub enum TaskResult {
    Requeue { task: Task },
    GenerateChunkMesh { mesh: Mesh },
    GenerateChunk { chunk: Chunk },
}