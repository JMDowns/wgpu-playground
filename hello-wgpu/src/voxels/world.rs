use std::collections::HashMap;
use super::chunk::Chunk;
use super::position::Position;
use crate::voxels::mesh::Mesh;

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
        Mesh::stupid(self.chunks.get(pos).unwrap())
    }
}