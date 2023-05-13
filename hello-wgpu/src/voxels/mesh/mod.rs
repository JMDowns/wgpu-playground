mod face;
mod greedy;
mod cull;

use derivables::{vertex::Vertex, block::Block};
use fundamentals::{world_position::WorldPosition, enums::{block_side::BlockSide, block_type::BlockTypeSize}, consts::NUM_VERTICES_IN_BUCKET};
use self::face::Face;

use super::chunk::Chunk;

#[derive(Debug)]
pub struct Mesh {
    pub front: (Vec<Vertex>, Vec<u32>, u32),
    pub back: (Vec<Vertex>, Vec<u32>, u32),
    pub left: (Vec<Vertex>, Vec<u32>, u32),
    pub right: (Vec<Vertex>, Vec<u32>, u32),
    pub top: (Vec<Vertex>, Vec<u32>, u32),
    pub bottom: (Vec<Vertex>, Vec<u32>, u32),
}

impl Mesh {
    pub fn new() -> Self {
        Mesh { 
            front: (Vec::new(), Vec::new(), 0), 
            back: (Vec::new(), Vec::new(), 0), 
            left: (Vec::new(), Vec::new(), 0), 
            right: (Vec::new(), Vec::new(), 0),
            top: (Vec::new(), Vec::new(), 0),
            bottom: (Vec::new(), Vec::new(), 0) 
        }
    }

    pub fn cull(chunk: &Chunk, index: u32) -> Self {
        cull::cull(chunk, index)
    }

    pub fn cull_side(chunk: &Chunk, index: u32, sides: &Vec<BlockSide>) -> Self {
        cull::cull_side(chunk, index, sides)
    }

    pub fn greedy(chunk: &Chunk, index: u32) -> Self {
        greedy::greedy(chunk, index)
    }

    pub fn greedy_sided(chunk: &Chunk, index: u32, sides: &Vec<BlockSide>) -> Self {
        greedy::greedy_sided(chunk, index, sides)
    }

