use std::sync::{Arc, RwLock};

use fundamentals::logi;

use crate::{thread_task_manager::ThreadTaskManager, voxels::world::World, gpu_manager::GPUManager, tasks::{Task, TaskResult, ChunkUpdateTaskIdentifyingInfo}};

pub struct TaskManager {
    thread_task_manager: ThreadTaskManager,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager { thread_task_manager: ThreadTaskManager::new() }
    }

    pub fn push_task(&mut self, task: Task) {
        self.thread_task_manager.push_task(task);
    }

    pub fn process_tasks(&mut self, world: Arc<RwLock<World>>, gpu_manager: &mut GPUManager) {
        let mut task_results = self.thread_task_manager.process_tasks();

        let mut chunks_generated = 0;
        let mut meshes_generated = 0;

        for task_result in task_results.drain(..) {
            match task_result {
                TaskResult::GenerateChunk { chunk_position } => {
                    logi!("Generated chunk {}!", chunks_generated);
                    chunks_generated += 1;
                    self.thread_task_manager.push_task(gpu_manager.create_generate_chunk_mesh_task(chunk_position, world.read().unwrap().get_chunk_at(&chunk_position).unwrap()));
                    
                    let chunk_generated = world.read().unwrap().get_chunk_at(&chunk_position).unwrap();

                    let upper_position = chunk_position.get_position_incremented_by(0, 1, 0);
                    match world.read().unwrap().get_chunk_at(&upper_position) {
                        Some(chunk_above) => {
                            self.thread_task_manager.push_task( 
                                Task::UpdateYAxisChunkPadding { 
                                    chunk_below: chunk_generated.clone(), 
                                    chunk_above, 
                                    additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo {
                                        chunk_position_1: chunk_position,
                                        chunk_position_2: upper_position
                                    }
                                }
                            );
                        }
                        None => {}
                    }

                    let lower_position = chunk_position.get_position_incremented_by(0, -1, 0);
                    match world.read().unwrap().get_chunk_at(&lower_position) {
                        Some(chunk_below) => {
                            self.thread_task_manager.push_task( 
                                Task::UpdateYAxisChunkPadding { 
                                    chunk_below, 
                                    chunk_above: chunk_generated.clone(), 
                                    additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo {
                                        chunk_position_1: lower_position,
                                        chunk_position_2: chunk_position
                                    }
                                }
                            );
                        }
                        None => {}
                    }

                    let left_position = chunk_position.get_position_incremented_by(0, 0, -1);
                    match world.read().unwrap().get_chunk_at(&left_position) {
                        Some(chunk_left) => {
                            self.thread_task_manager.push_task( 
                                Task::UpdateZAxisChunkPadding { 
                                    chunk_left, 
                                    chunk_right: chunk_generated.clone(), 
                                    additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo {
                                        chunk_position_1: left_position,
                                        chunk_position_2: chunk_position
                                    }
                                }
                            );
                        }
                        None => {}
                    }

                    let right_position = chunk_position.get_position_incremented_by(0, 0, 1);
                    match world.read().unwrap().get_chunk_at(&right_position) {
                        Some(chunk_right) => {
                            self.thread_task_manager.push_task( 
                                Task::UpdateZAxisChunkPadding { 
                                    chunk_right, 
                                    chunk_left: chunk_generated.clone(), 
                                    additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo {
                                        chunk_position_1: right_position,
                                        chunk_position_2: chunk_position
                                    }
                                }
                            );
                        }
                        None => {}
                    }

                    let front_position = chunk_position.get_position_incremented_by(-1, 0, 0);
                    match world.read().unwrap().get_chunk_at(&front_position) {
                        Some(chunk_front) => {
                            self.thread_task_manager.push_task( 
                                Task::UpdateXAxisChunkPadding { 
                                    chunk_front, 
                                    chunk_back: chunk_generated.clone(), 
                                    additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo {
                                        chunk_position_1: front_position,
                                        chunk_position_2: chunk_position
                                    }
                                }
                            );
                        }
                        None => {}
                    }

                    let back_position = chunk_position.get_position_incremented_by(1, 0, 0);
                    match world.read().unwrap().get_chunk_at(&back_position) {
                        Some(chunk_back) => {
                            self.thread_task_manager.push_task( 
                                Task::UpdateXAxisChunkPadding { 
                                    chunk_back, 
                                    chunk_front: chunk_generated.clone(), 
                                    additional_data_to_identify_and_hash: ChunkUpdateTaskIdentifyingInfo {
                                        chunk_position_1: back_position,
                                        chunk_position_2: chunk_position
                                    }
                                }
                            );
                        }
                        None => {}
                    }
                },
                TaskResult::GenerateChunkMesh { } => {
                    logi!("Generated mesh {}!", meshes_generated);
                    gpu_manager.process_generate_chunk_mesh_task_result();
                    meshes_generated += 1;
                }
                TaskResult::UpdateChunkPadding { chunk_positions } => {
                    for (chunk_position, side) in chunk_positions {
                        self.thread_task_manager.push_task(gpu_manager.create_generate_chunk_side_mesh_task(chunk_position, world.read().unwrap().get_chunk_at(&chunk_position).unwrap(), side));
                    }
                }
                TaskResult::UpdateChunkSideMesh {  } => {}
                TaskResult::Requeue { task } => {
                    self.thread_task_manager.push_task(task);
                }
            }
        }
    }
}