use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide, logi};
use web_time::Instant;

use crate::voxels::{mesh::Mesh, chunk::{ChunkBlockIterator, Chunk}};

pub fn cull(chunk: &Chunk, index: u32) -> Mesh {
    cull_side(chunk, index, &vec![BlockSide::FRONT, BlockSide::BACK, BlockSide::LEFT, BlockSide::RIGHT, BlockSide::TOP, BlockSide::BOTTOM])
}

pub fn cull_side(chunk: &Chunk, index: u32, sides: &Vec<BlockSide>) -> Mesh {
    let now = Instant::now();
    let mut mesh = Mesh::new();

    let mut vertex_arr = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut index_arr = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    let mut num_faces_generated = [0;6];

    let mut cbi = ChunkBlockIterator::new(chunk);

    while let Some(((i,j,k), block)) = cbi.get_next_block() {
        for side in sides.iter() {
            if !Mesh::is_adjacent_blocks_solid_side(chunk, i, j, k, *side) {
                vertex_arr[*side as usize].append(&mut Mesh::generate_cube_side(WorldPosition::new(i as i32-1,j as i32-1,k as i32-1), block.get_texture_indices(), index, *side));
                index_arr[*side as usize].append(&mut Mesh::generate_cube_indices_side(*side, num_faces_generated[*side as usize]));
                num_faces_generated[*side as usize] += 1;
            }
        }
        
        
    }

    mesh.add_vertices(vertex_arr, index_arr);

    let after = Instant::now();
    let time  = (after-now).as_millis();
    let cpos = chunk.position;
    logi!("Cull mesh for position {} sides {:?} took {} milliseconds", cpos, sides, time);

    mesh
}