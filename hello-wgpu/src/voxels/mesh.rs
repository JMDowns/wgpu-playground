use derivables::vertex::Vertex;
use fundamentals::world_position::WorldPosition;
use fundamentals::consts::CHUNK_DIMENSION;
use super::chunk::{Chunk, ChunkBlockIterator};

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

    pub fn cull_ambient_occlusion(chunk: &Chunk, solid_data: Vec<Vec<Vec<bool>>>, index: u32) -> Self {
        let mut mesh = Mesh::new();

        let mut cbi = ChunkBlockIterator::new(chunk);

        while let Some(((i,j,k), block)) = cbi.get_next_block() {
            let adjacent_blocks_data = Self::generate_adjacent_blocks(&solid_data, i+1, j+1, k+1);
            mesh.add_vertices(
                Self::generate_cube(WorldPosition::new(i as i32,j as i32,k as i32), block.get_texture_indices(), &adjacent_blocks_data, index), 
                Self::generate_cube_indices(&adjacent_blocks_data)
            );
        }

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

    fn generate_adjacent_blocks(solid_data: &Vec<Vec<Vec<bool>>>, i: usize, j: usize, k: usize) -> [bool; 6] {
        let mut adjacency_data = [false;6];
        adjacency_data[0] = solid_data[i-1][j][k];
        adjacency_data[1] = solid_data[i+1][j][k];
        adjacency_data[2] = solid_data[i][j][k-1];
        adjacency_data[3] = solid_data[i][j][k+1];
        adjacency_data[4] = solid_data[i][j+1][k];
        adjacency_data[5] = solid_data[i][j-1][k];
        adjacency_data
    }
    
    fn generate_cube(pos: WorldPosition, tex_index_arr: &[usize; 6], adjacent_blocks_data: &[bool;6], index: u32) -> [Vec<Vertex>; 6] {
        let positions = pos.generate_vertex_world_positions();
        let mut vertices_arr = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),];
        let all_vertices_arr = 
        [
            //Front
            [
            Vertex::new(positions[0], tex_index_arr[0], 1, index),
            Vertex::new(positions[1], tex_index_arr[0], 3, index),
            Vertex::new(positions[2], tex_index_arr[0], 0, index),
            Vertex::new(positions[3], tex_index_arr[0], 2, index),
            ].to_vec(),
            //Back
            [
            Vertex::new(positions[4], tex_index_arr[1], 3, index),
            Vertex::new(positions[5], tex_index_arr[1], 1, index),
            Vertex::new(positions[6], tex_index_arr[1], 2, index),
            Vertex::new(positions[7], tex_index_arr[1], 0, index),
            ].to_vec(),
            //Left
            [
            Vertex::new(positions[0], tex_index_arr[2], 3, index),
            Vertex::new(positions[2], tex_index_arr[2], 2, index),
            Vertex::new(positions[4], tex_index_arr[2], 1, index),
            Vertex::new(positions[6], tex_index_arr[2], 0, index),
            ].to_vec(),
            //Right
            [
            Vertex::new(positions[1], tex_index_arr[3], 1, index),
            Vertex::new(positions[3], tex_index_arr[3], 0, index),
            Vertex::new(positions[5], tex_index_arr[3], 3, index),
            Vertex::new(positions[7], tex_index_arr[3], 2, index),
            ].to_vec(),
            //Top
            [
            Vertex::new(positions[2], tex_index_arr[4], 1, index),
            Vertex::new(positions[3], tex_index_arr[4], 3, index),
            Vertex::new(positions[6], tex_index_arr[4], 0, index),
            Vertex::new(positions[7], tex_index_arr[4], 2, index),
            ].to_vec(),
            //Bottom
            [
            Vertex::new(positions[0], tex_index_arr[5], 0, index),
            Vertex::new(positions[1], tex_index_arr[5], 2, index),
            Vertex::new(positions[4], tex_index_arr[5], 1, index),
            Vertex::new(positions[5], tex_index_arr[5], 3, index),
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
}

