use derivables::{vertex::Vertex, block::Block};
use fundamentals::{world_position::WorldPosition, enums::{block_side::BlockSide, block_type::BlockType}, consts::{CHUNK_DIMENSION, NUM_VERTICES_IN_BUCKET}};
use super::chunk::{Chunk, ChunkBlockIterator};

use strum::IntoEnumIterator;


#[derive(Debug, Clone, Copy, PartialEq)]
struct Face {
    pub lr: (usize, usize, usize),
    pub ll: (usize, usize, usize),
    pub ur: (usize, usize, usize),
    pub ul: (usize, usize, usize),
    pub block_type_int: usize,
    pub block_side: BlockSide
}

impl Face {
    pub fn new(i: usize, j: usize, k: usize, block_type_int: usize, block_side: BlockSide) -> Self {
        match block_side {
            BlockSide::FRONT => {
                Face {
                    ll: (i, j, k),
                    lr: (i, j, k+1),
                    ul: (i, j+1, k),
                    ur: (i, j+1, k+1),
                    block_type_int,
                    block_side
                }
            },

            BlockSide::BACK => {
                Face {
                    ll: (i+1, j, k+1),
                    lr: (i+1, j, k),
                    ul: (i+1, j+1, k+1),
                    ur: (i+1, j+1, k),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::LEFT => {
                Face {
                    ll: (i+1, j, k),
                    lr: (i, j, k),
                    ul: (i+1, j+1, k),
                    ur: (i, j+1, k),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::RIGHT => {
                Face {
                    ll: (i, j, k+1),
                    lr: (i+1, j, k+1),
                    ul: (i, j+1, k+1),
                    ur: (i+1, j+1, k+1),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::TOP => {
                Face {
                    ll: (i, j+1, k),
                    lr: (i, j+1, k+1),
                    ul: (i+1, j+1, k),
                    ur: (i+1, j+1, k+1),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::BOTTOM => {
                Face {
                    ll: (i, j, k+1),
                    lr: (i, j, k),
                    ul: (i+1, j, k+1),
                    ur: (i+1, j, k),
                    block_type_int,
                    block_side
                }
            }
        }
        
    }

    fn merge_up(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.ul == other.ll && self.ur == other.lr {
            return Some(Face {
                ul: other.ul,
                ur: other.ur,
                ll: self.ll,
                lr: self.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int
            });
        }

        None
    }

    fn merge_right(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.lr == other.ll && self.ur == other.ul {
            return Some(Face {
                ul: self.ul,
                ur: other.ur,
                ll: self.ll,
                lr: other.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int
            });
        }

        None
    }

    fn merge_left(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.ll == other.lr && self.ul == other.ur {
            return Some(Face {
                ul: other.ul,
                ur: self.ur,
                ll: other.ll,
                lr: self.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int
            });
        }

        None
    }
}

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
        let mut mesh = Mesh::new();

        let mut cbi = ChunkBlockIterator::new(chunk);

        while let Some(((i,j,k), block)) = cbi.get_next_block() {
            let adjacent_blocks_data = Self::generate_adjacent_blocks(&chunk, i, j, k);
            mesh.add_vertices(
                Self::generate_cube(WorldPosition::new(i as i32-1,j as i32-1,k as i32-1), block.get_texture_indices(), &adjacent_blocks_data, index), 
                Self::generate_cube_indices(&adjacent_blocks_data),
            );
        }

        mesh
    }

    pub fn cull_side(chunk: &Chunk, index: u32, side: BlockSide) -> (Vec<Vertex>, Vec<u32>, u32) {
        let mut mesh_side_vertices = Vec::new();
        let mut mesh_side_indices = Vec::new();
        let mut mesh_side_indices_count = 0;

        let mut num_faces_generated = 0;

        let mut cbi = ChunkBlockIterator::new(chunk);

        while let Some(((i,j,k), block)) = cbi.get_next_block() {
            if !Mesh::is_adjacent_blocks_solid_side(chunk, i, j, k, side) {
                mesh_side_vertices.append(&mut Self::generate_cube_side(WorldPosition::new(i as i32-1,j as i32-1,k as i32-1), block.get_texture_indices(), index, side));
                let mut index_vec = Self::generate_cube_indices_side(side, num_faces_generated);
                mesh_side_indices_count += index_vec.len() as u32;
                mesh_side_indices.append(&mut index_vec);
                num_faces_generated += 1;
            }
            
        }

        (mesh_side_vertices, mesh_side_indices, mesh_side_indices_count)
    }

    fn get_boundary_from_face(face: &Face) -> usize {
        match face.block_side {
            BlockSide::FRONT => {
                face.ul.1
            }
            BlockSide::BACK => {
                face.ul.1
            }
            BlockSide::LEFT => {
                face.ul.0
            }
            BlockSide::RIGHT => {
                face.ur.0
            }
            BlockSide::TOP => {
                face.ur.0
            }
            BlockSide::BOTTOM => {
                face.ur.0
            }
        }
    }

    fn greedy_merge_and_modify_vecs(current_layer: &mut Vec<Vec<Face>>, before_layer: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>, side: BlockSide) {
        for layer_index in 0..CHUNK_DIMENSION as usize {
            match (before_layer[layer_index].len(), current_layer[layer_index].len()) {
                (0, _) => {
                    before_layer[layer_index].extend(current_layer[layer_index].drain(..));
                }
                (_, 0) => {
                    for before_face in before_layer[layer_index].drain(..) {
                        faces_to_make.push(before_face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = before_layer[layer_index][before_index];
                        let current_face = current_layer[layer_index][current_index];
                        let merged_face_option = match side {
                            BlockSide::FRONT => {
                                before_face.merge_right(&current_face)
                            }
                            BlockSide::BACK => {
                                before_face.merge_left(&current_face)
                            }
                            BlockSide::LEFT => {
                                before_face.merge_up(&current_face)
                            }
                            BlockSide::RIGHT => {
                                before_face.merge_up(&current_face)
                            }
                            BlockSide::TOP => {
                                before_face.merge_right(&current_face)
                            }
                            BlockSide::BOTTOM => {
                                before_face.merge_left(&current_face)
                            }
                        };
                        if let Some(merged_face) = merged_face_option {
                            current_layer[layer_index][current_index] = merged_face;
                            before_index += 1;
                            current_index += 1;
                        } else {
                            let (before_boundary, current_boundary) = (Self::get_boundary_from_face(&before_face), Self::get_boundary_from_face(&current_face));
                            if before_boundary == current_boundary {
                                faces_to_make.push(before_layer[layer_index][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_boundary < current_boundary {
                                while (before_index < before_len && Self::get_boundary_from_face(&before_layer[layer_index][before_index]) < current_boundary) {
                                    faces_to_make.push(before_layer[layer_index][before_index]);
                                    before_index += 1;
                                }
                            } else if before_boundary > current_boundary {
                                while (current_index < current_len && Self::get_boundary_from_face(&current_layer[layer_index][current_index]) < before_boundary) {
                                    current_index += 1;
                                }
                                if current_index == current_len {
                                    faces_to_make.push(before_layer[layer_index][before_index]);
                                    before_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(before_layer[layer_index][i]);
                    }

                    before_layer[layer_index].clear();
                    before_layer[layer_index].extend(current_layer[layer_index].drain(..));
                    
                }
            }
        }
    }

    pub fn greedy(chunk: &Chunk, index: u32) -> Self {
        let mut mesh = Mesh::new();
        let mut cbi = ChunkBlockIterator::new(chunk);

        let mut side_layers = vec![vec![Vec::new(); CHUNK_DIMENSION as usize]; 6];
        let mut side_before_layers = vec![vec![Vec::new(); CHUNK_DIMENSION as usize]; 6];

        let mut faces_to_make = Vec::new();

        let mut current_x = 0;
        let mut current_y = 0;
        let mut current_z = 0;

        while let Some(((i,j,k), block)) = cbi.get_next_block() {
            if current_y < j-1 {
                Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::LEFT as usize], &mut side_before_layers[BlockSide::LEFT as usize], &mut faces_to_make, BlockSide::LEFT);
                Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::RIGHT as usize], &mut side_before_layers[BlockSide::RIGHT as usize], &mut faces_to_make, BlockSide::RIGHT);
            }
            if current_z < k-1 {
                Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::FRONT as usize], &mut side_before_layers[BlockSide::FRONT as usize], &mut faces_to_make, BlockSide::FRONT);
                Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::BACK as usize], &mut side_before_layers[BlockSide::BACK as usize], &mut faces_to_make, BlockSide::BACK);
                Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::TOP as usize], &mut side_before_layers[BlockSide::TOP as usize], &mut faces_to_make, BlockSide::TOP);
                Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::BOTTOM as usize], &mut side_before_layers[BlockSide::BOTTOM as usize], &mut faces_to_make, BlockSide::BOTTOM);
            }
            current_x = i-1;
            current_y = j-1;
            current_z = k-1;
            let adjacent_blocks_data = Self::generate_adjacent_blocks(&chunk, i, j, k);

            for side in BlockSide::iter() {
                if !adjacent_blocks_data[side as usize] {
                    let side_face = Face::new(current_x,current_y,current_z, block.block_type as usize, side);
                    let orientation_index = match side {
                        BlockSide::FRONT => {
                            current_x
                        }
                        BlockSide::BACK => {
                            current_x                            
                        }
                        BlockSide::LEFT => {
                            current_z
                        }
                        BlockSide::RIGHT => {
                            current_z
                        }
                        BlockSide::TOP => {
                            current_y
                        }
                        BlockSide::BOTTOM => {
                            current_y
                        }
                    };
                    match side_layers[side as usize][orientation_index].last() {
                        Some(face) => {
                            let merged_option = match side {
                                BlockSide::FRONT => {
                                    face.merge_up(&side_face)
                                }
                                BlockSide::BACK => {
                                    face.merge_up(&side_face)
                                }
                                BlockSide::LEFT => {
                                    face.merge_left(&side_face)
                                }
                                BlockSide::RIGHT => {
                                    face.merge_right(&side_face)
                                }
                                BlockSide::TOP => {
                                    face.merge_up(&side_face)
                                }
                                BlockSide::BOTTOM => {
                                    face.merge_up(&side_face)
                                }
                            };
                            if let Some(merged_face) = merged_option {
                                side_layers[side as usize][orientation_index].pop();
                                side_layers[side as usize][orientation_index].push(merged_face);
                            } else {
                                side_layers[side as usize][orientation_index].push(side_face);
                            }
                        },
                        None => {
                            side_layers[side as usize][orientation_index].push(side_face);
                        }
                    }
                }
            }
        }

        Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::FRONT as usize], &mut side_before_layers[BlockSide::FRONT as usize], &mut faces_to_make, BlockSide::FRONT);
        Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::BACK as usize], &mut side_before_layers[BlockSide::BACK as usize], &mut faces_to_make, BlockSide::BACK);
        Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::LEFT as usize], &mut side_before_layers[BlockSide::LEFT as usize], &mut faces_to_make, BlockSide::LEFT);
        Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::RIGHT as usize], &mut side_before_layers[BlockSide::RIGHT as usize], &mut faces_to_make, BlockSide::RIGHT);
        Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::TOP as usize], &mut side_before_layers[BlockSide::TOP as usize], &mut faces_to_make, BlockSide::TOP);
        Self::greedy_merge_and_modify_vecs(&mut side_layers[BlockSide::BOTTOM as usize], &mut side_before_layers[BlockSide::BOTTOM as usize], &mut faces_to_make, BlockSide::BOTTOM);

        for i in 0..6 {
            for face_vec in side_before_layers[i].iter() {
                for face in face_vec {
                    faces_to_make.push(*face);
                }
            }
            for face_vec in side_layers[i].iter() {
                for face in face_vec {
                    faces_to_make.push(*face);
                }
            }
        }

        let mut vertex_vec = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut index_vec = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut num_faces_generated = vec![0;6];

        for face in faces_to_make {
            let face_index = face.block_side as usize;
            vertex_vec[face_index].extend(Self::generate_face_vertices(&face, index));
            index_vec[face_index].extend(Self::generate_face_indices(&face, index, num_faces_generated[face_index]));
            num_faces_generated[face_index] += 1;
        }

        mesh.add_vertices(vertex_vec, index_vec);

        mesh
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

    fn generate_adjacent_blocks(chunk: &Chunk, i: usize, j: usize, k: usize) -> [bool; 6] {
        let mut adjacency_data = [false;6];
        adjacency_data[0] = chunk.is_block_solid(i-1, j, k);
        adjacency_data[1] = chunk.is_block_solid(i+1, j, k);
        adjacency_data[2] = chunk.is_block_solid(i, j, k-1);
        adjacency_data[3] = chunk.is_block_solid(i, j, k+1);
        adjacency_data[4] = chunk.is_block_solid(i, j+1, k);
        adjacency_data[5] = chunk.is_block_solid(i, j-1, k);
        adjacency_data
    }

    fn is_adjacent_blocks_solid_side(chunk: &Chunk, i: usize, j: usize, k: usize, side: BlockSide) -> bool {
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
        let texture_indices = &Block::get_texture_indices_from_int(face.block_type_int);
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

    fn generate_face_indices(face: &Face, index: u32, num_faces_generated: u32) -> Vec<u32> {
        [
            0+num_faces_generated*4,1+num_faces_generated*4,3+num_faces_generated*4,
            0+num_faces_generated*4,3+num_faces_generated*4,2+num_faces_generated*4,
        ].to_vec()
    }
    
    fn generate_cube(pos: WorldPosition, tex_index_arr: &[usize; 6], adjacent_blocks_data: &[bool;6], index: u32) -> [Vec<Vertex>; 6] {
        let positions = pos.generate_vertex_world_positions();
        let mut vertices_arr = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),];
        let all_vertices_arr = 
        [
            //Front
            [
            Vertex::new(positions[0], tex_index_arr[0], 0, 1, index),
            Vertex::new(positions[1], tex_index_arr[0], 1, 1, index),
            Vertex::new(positions[2], tex_index_arr[0], 0, 0, index),
            Vertex::new(positions[3], tex_index_arr[0], 1, 0, index),
            ].to_vec(),
            //Back
            [
            Vertex::new(positions[4], tex_index_arr[1], 1, 1, index),
            Vertex::new(positions[5], tex_index_arr[1], 0, 1, index),
            Vertex::new(positions[6], tex_index_arr[1], 1, 0, index),
            Vertex::new(positions[7], tex_index_arr[1], 0, 0, index),
            ].to_vec(),
            //Left
            [
            Vertex::new(positions[0], tex_index_arr[2], 1, 1, index),
            Vertex::new(positions[2], tex_index_arr[2], 1, 0, index),
            Vertex::new(positions[4], tex_index_arr[2], 0, 1, index),
            Vertex::new(positions[6], tex_index_arr[2], 0, 0, index),
            ].to_vec(),
            //Right
            [
            Vertex::new(positions[1], tex_index_arr[3], 0, 1, index),
            Vertex::new(positions[3], tex_index_arr[3], 0, 0, index),
            Vertex::new(positions[5], tex_index_arr[3], 1, 1, index),
            Vertex::new(positions[7], tex_index_arr[3], 1, 0, index),
            ].to_vec(),
            //Top
            [
            Vertex::new(positions[2], tex_index_arr[4], 0, 1, index),
            Vertex::new(positions[3], tex_index_arr[4], 1, 1, index),
            Vertex::new(positions[6], tex_index_arr[4], 0, 0, index),
            Vertex::new(positions[7], tex_index_arr[4], 1, 0, index),
            ].to_vec(),
            //Bottom
            [
            Vertex::new(positions[0], tex_index_arr[5], 0, 0, index),
            Vertex::new(positions[1], tex_index_arr[5], 1, 0, index),
            Vertex::new(positions[4], tex_index_arr[5], 0, 1, index),
            Vertex::new(positions[5], tex_index_arr[5], 1, 1, index),
            ].to_vec()
        ];
    
        for i in 0..6 {
            if !adjacent_blocks_data[i] {
                vertices_arr[i] = all_vertices_arr[i].clone();
            }
        }
    
        vertices_arr
    }
    fn generate_cube_indices(adjacent_blocks_data: &[bool;6]) -> [Vec<u32>;6] {
        let mut indices_arr = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),];
        let all_indices_arr = 
        [
            // Front Face
            [
            0,1,3,
            0,3,2,
            ].to_vec(),
                
            // Back Face
            [    
            1,0,2,
            1,2,3,
            ].to_vec(),
                
            // Left Face
            [
            2,0,1,
            2,1,3,
            ].to_vec(),
                
            // Right Face
            [
            0,2,3,
            0,3,1,
            ].to_vec(),
                
            // Top Face
            [
            0,1,3,
            0,3,2,
            ].to_vec(),
            
            // Bottom Face
            [
            1,0,2,
            1,2,3
            ].to_vec()
                
        ];
    
        for i in 0..6 {
            if !adjacent_blocks_data[i] {
                indices_arr[i] = all_indices_arr[i].clone();
            }
        }
    
        indices_arr
    }

    fn generate_cube_side(pos: WorldPosition, tex_index_arr: &[usize; 6], index: u32, side: BlockSide) -> Vec<Vertex> {
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
}