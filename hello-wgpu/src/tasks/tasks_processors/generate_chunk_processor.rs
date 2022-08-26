use std::sync::{Arc, RwLock};

use crate::tasks::TaskResult;
use crate::{voxels::world::World};
use fundamentals::world_position::WorldPosition;

pub struct GenerateChunkProcessor {}

impl GenerateChunkProcessor {
    pub fn process_task(chunk_position: &WorldPosition, world: Arc<RwLock<World>>) -> TaskResult {
        world.write().unwrap().add_chunk(World::generate_chunk_at(&chunk_position));
        TaskResult::GenerateChunk { chunk_position: *chunk_position }
    }
}