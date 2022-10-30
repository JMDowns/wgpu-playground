use std::collections::HashMap;
use fundamentals::consts::CHUNK_DIMENSION;
use super::chunk::Chunk;
use crate::voxels::mesh::Mesh;
use fundamentals::world_position::WorldPosition;

pub struct World {
    chunks: HashMap<WorldPosition, Chunk>,
}

impl World {
    pub fn new() -> Self {
        World { chunks: HashMap::new() }
    }

    pub fn generate_chunk_at(pos: &WorldPosition) -> Chunk {
        Chunk::_solid(pos)
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }
    
    pub fn generate_mesh_at(&self, pos: &WorldPosition, index: u32) -> Option<Mesh> {
        let block_solid_data = self.list_if_blocks_are_solid_in_and_surrounding_chunk(pos);
        match self.chunks.get(pos) {
            Some(chunk) => Some(Mesh::cull_ambient_occlusion(chunk, block_solid_data, index)),
            None => None
        }
        
    }

    fn list_if_blocks_are_solid_in_and_surrounding_chunk(&self, pos: &WorldPosition) -> Vec<Vec<Vec<bool>>> {
        let mut block_info = Vec::new();
        let chunk_options = pos.generate_neighborhood1_world_positions().map(|d2arr| d2arr.map(|d1arr| d1arr.map(|pos| self.chunks.get(&pos))));
        for i in 0..(CHUNK_DIMENSION+2 )as usize {
            block_info.push(Vec::new());
            for j in 0..(CHUNK_DIMENSION+2 )as usize {
                block_info[i].push(Vec::new());
                for k in 0..(CHUNK_DIMENSION+2 )as usize {
                    let offset_i = i + CHUNK_DIMENSION as usize-1;
                    let offset_j = j + CHUNK_DIMENSION as usize-1;
                    let offset_k = k + CHUNK_DIMENSION as usize-1;
                    block_info[i][j].push(match chunk_options[offset_i / CHUNK_DIMENSION as usize][offset_j / CHUNK_DIMENSION as usize][offset_k / CHUNK_DIMENSION as usize] {
                        Some(chunk) => {
                            chunk.is_block_solid(offset_i % CHUNK_DIMENSION as usize, offset_j % CHUNK_DIMENSION as usize, offset_k % CHUNK_DIMENSION as usize)
                        },
                        None => false
                    });
                }
            }
        }

        block_info
    }
}