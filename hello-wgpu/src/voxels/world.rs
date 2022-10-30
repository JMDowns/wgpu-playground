use std::collections::HashMap;
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
        Chunk::checkerboard(pos)
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }
    
    pub fn generate_mesh_at(&self, pos: &WorldPosition, index: u32) -> Option<Mesh> {
        match self.chunks.get(pos) {
            Some(chunk) => Some(Mesh::cull_ambient_occlusion(chunk, index)),
            None => None
        }
    }
}