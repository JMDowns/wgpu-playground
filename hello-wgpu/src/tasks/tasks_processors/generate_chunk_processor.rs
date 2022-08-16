use crate::tasks::TaskResult;
use crate::{voxels::world::World};
use fundamentals::world_position::WorldPosition;

pub struct GenerateChunkProcessor {}

impl GenerateChunkProcessor {
    pub fn process_task(chunk_position: &WorldPosition) -> TaskResult {
        TaskResult::GenerateChunk { chunk: World::generate_chunk_at(&chunk_position) }
    }
}