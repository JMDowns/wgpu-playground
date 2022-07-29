use super::position::Position;
use super::block::{Block};
use fundamentals::enums::block_type::BlockType;
use fundamentals::consts;

pub struct Chunk {
    pub position: Position,
    pub blocks: [Block; 4096]
}

impl Chunk {
    pub fn random(position: &Position) -> Self {
        let mut blocks_vec = Vec::new();

        for _ in 0..4096 {
            if fastrand::u8(0..10) == 0 {
                blocks_vec.push(Block::new(num::FromPrimitive::from_u16(fastrand::u16(1..consts::NUM_BLOCK_TYPES)).unwrap()))
            } else {
                blocks_vec.push(Block::new(BlockType::AIR));
            }
        }

        Chunk { position: *position, blocks: Chunk::get_arr(blocks_vec) }
    }

    pub fn get_block_at(&self, cx: usize, cy: usize, cz: usize) -> &Block{
        &self.blocks[cx+16*cy+256*cz]
    }

    pub fn get_absolute_coords_usize(&self, cx: usize, cy: usize, cz: usize)  -> Position {
        Position { x: cx as i32 - 16*self.position.x, y: cy as i32 - 16*self.position.y, z: cz as i32 - 16*self.position.z}
    }

    fn get_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
        v.try_into().unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
    }
}