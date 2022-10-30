use bitvec::bitvec;
use fundamentals::world_position::WorldPosition;
use derivables::block::Block;
use fundamentals::enums::block_type::BlockType;
use fundamentals::consts;
use noise::Perlin;
use noise::NoiseFn;
use bitvec::prelude::BitVec;
use consts::{CHUNK_DIMENSION, CHUNK_PLANE_SIZE, CHUNK_SIZE};

pub struct Chunk {
    pub position: WorldPosition,
    pub solid_array: BitVec,
    pub offsets_at_plane: Vec<u32>,
    pub blocks: Vec<Block>,
}

impl Chunk {
    pub fn _corner(position: &WorldPosition) -> Self {
        let mut blocks = vec![Block::new(BlockType::WOOD)];
        let mut offsets_at_plane = vec![1; CHUNK_DIMENSION as usize];
        offsets_at_plane[0] = 0;
        let mut solid_array = BitVec::new();
        solid_array.push(true);

        for _ in 1..CHUNK_SIZE {
            solid_array.push(false);
        }

        Chunk { position: *position, solid_array, offsets_at_plane, blocks}
    }

    pub fn _perlin(position: &WorldPosition) -> Self {
        let mut blocks_vec = Vec::new();

        let perlin = Perlin::new();

        let mut solid_array = BitVec::new();

        let mut offset = 0;

        let mut offsets_at_plane = vec![0; CHUNK_DIMENSION as usize];

        for i in 0..CHUNK_DIMENSION as i32 {
            offsets_at_plane[i as usize] = offset;
            for j in 0..CHUNK_DIMENSION as i32 {
                for k in 0..CHUNK_DIMENSION as i32 {
                    let bposition = WorldPosition::new(i- CHUNK_DIMENSION*position.x, j - CHUNK_DIMENSION*position.y, k - CHUNK_DIMENSION*position.z);
                    let perlin_sample = perlin.get(bposition.to_perlin_pos(0.1));
                    if perlin_sample < -0.3 || perlin_sample > 0.3 {
                        blocks_vec.push(Block::new(BlockType::WOOD));
                        solid_array.push(true);
                        offset += 1;
                    } else {
                        solid_array.push(false);
                    }
                }
            }
        }

        Chunk { position: *position, solid_array, offsets_at_plane, blocks: blocks_vec }
    }

    pub fn _solid(position: &WorldPosition) -> Self {
        let mut blocks_vec = Vec::new();
        let mut solid_array = BitVec::new();
        let mut offsets_at_plane = vec![0; CHUNK_DIMENSION as usize];

        for i in 0..CHUNK_DIMENSION {
            offsets_at_plane[i as usize] = (i*CHUNK_PLANE_SIZE) as u32;
        }
        
        for _ in 0..CHUNK_SIZE as i32 {
            blocks_vec.push(Block::new(BlockType::WOOD));
            solid_array.push(true);
        }

        Chunk { position: *position, solid_array, offsets_at_plane, blocks: blocks_vec }
    }

    pub fn checkerboard(position: &WorldPosition) -> Self {
        let mut blocks_vec = Vec::new();
        let mut solid_array = BitVec::new();

        let mut push_air = false;

        let mut offsets_at_plane = vec![0; CHUNK_DIMENSION as usize];

        for i in 0..CHUNK_DIMENSION {
            offsets_at_plane[i as usize] = (i*CHUNK_PLANE_SIZE / 2) as u32;
        }

        for i in 0..CHUNK_SIZE as i32 {
            if push_air {
                solid_array.push(false);
            } else {
                blocks_vec.push(Block::new(BlockType::WOOD));
                solid_array.push(true);
            }

            push_air = !push_air;

            if (i+1) % CHUNK_DIMENSION == 0 {
                push_air = !push_air
            }

            if (i+1) % CHUNK_PLANE_SIZE == 0 {
                push_air = !push_air
            }
        }

        Chunk { position: *position, solid_array, offsets_at_plane, blocks: blocks_vec}
    }

    pub fn get_block_at(&self, cx: usize, cy: usize, cz: usize) -> &Block{
        let mut offset = self.offsets_at_plane[cz] as usize;
        for j in 0..cy {
            for i in 0..CHUNK_DIMENSION as usize {
                if self.is_block_solid(i, j, cz) {
                    offset += 1;
                }
            }
        }
        for i in 0..cx {
            if self.is_block_solid(i, cy, cz) {
                offset += 1;
            }
        }
        &self.blocks[offset]
    }

    

    pub fn is_block_solid(&self, cx: usize, cy: usize, cz: usize) -> bool{
        self.solid_array[cx+(CHUNK_DIMENSION as usize)*cy+(CHUNK_PLANE_SIZE as usize)*cz]
    }
}

pub struct ChunkBlockIterator<'a> {
    chunk_ref: &'a Chunk,
    current_solid_offset: usize,
    current_block_offset: usize,
    has_started_iteration: bool
}

impl<'a> ChunkBlockIterator<'a> {
    pub fn new(chunk_ref: &'a Chunk) -> Self {
        ChunkBlockIterator { chunk_ref, current_solid_offset: 0, current_block_offset: 0, has_started_iteration: false }
    }

    pub fn get_next_block(&mut self) -> Option<(((usize, usize, usize), &Block))> {
        if self.has_started_iteration {
            self.current_solid_offset += 1;
            self.current_block_offset += 1;
        } else {
            self.has_started_iteration = true;
        }
        while (self.current_solid_offset < self.chunk_ref.solid_array.len() && self.chunk_ref.solid_array[self.current_solid_offset] == false) {
            self.current_solid_offset += 1;
        }


        if self.current_block_offset == self.chunk_ref.blocks.len() {
            return None
        }

        Some(((self.current_solid_offset % CHUNK_DIMENSION as usize, (self.current_solid_offset % CHUNK_PLANE_SIZE as usize) / CHUNK_DIMENSION as usize, self.current_solid_offset / CHUNK_PLANE_SIZE as usize), &self.chunk_ref.blocks[self.current_block_offset as usize]))
    }
}