use core::time;
use std::sync::{Arc, RwLock};

use crate::{voxels::{mesh::Mesh, chunk::Chunk}, tasks::{TaskResult, Task, TaskError}, gpu_manager::gpu_data::vertex_gpu_data::VertexGPUData};
use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide, consts::MESH_METHOD};

pub struct GenerateChunkMeshProcessor {}

impl GenerateChunkMeshProcessor {
    pub fn process_task(chunk_position: &WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<wgpu::Queue>>) -> TaskResult {
        let chunk_index = *vertex_gpu_data.read().unwrap().pos_to_gpu_index.get(chunk_position).unwrap() as u32;
        
        let mut mesh = Mesh::new();

        match MESH_METHOD {
            "greedy" => mesh = Mesh::greedy(&chunk.read().unwrap(), chunk_index),
            "cull" => mesh = Mesh::cull(&chunk.read().unwrap(), chunk_index),
            _ => {}
        }

        let mut times_out_of_memory = 0;
        
        let mut enough_memory = vertex_gpu_data.read().unwrap().enough_memory_for_mesh(&mesh, chunk_position);
        while !enough_memory {
            times_out_of_memory += 1;
                if times_out_of_memory == 5 {
                    let memory_info = vertex_gpu_data.read().unwrap().get_memory_info();
                    return TaskResult::Requeue { 
                        task: Task::GenerateChunkMesh { chunk_position: *chunk_position, chunk, vertex_gpu_data, queue }, 
                        error: Some(TaskError::OutOfMemory { memory_info }) 
                    }
                }
            std::thread::sleep(std::time::Duration::from_millis(1000));
            enough_memory = vertex_gpu_data.read().unwrap().enough_memory_for_mesh(&mesh, chunk_position);
        }

        vertex_gpu_data.write().unwrap().add_mesh_data_drain(mesh, chunk_position, queue);

        TaskResult::GenerateChunkMesh {  }
    }
}

pub struct GenerateChunkSideMeshesProcessor {}

impl GenerateChunkSideMeshesProcessor {
    pub fn process_task(chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, vertex_gpu_data: Arc<RwLock<VertexGPUData>>, queue: Arc<RwLock<wgpu::Queue>>, sides: Vec<BlockSide>) -> TaskResult {
        if vertex_gpu_data.read().unwrap().has_meshed_position(&chunk_position) {
            let chunk_index = *vertex_gpu_data.read().unwrap().pos_to_gpu_index.get(&chunk_position).unwrap() as u32;

            let mut mesh = Mesh::new();

            match MESH_METHOD {
                "greedy" => mesh = Mesh::greedy_sided(&chunk.read().unwrap(), chunk_index, &sides),
                "cull" => mesh = Mesh::cull_side(&chunk.read().unwrap(), chunk_index, &sides),
                _ => {}
            }

            let mut times_out_of_memory = 0;

            let mut enough_memory = vertex_gpu_data.read().unwrap().enough_memory_for_mesh(&mesh, &chunk_position);
            while !enough_memory {
                times_out_of_memory += 1;
                if times_out_of_memory == 5 {
                    let memory_info = vertex_gpu_data.read().unwrap().get_memory_info();
                    return TaskResult::Requeue { 
                        task: Task::GenerateChunkSideMeshes { chunk_position, chunk, vertex_gpu_data, queue, sides }, 
                        error: Some(TaskError::OutOfMemory { memory_info }) 
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(1000));
                enough_memory = vertex_gpu_data.read().unwrap().enough_memory_for_mesh(&mesh, &chunk_position);
            }
            
            vertex_gpu_data.write().unwrap().update_side_mesh_data_drain(mesh, &chunk_position, queue, &sides);

            TaskResult::UpdateChunkSideMesh {  }
        } else {
            TaskResult::Requeue { task: Task::GenerateChunkSideMeshes { chunk_position, chunk, vertex_gpu_data, queue, sides }, error: None }
        }
    }
}