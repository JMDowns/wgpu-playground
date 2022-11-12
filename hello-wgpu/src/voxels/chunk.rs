use fundamentals::consts::NUM_BLOCK_TYPES;
use fundamentals::world_position::WorldPosition;
use derivables::block::Block;
use fundamentals::enums::block_type::BlockType;
use fundamentals::consts;
use noise::Perlin;
use noise::NoiseFn;
use bitvec::prelude::BitVec;
use consts::{CHUNK_DIMENSION, CHUNK_PLANE_SIZE, CHUNK_SIZE, CHUNK_DIMENSION_WRAPPED, CHUNK_PLANE_SIZE_WRAPPED, CHUNK_SIZE_WRAPPED};

pub struct Chunk {
    pub position: WorldPosition,
    pub solid_array: BitVec,
    pub offsets_at_plane: Vec<u32>,
    pub blocks: Vec<Block>,
}

impl Chunk {
    pub fn _corner(position: &WorldPosition) -> Self {
        let mut cci = ChunkCreationIterator::new(*position);

        cci.push_block_type(BlockType::WOOD);

        for _ in 1..CHUNK_SIZE {
            cci.push_block_type(BlockType::AIR);
        }

        cci.return_chunk()
    }
    
    pub fn _xaxis(position: &WorldPosition) -> Self {
        let mut cci = ChunkCreationIterator::new(*position);

        for i in 0..CHUNK_SIZE {
            if i < CHUNK_DIMENSION as usize {
                cci.push_block_type(BlockType::WOOD);
            } else {
                cci.push_block_type(BlockType::AIR);
            }
        }

        cci.return_chunk()
    }

    pub fn _zaxis(position: &WorldPosition) -> Self {
        let mut cci = ChunkCreationIterator::new(*position);

        for i in 0..CHUNK_SIZE {
            if i % CHUNK_PLANE_SIZE as usize == 0 {
                cci.push_block_type(BlockType::WOOD);
            } else {
                cci.push_block_type(BlockType::AIR);
            }
        }

        cci.return_chunk()
    }

    pub fn _perlin(position: &WorldPosition) -> Self {
        let perlin = Perlin::new();
        let mut cci = ChunkCreationIterator::new(*position);

        for k in 0..CHUNK_DIMENSION as i32 {
            for j in 0..CHUNK_DIMENSION as i32 {
                for i in 0..CHUNK_DIMENSION as i32 {
                    let bposition = WorldPosition::new(i + CHUNK_DIMENSION*position.x, j + CHUNK_DIMENSION*position.y, k + CHUNK_DIMENSION*position.z);
                    let perlin_sample = perlin.get(bposition.to_perlin_pos(0.1));
                    if perlin_sample < -0.2 || perlin_sample > 0.2 {
                        cci.push_block_type(num::FromPrimitive::from_u8(fastrand::u8(1..NUM_BLOCK_TYPES as u8)).unwrap());
                        //cci.push_block_type(BlockType::WOOD);
                    } else {
                        cci.push_block_type(BlockType::AIR);
                    }
                }
            }
        }

        cci.return_chunk()
    }

    pub fn _solid(position: &WorldPosition) -> Self {
        let mut cci = ChunkCreationIterator::new(*position);

        for _ in 0..CHUNK_SIZE as i32 {
            cci.push_block_type(BlockType::WOOD);
        }

        cci.return_chunk()
    }

    pub fn checkerboard(position: &WorldPosition) -> Self {
        let mut push_air = false;

        let mut cci = ChunkCreationIterator::new(*position);
        for i in 0..CHUNK_SIZE as i32 {
            if push_air {
                cci.push_block_type(BlockType::AIR);
            } else {
                cci.push_block_type(BlockType::WOOD);
            }

            push_air = !push_air;

            if (i+1) % CHUNK_DIMENSION == 0 {
                push_air = !push_air
            }

            if (i+1) % CHUNK_PLANE_SIZE == 0 {
                push_air = !push_air
            }
        }

        cci.return_chunk()
    }

    pub fn _get_block_at(&self, cx: usize, cy: usize, cz: usize) -> &Block{
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
        self.solid_array[cx+(CHUNK_DIMENSION_WRAPPED as usize)*cy+(CHUNK_PLANE_SIZE_WRAPPED as usize)*cz]
    }

    pub fn update_solid_array(&mut self, cx: usize, cy: usize, cz: usize, solid_value: bool) {
        self.solid_array.set(cx+(CHUNK_DIMENSION_WRAPPED as usize)*cy+(CHUNK_PLANE_SIZE_WRAPPED as usize)*cz, solid_value); 
    }
}

struct ChunkCreationIterator {
    position: WorldPosition,
    solid_array: BitVec,
    offsets_at_plane: Vec<u32>,
    blocks: Vec<Block>,
    local_x: usize,
    local_y: usize,
    local_z: usize,
    block_offset: u32,
}

