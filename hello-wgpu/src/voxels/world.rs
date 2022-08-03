use std::collections::HashMap;
use fundamentals::consts::CHUNK_DIMENSION;
use super::chunk::Chunk;
use super::position::Position;
use crate::voxels::mesh::Mesh;
use fundamentals::enums::block_type::BlockType;

pub struct World {
    chunks: HashMap<Position, Chunk>,
}

impl World {
    pub fn new(radius: i32) -> Self {

        let mut chunks = HashMap::new();
        for x in -radius..radius+1 {
            for y in -radius..radius+1 {
                for z in -radius..radius+1{
                    let pos = Position::new(x,y,z);
                    chunks.insert(pos, Chunk::perlin(&pos));
                }
            }
        }

        World { chunks }
    }
    
    pub fn generate_mesh_at(&self, pos: &Position) -> Mesh {
        let block_solid_data = self.list_if_blocks_are_solid_in_and_surrounding_chunk(pos);
        Mesh::cull_ambient_occlusion(self.chunks.get(pos).unwrap(), block_solid_data)
    }

    fn list_if_blocks_are_solid_in_and_surrounding_chunk(&self, pos: &Position) -> [[[bool; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2] {
        let mut block_info = [[[false; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2];
        let chunk_options = pos.generate_neighborhood1_positions().map(|d2arr| d2arr.map(|d1arr| d1arr.map(|pos| self.chunks.get(&pos))));
        for k in 0..(CHUNK_DIMENSION+2 )as usize {
            for j in 0..(CHUNK_DIMENSION+2 )as usize {
                for i in 0..(CHUNK_DIMENSION+2 )as usize {
                    let offset_i = i + CHUNK_DIMENSION as usize-1;
                    let offset_j = j + CHUNK_DIMENSION as usize-1;
                    let offset_k = k + CHUNK_DIMENSION as usize-1;
                    block_info[i][j][k] = match chunk_options[offset_i / CHUNK_DIMENSION as usize][offset_j / CHUNK_DIMENSION as usize][offset_k / CHUNK_DIMENSION as usize] {
                        Some(chunk) => chunk.get_block_at(offset_i % CHUNK_DIMENSION as usize, offset_j % CHUNK_DIMENSION as usize, offset_k % CHUNK_DIMENSION as usize).get_block_type() != BlockType::AIR,
                        None => false
                    };
                }
            }
        }

        block_info
    }
}