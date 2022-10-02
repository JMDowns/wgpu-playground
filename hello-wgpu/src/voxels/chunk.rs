use fundamentals::world_position::WorldPosition;
use derivables::block::Block;
use fundamentals::enums::block_type::BlockType;
use fundamentals::consts;
use noise::Perlin;
use noise::NoiseFn;
use consts::{CHUNK_DIMENSION, CHUNK_PLANE_SIZE, CHUNK_SIZE};

pub struct Chunk {
    pub position: WorldPosition,
    pub blocks: Vec<Block>,
}

impl Chunk {
    pub fn perlin(position: &WorldPosition) -> Self {
        let mut blocks_vec = Vec::new();

        let perlin = Perlin::new();

        for i in 0..CHUNK_DIMENSION as i32 {
            for j in 0..CHUNK_DIMENSION as i32 {
                for k in 0..CHUNK_DIMENSION as i32 {
                    let bposition = WorldPosition::new(i- CHUNK_DIMENSION*position.x, j - CHUNK_DIMENSION*position.y, k - CHUNK_DIMENSION*position.z);
                    let perlin_sample = perlin.get(bposition.to_perlin_pos(0.1));
                    if perlin_sample < -0.1 || perlin_sample > 0.1 {
                        blocks_vec.push(Block::new(BlockType::WOOD));
                    } else {
                        blocks_vec.push(Block::new(BlockType::AIR));
                    }
                }
            }
        }

        Chunk { position: *position, blocks: blocks_vec }
    }

    pub fn _solid(position: &WorldPosition) -> Self {
        let mut blocks_vec = Vec::new();

        for _ in 0..CHUNK_SIZE as i32 {
            blocks_vec.push(Block::new(BlockType::DIRT));
        }

        Chunk { position: *position, blocks: blocks_vec }
    }

    pub fn checkerboard(position: &WorldPosition) -> Self {
        let mut blocks_vec = Vec::new();

        let mut push_air = false;

        for i in 0..CHUNK_SIZE as i32 {
            if push_air {
                blocks_vec.push(Block::new(BlockType::AIR));
            } else {
                blocks_vec.push(Block::new(BlockType::DIRT));
            }

            push_air = !push_air;

            if (i+1) % CHUNK_DIMENSION == 0 {
                push_air = !push_air
            }

            if (i+1) % CHUNK_PLANE_SIZE == 0 {
                push_air = !push_air
            }
        }

        Chunk { position: *position, blocks: blocks_vec }
    }

    pub fn get_block_at(&self, cx: usize, cy: usize, cz: usize) -> &Block{
        &self.blocks[cx+(CHUNK_DIMENSION as usize)*cy+(CHUNK_PLANE_SIZE as usize)*cz]
    }
}