impl ChunkCreationIterator {
    pub fn new(position: WorldPosition) -> Self {
        let mut solid_array = BitVec::with_capacity(CHUNK_SIZE_WRAPPED);
        for _ in 0..(CHUNK_PLANE_SIZE_WRAPPED + CHUNK_DIMENSION_WRAPPED + 1) {
            solid_array.push(false);
        }
        ChunkCreationIterator { position, solid_array, offsets_at_plane: Vec::new(), blocks: Vec::new(), local_x: 1, local_y: 1, local_z: 1, block_offset: 0 }
    }

    pub fn return_chunk(self) -> Chunk {
        Chunk { position: self.position, solid_array: self.solid_array, offsets_at_plane: self.offsets_at_plane, blocks: self.blocks }
    }

    pub fn push_block_type(&mut self, block_type: BlockType) {
        let is_solid = block_type != BlockType::AIR;
        self.solid_array.push(is_solid);
        if is_solid {
            self.blocks.push(Block::new(block_type));
            self.block_offset += 1;
        }
        self.local_x += 1;
        if self.local_x == CHUNK_DIMENSION_WRAPPED - 1 {
            self.solid_array.push(false);
            self.solid_array.push(false);
            self.local_x = 1;
            self.local_y += 1;
            if self.local_y == CHUNK_DIMENSION_WRAPPED - 1 {
                self.offsets_at_plane.push(self.block_offset);
                for _ in 0..2*CHUNK_DIMENSION_WRAPPED {
                    self.solid_array.push(false);
                }
                self.local_y = 1;
                self.local_z += 1;
                if self.local_z == CHUNK_DIMENSION_WRAPPED - 1 {
                    for _ in 0..CHUNK_PLANE_SIZE_WRAPPED-CHUNK_DIMENSION_WRAPPED-1 {
                        self.solid_array.push(false);
                    }
                    return;
                }
            }
        }
    }
}

pub struct ChunkBlockIterator<'a> {
    chunk_ref: &'a Chunk,
    current_solid_offset: usize,
    current_block_offset: usize,
    has_started_iteration: bool,
    local_x: usize,
    local_y: usize,
    local_z: usize
}

impl<'a> ChunkBlockIterator<'a> {
    pub fn new(chunk_ref: &'a Chunk) -> Self {
        ChunkBlockIterator { chunk_ref, current_solid_offset: 0, current_block_offset: 0, has_started_iteration: false, local_x: 0, local_y: 0, local_z: 0 }
    }

    pub fn get_next_block(&mut self) -> Option<((usize, usize, usize), &Block)> {
        if self.has_started_iteration {
            self.current_block_offset += 1;
            self.current_solid_offset += 1;
            self.local_x += 1;
        } else {
            self.has_started_iteration = true;
        }

        if self.current_block_offset >= self.chunk_ref.blocks.len() {
            return None
        }

        if self.local_z == 0 {
            self.current_solid_offset += CHUNK_PLANE_SIZE_WRAPPED;
            self.local_z += 1;
        }
        if self.local_y == 0 {
            self.current_solid_offset += CHUNK_DIMENSION_WRAPPED;
            self.local_y += 1;
        }
        if self.local_x == 0 {
            self.current_solid_offset += 1;
            self.local_x += 1;
        }

        if self.local_x == CHUNK_DIMENSION_WRAPPED - 1 {
            self.current_solid_offset += 2;
            self.local_x = 1;
            self.local_y += 1;
            if self.local_y == CHUNK_DIMENSION_WRAPPED - 1 {
                self.current_solid_offset += 2*CHUNK_DIMENSION_WRAPPED;
                self.local_y = 1;
                self.local_z += 1;
                if self.local_z == CHUNK_DIMENSION_WRAPPED - 1 {
                    return None;
                }
            }
        }

        while self.current_solid_offset < self.chunk_ref.solid_array.len() && self.chunk_ref.solid_array[self.current_solid_offset] == false {
            self.local_x += 1;
            self.current_solid_offset += 1;
            if self.local_x == CHUNK_DIMENSION_WRAPPED - 1 {
                self.current_solid_offset += 2;
                self.local_x = 1;
                self.local_y += 1;
                if self.local_y == CHUNK_DIMENSION_WRAPPED - 1 {
                    self.current_solid_offset += 2*CHUNK_DIMENSION_WRAPPED;
                    self.local_y = 1;
                    self.local_z += 1;
                    if self.local_z == CHUNK_DIMENSION_WRAPPED - 1 {
                        return None;
                    }
                }
            }
        }

        Some(((self.local_x, self.local_y, self.local_z), &self.chunk_ref.blocks[self.current_block_offset as usize]))
    }
}