    pub fn add_vertices(&mut self, mut block_vertices: [Vec<Vertex>; 6], block_indices: [Vec<u32>; 6]) {
        self.front.1.append(&mut block_indices[0].iter().map(|e| (e+self.front.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.back.1.append(&mut block_indices[1].iter().map(|e| (e+self.back.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.left.1.append(&mut block_indices[2].iter().map(|e| (e+self.left.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.right.1.append(&mut block_indices[3].iter().map(|e| (e+self.right.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.top.1.append(&mut block_indices[4].iter().map(|e| (e+self.top.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.bottom.1.append(&mut block_indices[5].iter().map(|e| (e+self.bottom.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());

        self.front.0.append(&mut block_vertices[0]);
        self.back.0.append(&mut block_vertices[1]);
        self.left.0.append(&mut block_vertices[2]);
        self.right.0.append(&mut block_vertices[3]);
        self.top.0.append(&mut block_vertices[4]);
        self.bottom.0.append(&mut block_vertices[5]);

        self.front.2 = self.front.1.len() as u32;
        self.back.2 = self.back.1.len() as u32;
        self.left.2 = self.left.1.len() as u32;
        self.right.2 = self.right.1.len() as u32;
        self.top.2 = self.top.1.len() as u32;
        self.bottom.2 = self.bottom.1.len() as u32;
    }

    pub fn generate_adjacent_blocks(chunk: &Chunk, i: usize, j: usize, k: usize) -> [bool; 6] {
        let mut adjacency_data = [false;6];
        adjacency_data[0] = chunk.is_block_solid(i-1, j, k);
        adjacency_data[1] = chunk.is_block_solid(i+1, j, k);
        adjacency_data[2] = chunk.is_block_solid(i, j, k-1);
        adjacency_data[3] = chunk.is_block_solid(i, j, k+1);
        adjacency_data[4] = chunk.is_block_solid(i, j+1, k);
        adjacency_data[5] = chunk.is_block_solid(i, j-1, k);
        adjacency_data
    }

    pub fn is_adjacent_blocks_solid_side(chunk: &Chunk, i: usize, j: usize, k: usize, side: BlockSide) -> bool {
        match side {
            BlockSide::FRONT => chunk.is_block_solid(i-1, j, k),
            BlockSide::BACK => chunk.is_block_solid(i+1, j, k),
            BlockSide::LEFT => chunk.is_block_solid(i, j, k-1),
            BlockSide::RIGHT => chunk.is_block_solid(i, j, k+1),
            BlockSide::TOP => chunk.is_block_solid(i, j+1, k),
            BlockSide::BOTTOM => chunk.is_block_solid(i, j-1, k)
        }
    }

    fn generate_face_vertices(face: &Face, index: u32) -> Vec<Vertex> {
        let texture_indices = &Block::get_texture_indices_from_int(face.block_type_int as BlockTypeSize);
        let (texture_index, u_offset, v_offset) = match face.block_side {
            BlockSide::FRONT => {
                (0, (face.lr.2-face.ll.2) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::BACK => {
                (1, (face.ll.2-face.lr.2) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::LEFT => {
                (2, (face.ll.0-face.lr.0) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::RIGHT => {
                (3, (face.lr.0-face.ll.0) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::TOP => {
                (4, (face.lr.2-face.ll.2) as u8, (face.ul.0-face.ll.0) as u8)
            }
            BlockSide::BOTTOM => {
                (5, (face.ll.2-face.lr.2) as u8, (face.ul.0-face.ll.0) as u8)
            }
        };

        [
            Vertex::new(WorldPosition::new(face.ll.0 as i32, face.ll.1 as i32, face.ll.2 as i32), texture_indices[texture_index], 0, v_offset, index),
            Vertex::new(WorldPosition::new(face.lr.0 as i32, face.lr.1 as i32, face.lr.2 as i32), texture_indices[texture_index], u_offset, v_offset, index),
            Vertex::new(WorldPosition::new(face.ul.0 as i32, face.ul.1 as i32, face.ul.2 as i32), texture_indices[texture_index], 0, 0, index),
            Vertex::new(WorldPosition::new(face.ur.0 as i32, face.ur.1 as i32, face.ur.2 as i32), texture_indices[texture_index], u_offset, 0, index)
        ].to_vec()
        
    }

    fn generate_face_indices(num_faces_generated: u32) -> Vec<u32> {
        [
            0+num_faces_generated*4,1+num_faces_generated*4,3+num_faces_generated*4,
            0+num_faces_generated*4,3+num_faces_generated*4,2+num_faces_generated*4,
        ].to_vec()
    }

    fn generate_cube_side(pos: WorldPosition, tex_index_arr: [usize; 6], index: u32, side: BlockSide) -> Vec<Vertex> {
        let positions = pos.generate_vertex_world_positions();
        match side {
            BlockSide::FRONT => {
                [
                    Vertex::new(positions[0], tex_index_arr[0], 0, 1, index),
                    Vertex::new(positions[1], tex_index_arr[0], 1, 1, index),
                    Vertex::new(positions[2], tex_index_arr[0], 0, 0, index),
                    Vertex::new(positions[3], tex_index_arr[0], 1, 0, index),
                ].to_vec()
            },
            BlockSide::BACK => {
                [
                    Vertex::new(positions[4], tex_index_arr[1], 1, 1, index),
                    Vertex::new(positions[5], tex_index_arr[1], 0, 1, index),
                    Vertex::new(positions[6], tex_index_arr[1], 1, 0, index),
                    Vertex::new(positions[7], tex_index_arr[1], 0, 0, index),
                ].to_vec()
            },
            BlockSide::LEFT => {
                [
                    Vertex::new(positions[0], tex_index_arr[2], 1, 1, index),
                    Vertex::new(positions[2], tex_index_arr[2], 1, 0, index),
                    Vertex::new(positions[4], tex_index_arr[2], 0, 1, index),
                    Vertex::new(positions[6], tex_index_arr[2], 0, 0, index),
                ].to_vec()
            },
            BlockSide::RIGHT => {
                [
                    Vertex::new(positions[1], tex_index_arr[3], 0, 1, index),
                    Vertex::new(positions[3], tex_index_arr[3], 0, 0, index),
                    Vertex::new(positions[5], tex_index_arr[3], 1, 1, index),
                    Vertex::new(positions[7], tex_index_arr[3], 1, 0, index),
                ].to_vec()
            },
            BlockSide::TOP => {
                [
                    Vertex::new(positions[2], tex_index_arr[4], 0, 1, index),
                    Vertex::new(positions[3], tex_index_arr[4], 1, 1, index),
                    Vertex::new(positions[6], tex_index_arr[4], 0, 0, index),
                    Vertex::new(positions[7], tex_index_arr[4], 1, 0, index),
                ].to_vec()
            },
            BlockSide::BOTTOM => {
                [
                    Vertex::new(positions[0], tex_index_arr[5], 0, 0, index),
                    Vertex::new(positions[1], tex_index_arr[5], 1, 0, index),
                    Vertex::new(positions[4], tex_index_arr[5], 0, 1, index),
                    Vertex::new(positions[5], tex_index_arr[5], 1, 1, index),
                ].to_vec()
            }
        }
    }
    
    fn generate_cube_indices_side(side: BlockSide, num_faces_generated: u32) -> Vec<u32> {
        match side {
            BlockSide::FRONT => {
                [
                    (0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                    (0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                ].to_vec()
            },
            BlockSide::BACK => {
                [    
                    (1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                    (1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                ].to_vec()
            },
            BlockSide::LEFT => {
                [
                    (2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                    (2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                ].to_vec()
            },
            BlockSide::RIGHT => {
                [
                    (0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                    (0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                ].to_vec()
            },
            BlockSide::TOP => {
                [
                    (0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                    (0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                ].to_vec()
            },
            BlockSide::BOTTOM => {
                [
                    (1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(0+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,
                    (1+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(2+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET,(3+4*num_faces_generated)%NUM_VERTICES_IN_BUCKET
                ].to_vec()
            }
        }
    }

    fn generate_occlusion_cube_indices_side(side: BlockSide, num_faces_generated: u32) -> Vec<u32> {
        match side {
            BlockSide::FRONT => {
                [
                    (0+4*num_faces_generated),(1+4*num_faces_generated),(3+4*num_faces_generated),
                    (0+4*num_faces_generated),(3+4*num_faces_generated),(2+4*num_faces_generated),
                ].to_vec()
            },
            BlockSide::BACK => {
                [    
                    (1+4*num_faces_generated),(0+4*num_faces_generated),(2+4*num_faces_generated),
                    (1+4*num_faces_generated),(2+4*num_faces_generated),(3+4*num_faces_generated),
                ].to_vec()
            },
            BlockSide::LEFT => {
                [
                    (2+4*num_faces_generated),(0+4*num_faces_generated),(1+4*num_faces_generated),
                    (2+4*num_faces_generated),(1+4*num_faces_generated),(3+4*num_faces_generated),
                ].to_vec()
            },
            BlockSide::RIGHT => {
                [
                    (0+4*num_faces_generated),(2+4*num_faces_generated),(3+4*num_faces_generated),
                    (0+4*num_faces_generated),(3+4*num_faces_generated),(1+4*num_faces_generated),
                ].to_vec()
            },
            BlockSide::TOP => {
                [
                    (0+4*num_faces_generated),(1+4*num_faces_generated),(3+4*num_faces_generated),
                    (0+4*num_faces_generated),(3+4*num_faces_generated),(2+4*num_faces_generated),
                ].to_vec()
            },
            BlockSide::BOTTOM => {
                [
                    (1+4*num_faces_generated),(0+4*num_faces_generated),(2+4*num_faces_generated),
                    (1+4*num_faces_generated),(2+4*num_faces_generated),(3+4*num_faces_generated)
                ].to_vec()
            }
        }
    }

    fn generate_occlusion_cube_side(chunk_position: &WorldPosition, index: u32, side: BlockSide) -> Vec<Vertex> {
        let positions = chunk_position.generate_occlusion_cube_world_positions();

        let tex_index_arr = [0,0,0,0,0,0];

        match side {
            BlockSide::FRONT => {
                [
                    Vertex::new(positions[0], tex_index_arr[0], 0, 1, index),
                    Vertex::new(positions[1], tex_index_arr[0], 1, 1, index),
                    Vertex::new(positions[2], tex_index_arr[0], 0, 0, index),
                    Vertex::new(positions[3], tex_index_arr[0], 1, 0, index),
                ].to_vec()
            },
            BlockSide::BACK => {
                [
                    Vertex::new(positions[4], tex_index_arr[1], 1, 1, index),
                    Vertex::new(positions[5], tex_index_arr[1], 0, 1, index),
                    Vertex::new(positions[6], tex_index_arr[1], 1, 0, index),
                    Vertex::new(positions[7], tex_index_arr[1], 0, 0, index),
                ].to_vec()
            },
            BlockSide::LEFT => {
                [
                    Vertex::new(positions[0], tex_index_arr[2], 1, 1, index),
                    Vertex::new(positions[2], tex_index_arr[2], 1, 0, index),
                    Vertex::new(positions[4], tex_index_arr[2], 0, 1, index),
                    Vertex::new(positions[6], tex_index_arr[2], 0, 0, index),
                ].to_vec()
            },
            BlockSide::RIGHT => {
                [
                    Vertex::new(positions[1], tex_index_arr[3], 0, 1, index),
                    Vertex::new(positions[3], tex_index_arr[3], 0, 0, index),
                    Vertex::new(positions[5], tex_index_arr[3], 1, 1, index),
                    Vertex::new(positions[7], tex_index_arr[3], 1, 0, index),
                ].to_vec()
            },
            BlockSide::TOP => {
                [
                    Vertex::new(positions[2], tex_index_arr[4], 0, 1, index),
                    Vertex::new(positions[3], tex_index_arr[4], 1, 1, index),
                    Vertex::new(positions[6], tex_index_arr[4], 0, 0, index),
                    Vertex::new(positions[7], tex_index_arr[4], 1, 0, index),
                ].to_vec()
            },
            BlockSide::BOTTOM => {
                [
                    Vertex::new(positions[0], tex_index_arr[5], 0, 0, index),
                    Vertex::new(positions[1], tex_index_arr[5], 1, 0, index),
                    Vertex::new(positions[4], tex_index_arr[5], 0, 1, index),
                    Vertex::new(positions[5], tex_index_arr[5], 1, 1, index),
                ].to_vec()
            }
        }
    }

    pub fn generate_occlusion_cube(chunk_position: &WorldPosition, chunk_index: u32) -> Mesh {
        let front_vertices = Self::generate_occlusion_cube_side(chunk_position, chunk_index, BlockSide::FRONT);
        let front_indices = Self::generate_occlusion_cube_indices_side(BlockSide::FRONT, 0);
        let front_len = front_vertices.len();

        let back_vertices = Self::generate_occlusion_cube_side(chunk_position, chunk_index, BlockSide::BACK);
        let back_indices = Self::generate_occlusion_cube_indices_side(BlockSide::BACK, 1);
        let back_len = back_vertices.len();

        let left_vertices = Self::generate_occlusion_cube_side(chunk_position, chunk_index, BlockSide::LEFT);
        let left_indices = Self::generate_occlusion_cube_indices_side(BlockSide::LEFT, 2);
        let left_len = left_vertices.len();

        let right_vertices = Self::generate_occlusion_cube_side(chunk_position, chunk_index, BlockSide::RIGHT);
        let right_indices = Self::generate_occlusion_cube_indices_side(BlockSide::RIGHT, 3);
        let right_len = right_vertices.len();

        let top_vertices = Self::generate_occlusion_cube_side(chunk_position, chunk_index, BlockSide::TOP);
        let top_indices = Self::generate_occlusion_cube_indices_side(BlockSide::TOP, 4);
        let top_len = top_vertices.len();

        let bottom_vertices = Self::generate_occlusion_cube_side(chunk_position, chunk_index, BlockSide::BOTTOM);
        let bottom_indices = Self::generate_occlusion_cube_indices_side(BlockSide::BOTTOM, 5);
        let bottom_len = bottom_vertices.len();
        Mesh {
            front: ( front_vertices, front_indices, front_len as u32 ),
            back: ( back_vertices, back_indices, back_len as u32 ),
            left: ( left_vertices, left_indices, left_len as u32 ),
            right: ( right_vertices, right_indices, right_len as u32 ),
            top: ( top_vertices, top_indices, top_len as u32 ),
            bottom: ( bottom_vertices, bottom_indices, bottom_len as u32 ),
        }
    }
}