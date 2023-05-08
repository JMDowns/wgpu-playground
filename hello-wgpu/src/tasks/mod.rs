use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

use crate::gpu_manager::gpu_data::vertex_gpu_data::{VertexGPUData, MemoryInfo};
use crate::voxels::chunk::Chunk;
use crate::voxels::world::World;
use fundamentals::enums::block_side::BlockSide;
use fundamentals::world_position::WorldPosition;
use wgpu::Queue;
use fundamentals::consts::{GENERATE_CHUNK_PRIORITY, GENERATE_MESH_PRIORITY, GENERATE_MESH_SIDE_PRIORITY, UPDATE_CHUNK_PADDING_X_PRIORITY, UPDATE_CHUNK_PADDING_Y_PRIORITY, UPDATE_CHUNK_PADDING_Z_PRIORITY};

pub mod tasks_processors;

pub enum Task {
    StopThread,
    GenerateChunkMesh { chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<Queue>> },
    GenerateChunk { chunk_position: WorldPosition, world: Arc<RwLock<World>>},
    UpdateYAxisChunkPadding { chunk_below: Arc<RwLock<Chunk>>, chunk_above: Arc<RwLock<Chunk>>, additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo},
    UpdateXAxisChunkPadding { chunk_front: Arc<RwLock<Chunk>>, chunk_back: Arc<RwLock<Chunk>>, additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo},
    UpdateZAxisChunkPadding { chunk_left: Arc<RwLock<Chunk>>, chunk_right: Arc<RwLock<Chunk>>, additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo},
    GenerateChunkSideMeshes { chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<Queue>>, sides: Vec<BlockSide> }
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

            Task::GenerateChunkSideMeshes { chunk_position: self_chunk_pos, .. } => {
                match other {
                    Task::GenerateChunkSideMeshes { chunk_position,.. } => {
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
            Task::GenerateChunkSideMeshes { chunk_position, .. } => {
                chunk_position.hash(state);
            }
            Task::StopThread => {}
        }
    }
}

pub fn get_task_priority(task: &Task) -> u32 {
    match task {
        Task::StopThread => 0,
        Task::GenerateChunk { .. } => GENERATE_CHUNK_PRIORITY,
        Task::UpdateYAxisChunkPadding { .. } => UPDATE_CHUNK_PADDING_Y_PRIORITY,
        Task::UpdateXAxisChunkPadding { .. } => UPDATE_CHUNK_PADDING_X_PRIORITY,
        Task::UpdateZAxisChunkPadding { .. } => UPDATE_CHUNK_PADDING_Z_PRIORITY,
        Task::GenerateChunkMesh { .. } => GENERATE_MESH_PRIORITY,
        Task::GenerateChunkSideMeshes { .. } => GENERATE_MESH_SIDE_PRIORITY,
    }
}

pub enum TaskResult {
    Requeue { task: Task, error: Option<TaskError> },
    GenerateChunkMesh { },
    GenerateChunk { chunk_position: WorldPosition },
    UpdateChunkPadding { chunk_positions: Vec<(WorldPosition, BlockSide)> },
    UpdateChunkSideMesh { }
}

pub enum TaskError {
    OutOfMemory { memory_info: MemoryInfo }
}