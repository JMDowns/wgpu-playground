use std::sync::{Arc, RwLock};

use fundamentals::{consts::CHUNK_DIMENSION, enums::block_side::BlockSide};

use crate::{voxels::chunk::{ChunkBlockIterator, Chunk}, tasks::TaskResult};

pub struct UpdateYAxisChunkPaddingProcessor {}

impl UpdateYAxisChunkPaddingProcessor {
    pub fn process_task(chunk_below: Arc<RwLock<Chunk>>, chunk_above: Arc<RwLock<Chunk>>) -> TaskResult {
        let mut chunk_below = chunk_below.write().unwrap();
        let mut chunk_above = chunk_above.write().unwrap();
        let chunk_below_position = chunk_below.position;
        let chunk_above_position = chunk_above.position;
        for j in 0..CHUNK_DIMENSION as usize {
            for i in 0..CHUNK_DIMENSION as usize {
                chunk_below.update_solid_array(i+1, CHUNK_DIMENSION as usize+1, j+1, chunk_above.is_block_solid(i+1, 1, j+1));
                chunk_above.update_solid_array(i+1, 0, j+1, chunk_below.is_block_solid(i+1, CHUNK_DIMENSION as usize, j+1));
            }
        }

        TaskResult::UpdateChunkPadding { chunk_positions: vec![(chunk_below_position, BlockSide::TOP), (chunk_above_position, BlockSide::BOTTOM)] }
    }
}

pub struct UpdateXAxisChunkPaddingProcessor {}

impl UpdateXAxisChunkPaddingProcessor {
    pub fn process_task(chunk_front: Arc<RwLock<Chunk>>, chunk_back: Arc<RwLock<Chunk>>) -> TaskResult {
        let mut chunk_front = chunk_front.write().unwrap();
        let mut chunk_back = chunk_back.write().unwrap();
        let chunk_front_position = chunk_front.position;
        let chunk_back_position = chunk_back.position;
        for j in 0..CHUNK_DIMENSION as usize {
            for i in 0..CHUNK_DIMENSION as usize {
                chunk_front.update_solid_array(CHUNK_DIMENSION as usize+1, i+1, j+1, chunk_back.is_block_solid(1, i+1, j+1));
                chunk_back.update_solid_array(0, i+1, j+1, chunk_front.is_block_solid(CHUNK_DIMENSION as usize, i+1, j+1));
            }
        }

        TaskResult::UpdateChunkPadding { chunk_positions: vec![(chunk_front_position, BlockSide::BACK), (chunk_back_position, BlockSide::FRONT)] }
    }
}

pub struct UpdateZAxisChunkPaddingProcessor {}

impl UpdateZAxisChunkPaddingProcessor {
    pub fn process_task(chunk_left: Arc<RwLock<Chunk>>, chunk_right: Arc<RwLock<Chunk>>) -> TaskResult {
        let mut chunk_left = chunk_left.write().unwrap();
        let mut chunk_right = chunk_right.write().unwrap();
        let chunk_left_position = chunk_left.position;
        let chunk_right_position = chunk_right.position;
        for j in 0..CHUNK_DIMENSION as usize {
            for i in 0..CHUNK_DIMENSION as usize {
                chunk_left.update_solid_array(i+1, j+1, CHUNK_DIMENSION as usize+1, chunk_right.is_block_solid(i+1, j+1, 1));
                chunk_right.update_solid_array(i+1, j+1, 0, chunk_left.is_block_solid(i+1, j+1, CHUNK_DIMENSION as usize));
            }
        }

        TaskResult::UpdateChunkPadding { chunk_positions: vec![(chunk_left_position, BlockSide::RIGHT), (chunk_right_position, BlockSide::LEFT)] }
    }
}