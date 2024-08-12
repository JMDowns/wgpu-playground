use crossbeam::channel::{Sender, Receiver};
use fundamentals::enums::block_side::BlockSide;
use priority_queue::PriorityQueue;
use fundamentals::consts::{NUM_ADDITIONAL_THREADS, NUM_TASKS_PER_THREAD};
use crate::tasks::tasks_processors::generate_chunk_mesh_processor::GenerateChunkSideMeshesProcessor;
use crate::tasks::tasks_processors::update_chunk_padding_processors::{UpdateXAxisChunkPaddingProcessor, UpdateZAxisChunkPaddingProcessor};
use crate::tasks::{Task, TaskResult, get_task_priority};
use crate::tasks::
    tasks_processors::{
        generate_chunk_mesh_processor::GenerateChunkMeshProcessor,
        generate_chunk_processor::GenerateChunkProcessor,
        update_chunk_padding_processors::UpdateYAxisChunkPaddingProcessor
    };

struct ThreadInfo {
    pub sender: Sender<Task>,
    pub receiver: Receiver<TaskResult>,
}

pub struct ThreadTaskManager {
    threads: Vec<ThreadInfo>,
    task_queue: PriorityQueue<Task, u32>
}

impl ThreadTaskManager {
    pub fn new() -> Self {
        let mut threads = Vec::new();
        for _ in 0..NUM_ADDITIONAL_THREADS {
            let (s_task, r_task) = crossbeam::channel::bounded(NUM_TASKS_PER_THREAD);
            let (s_task_result, r_task_result) = crossbeam::channel::bounded(NUM_TASKS_PER_THREAD*2);
            let builder = std::thread::Builder::new();
            let _ = builder.spawn(move || {
                let mut should_run = true;
                while should_run {
                    match r_task.recv() {
                        Ok(task) => {
                            match task {
                                Task::StopThread {  } => should_run = false,
                                Task::GenerateChunk { chunk_position, world } => { 
                                    match s_task_result.send(GenerateChunkProcessor::process_task(&chunk_position, world)) {
                                        Ok(_) => {}
                                        Err(_) => should_run = false
                                    }
                                },
                                Task::GenerateChunkMesh { chunk_position, chunk, vertex_gpu_data, queue, chunk_index_state} => {
                                    match s_task_result.send(GenerateChunkMeshProcessor::process_task(&chunk_position, chunk, vertex_gpu_data, queue, chunk_index_state)) {
                                        Ok(_) => {}
                                        Err(_) => should_run = false
                                    }
                                }
                                Task::UpdateYAxisChunkPadding { chunk_below, chunk_above, .. } => {
                                    match s_task_result.send(UpdateYAxisChunkPaddingProcessor::process_task(chunk_below, chunk_above)) {
                                        Ok(_) => {}
                                        Err(_) => should_run = false
                                    }
                                }
                                Task::UpdateXAxisChunkPadding { chunk_front, chunk_back, .. } => {
                                    match s_task_result.send(UpdateXAxisChunkPaddingProcessor::process_task(chunk_front, chunk_back)) {
                                        Ok(_) => {}
                                        Err(_) => should_run = false
                                    }
                                }
                                Task::UpdateZAxisChunkPadding { chunk_left, chunk_right, .. } => {
                                    match s_task_result.send(UpdateZAxisChunkPaddingProcessor::process_task(chunk_left, chunk_right)) {
                                        Ok(_) => {}
                                        Err(_) => should_run = false
                                    }
                                }
                                Task::GenerateChunkSideMeshes { chunk_position, chunk, vertex_gpu_data, queue, sides, chunk_index_state } => {
                                    match s_task_result.send(GenerateChunkSideMeshesProcessor::process_task(chunk_position, chunk, vertex_gpu_data, queue, sides, chunk_index_state)) {
                                        Ok(_) => {}
                                        Err(_) => should_run = false
                                    }
                                }
                            }
                        },

                        Err(_) => should_run = false
                        
                    }
                    
                }
            });
            threads.push(ThreadInfo { sender: s_task, receiver: r_task_result });
        }

        Self {
           threads,
           task_queue: PriorityQueue::new() 
        }
    }

    fn append_sides_to_existing(existing_sides: &mut Vec<BlockSide>, other_sides: &Vec<BlockSide>) {
        for side in other_sides.iter() {
            if !existing_sides.contains(side) {
                existing_sides.push(*side)
            }
        }
    }

    pub fn push_task(&mut self, task: Task) {
        match task {
            Task::GenerateChunkSideMeshes { ref sides, .. } => {
                match self.task_queue.get_mut(&task) {
                    Some((existing_task, _)) => {
                        match existing_task {
                            Task::GenerateChunkSideMeshes { sides: existing_sides, .. } => {
                                Self::append_sides_to_existing(existing_sides, sides);
                                return;
                            }
                            _ => {}
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }

        let task_priority = get_task_priority(&task);
        self.task_queue.push(task, task_priority);
    }

    pub fn process_tasks(&mut self) -> Vec<TaskResult> {
        let mut result_vec = Vec::new();
        let mut full_threads = 0;
        while full_threads < NUM_ADDITIONAL_THREADS && !self.task_queue.is_empty() {
            full_threads = 0;
            for thread_info in self.threads.iter() {
                if !thread_info.sender.is_full() && !self.task_queue.is_empty() {
                    thread_info.sender.send(self.task_queue.pop().unwrap().0).unwrap();
                } else {
                    full_threads += 1;
                }
            }
        }
        for thread_info in self.threads.iter() {
            while !thread_info.receiver.is_empty() {
                result_vec.push(thread_info.receiver.recv().unwrap());
            }
        }
        result_vec
    }

    pub fn kill_threads(&self) {
        for thread_info in self.threads.iter() {
            thread_info.sender.send(Task::StopThread {  }).unwrap();
        }
    }
}

impl Drop for ThreadTaskManager {
    fn drop(&mut self) {
        self.kill_threads()
    }
}