use crate::tasks::TaskResult;
use crate::{voxels::world::World, voxels::position::Position};
use crate::voxels::chunk::Chunk;

pub struct GenerateChunkProcessor {}

impl GenerateChunkProcessor {
    pub fn process_task(chunk_position: &Position) -> TaskResult {
        TaskResult::GenerateChunk { chunk: World::generate_chunk_at(&chunk_position) }
    }
}