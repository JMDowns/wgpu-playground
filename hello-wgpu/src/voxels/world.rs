use std::{collections::HashMap, sync::{RwLock, Arc}};
use super::chunk::Chunk;
use fundamentals::{world_position::WorldPosition, consts::CHUNK_GENERATION_METHOD};

pub struct World {
    chunks: HashMap<WorldPosition, Arc<RwLock<Chunk>>>,
}

impl World {
    pub fn new() -> Self {
        World { chunks: HashMap::new() }
    }

    pub fn generate_chunk_at(position: &WorldPosition) -> Chunk {
        match CHUNK_GENERATION_METHOD {
            "perlin" => Chunk::perlin(position),
            "checkerboard" => Chunk::checkerboard(position),
            "solid" => Chunk::solid(position),
            "empty" => Chunk::empty(position),
            _ => Chunk::empty(position)
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, Arc::new(RwLock::new(chunk)));
    }
    
    pub fn get_chunk_at(&self, pos: &WorldPosition) -> Option<Arc<RwLock<Chunk>>> {
        match self.chunks.get(pos) {
            Some(chunk) => Some(chunk.clone()),
            None => None
        }
    }
}