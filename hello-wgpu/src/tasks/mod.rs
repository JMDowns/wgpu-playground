use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

use crate::gpu_manager::gpu_data::vertex_gpu_data::VertexGPUData;
use crate::voxels::world::World;
use fundamentals::world_position::WorldPosition;
use wgpu::Queue;

pub mod tasks_processors;

pub enum Task {
    StopThread,
    GenerateChunkMesh { chunk_position: WorldPosition, world: Arc<RwLock<World>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<Queue>> },
    GenerateChunk { chunk_position: WorldPosition, world: Arc<RwLock<World>>},
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Task::StopThread => {
                match other {
                    Task::StopThread => true,
                    _ => false
                }
            },

            Task::GenerateChunkMesh { chunk_position, .. } => {
                let self_chunk_pos = chunk_position;
                match other {
                    Task::GenerateChunkMesh { chunk_position, .. } => {
                        *self_chunk_pos == *chunk_position
                    }
                    _ => false
                }
            }

            Task::GenerateChunk { chunk_position, ..} => {
                let self_chunk_pos = chunk_position;
                match other {
                    Task::GenerateChunk { chunk_position, .. } => {
                        *self_chunk_pos == *chunk_position
                    }
                    _ => false
                }
            }
        }
    }
}
impl Eq for Task {}

impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Task::GenerateChunk { chunk_position, .. } => chunk_position.hash(state),
            Task::GenerateChunkMesh { chunk_position, .. } => {
                chunk_position.hash(state);
            }
            Task::StopThread => {}
        }
    }
}

pub fn get_task_priority(task: &Task) -> u32 {
    match task {
        Task::StopThread => 0,
        Task::GenerateChunk { .. } => 1,
        Task::GenerateChunkMesh { .. } => 2
    }
}

pub enum TaskResult {
    Requeue { task: Task },
    GenerateChunkMesh { },
    GenerateChunk { chunk_position: WorldPosition }
}