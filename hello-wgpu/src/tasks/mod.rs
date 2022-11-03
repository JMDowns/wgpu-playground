use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

use crate::gpu_manager::gpu_data::vertex_gpu_data::VertexGPUData;
use crate::voxels::chunk::Chunk;
use crate::voxels::world::World;
use fundamentals::enums::block_side::BlockSide;
use fundamentals::world_position::WorldPosition;
use wgpu::Queue;

pub mod tasks_processors;

pub enum Task {
    StopThread,
    GenerateChunkMesh { chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<Queue>> },
    GenerateChunk { chunk_position: WorldPosition, world: Arc<RwLock<World>>},
    UpdateYAxisChunkPadding { chunk_below: Arc<RwLock<Chunk>>, chunk_above: Arc<RwLock<Chunk>>, additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo},
    UpdateXAxisChunkPadding { chunk_front: Arc<RwLock<Chunk>>, chunk_back: Arc<RwLock<Chunk>>, additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo},
    UpdateZAxisChunkPadding { chunk_left: Arc<RwLock<Chunk>>, chunk_right: Arc<RwLock<Chunk>>, additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo},
    GenerateChunkSideMesh { chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<Queue>>, side: BlockSide }
}

pub struct ChunkUpdateTaskIdentifyingInfo {
    pub chunk_position_1: WorldPosition,
    pub chunk_position_2: WorldPosition
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

            Task::UpdateYAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                let self_positions = additional_data_to_identify_and_hash;
                match other {
                    Task::UpdateYAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                        (self_positions.chunk_position_1 == additional_data_to_identify_and_hash.chunk_position_1 &&
                        self_positions.chunk_position_2 == additional_data_to_identify_and_hash.chunk_position_2) ||
                        (self_positions.chunk_position_2 == additional_data_to_identify_and_hash.chunk_position_1 &&
                            self_positions.chunk_position_1 == additional_data_to_identify_and_hash.chunk_position_2)
                    }
                    _ => false
                }
            }

            Task::UpdateXAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                let self_positions = additional_data_to_identify_and_hash;
                match other {
                    Task::UpdateXAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                        (self_positions.chunk_position_1 == additional_data_to_identify_and_hash.chunk_position_1 &&
                        self_positions.chunk_position_2 == additional_data_to_identify_and_hash.chunk_position_2) ||
                        (self_positions.chunk_position_2 == additional_data_to_identify_and_hash.chunk_position_1 &&
                            self_positions.chunk_position_1 == additional_data_to_identify_and_hash.chunk_position_2)
                    }
                    _ => false
                }
            }

            Task::UpdateZAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                let self_positions = additional_data_to_identify_and_hash;
                match other {
                    Task::UpdateZAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                        (self_positions.chunk_position_1 == additional_data_to_identify_and_hash.chunk_position_1 &&
                        self_positions.chunk_position_2 == additional_data_to_identify_and_hash.chunk_position_2) ||
                        (self_positions.chunk_position_2 == additional_data_to_identify_and_hash.chunk_position_1 &&
                            self_positions.chunk_position_1 == additional_data_to_identify_and_hash.chunk_position_2)
                    }
                    _ => false
                }
            }

            Task::GenerateChunkSideMesh { chunk_position, side, .. } => {
                let self_chunk_pos = chunk_position;
                let self_side = side;
                match other {
                    Task::GenerateChunkSideMesh { chunk_position, side, .. } => {
                        *self_chunk_pos == *chunk_position && *self_side == *side 
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
            Task::UpdateYAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                1.hash(state);
                additional_data_to_identify_and_hash.chunk_position_1.hash(state);
                additional_data_to_identify_and_hash.chunk_position_2.hash(state);
            }
            Task::UpdateXAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                2.hash(state);
                additional_data_to_identify_and_hash.chunk_position_1.hash(state);
                additional_data_to_identify_and_hash.chunk_position_2.hash(state);
            }
            Task::UpdateZAxisChunkPadding { additional_data_to_identify_and_hash, .. } => {
                3.hash(state);
                additional_data_to_identify_and_hash.chunk_position_1.hash(state);
                additional_data_to_identify_and_hash.chunk_position_2.hash(state);
            }
            Task::GenerateChunkSideMesh { chunk_position, side, .. } => {
                chunk_position.hash(state);
                side.hash(state);
            }
            Task::StopThread => {}
        }
    }
}

pub fn get_task_priority(task: &Task) -> u32 {
    match task {
        Task::StopThread => 0,
        Task::GenerateChunk { .. } => 1,
        Task::UpdateYAxisChunkPadding { .. } => 2,
        Task::UpdateXAxisChunkPadding { .. } => 2,
        Task::UpdateZAxisChunkPadding { .. } => 2,
        Task::GenerateChunkMesh { .. } => 3,
        Task::GenerateChunkSideMesh { .. } => 4,
    }
}

pub enum TaskResult {
    Requeue { task: Task },
    GenerateChunkMesh { },
    GenerateChunk { chunk_position: WorldPosition },
    UpdateChunkPadding { chunk_positions: Vec<(WorldPosition, BlockSide)> },
    UpdateChunkSideMesh { }
}