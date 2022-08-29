use std::collections::HashMap;

use crate::gpu_data::{vertex_gpu_data::VertexGPUData, vec_vertex_index_length_triple::VecVertexIndexLengthsTriple};
use derivables::vertex::Vertex;
use fundamentals::texture_coords::TextureCoordinates;
use fundamentals::world_position::WorldPosition;
use fundamentals::consts::CHUNK_DIMENSION;
use wgpu::util::DeviceExt;
use super::chunk::Chunk;

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

    pub fn generate_vertex_buffer(vertex_vec: &Vec<Vertex>, device: &wgpu::Device, label: &str) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&String::from(format!("{} Vertex", label))),
                contents: bytemuck::cast_slice(vertex_vec),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    }

    pub fn generate_index_buffer(index_vec: &Vec<u32>, device: &wgpu::Device, label: &str) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&String::from(format!("{} Index", label))),
                contents: bytemuck::cast_slice(index_vec),
                usage: wgpu::BufferUsages::INDEX,
            }
        )
    }

    pub fn cull_ambient_occlusion(chunk: &Chunk, solid_data: [[[bool; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2], index: u32) -> Self {

        let mut mesh = Mesh::new();

        for k in 0..CHUNK_DIMENSION as usize {
            for j in 0..CHUNK_DIMENSION as usize {
                for i in 0..CHUNK_DIMENSION as usize {
                    let block = chunk.get_block_at(i, j, k);
                    let ambient_occlusion_on_vertices = Self::generate_ambient_occlusion_on_vertices(&solid_data, i+1, j+1, k+1);
                    let adjacent_blocks_data = Self::generate_adjacent_blocks(&solid_data, i+1, j+1, k+1);
                    if !block.is_air() {
                        mesh.add_vertices(
                            Self::generate_cube(WorldPosition::new(i as i32,j as i32,k as i32), block.get_texture_coords(), &ambient_occlusion_on_vertices, &adjacent_blocks_data, index), 
                            Self::generate_cube_indices(&adjacent_blocks_data, &ambient_occlusion_on_vertices)
                        );
                    }
                    
                }
            }
        }

        mesh
    }

    pub fn add_vertices(&mut self, mut block_vertices: [Vec<Vertex>; 6], block_indices: [Vec<u32>; 6]) {
        self.front.1.append(&mut block_indices[0].iter().map(|e| e+self.front.0.len() as u32).collect());
        self.back.1.append(&mut block_indices[1].iter().map(|e| e+self.back.0.len() as u32).collect());
        self.left.1.append(&mut block_indices[2].iter().map(|e| e+self.left.0.len() as u32).collect());
        self.right.1.append(&mut block_indices[3].iter().map(|e| e+self.right.0.len() as u32).collect());
        self.top.1.append(&mut block_indices[4].iter().map(|e| e+self.top.0.len() as u32).collect());
        self.bottom.1.append(&mut block_indices[5].iter().map(|e| e+self.bottom.0.len() as u32).collect());

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

    fn generate_adjacent_blocks(solid_data: &[[[bool; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2], i: usize, j: usize, k: usize) -> [bool; 6] {
        let mut adjacency_data = [false;6];
        adjacency_data[0] = solid_data[i-1][j][k];
        adjacency_data[1] = solid_data[i+1][j][k];
        adjacency_data[2] = solid_data[i][j][k-1];
        adjacency_data[3] = solid_data[i][j][k+1];
        adjacency_data[4] = solid_data[i][j+1][k];
        adjacency_data[5] = solid_data[i][j-1][k];
        adjacency_data
    }
    
    fn generate_ambient_occlusion_on_vertices(solid_data: &[[[bool; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2]; CHUNK_DIMENSION as usize+2], i: usize, j: usize, k: usize) -> [[u8;3];8] {
        [
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j][k-1], solid_data[i-1][j-1][k], solid_data[i-1][j-1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j-1][k-1], solid_data[i-1][j][k-1], solid_data[i-1][j-1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j-1][k], solid_data[i][j-1][k-1], solid_data[i-1][j-1][k-1]),
            ],
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j][k+1], solid_data[i-1][j-1][k], solid_data[i-1][j-1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j-1][k+1], solid_data[i-1][j][k+1], solid_data[i-1][j-1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j-1][k], solid_data[i][j-1][k+1], solid_data[i-1][j-1][k+1]),
            ],
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j][k-1], solid_data[i-1][j+1][k], solid_data[i-1][j+1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j+1][k-1], solid_data[i-1][j][k-1], solid_data[i-1][j+1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j+1][k], solid_data[i][j+1][k-1], solid_data[i-1][j+1][k-1]),
            ],
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j][k+1], solid_data[i-1][j+1][k], solid_data[i-1][j+1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j+1][k+1], solid_data[i-1][j][k+1], solid_data[i-1][j+1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i-1][j+1][k], solid_data[i][j+1][k+1], solid_data[i-1][j+1][k+1]),
            ],
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j][k-1], solid_data[i+1][j-1][k], solid_data[i+1][j-1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j-1][k-1], solid_data[i+1][j][k-1], solid_data[i+1][j-1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j-1][k], solid_data[i][j-1][k-1], solid_data[i+1][j-1][k-1]),
            ],
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j][k+1], solid_data[i+1][j-1][k], solid_data[i+1][j-1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j-1][k+1], solid_data[i+1][j][k+1], solid_data[i+1][j-1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j-1][k], solid_data[i][j-1][k+1], solid_data[i+1][j-1][k+1]),
            ],
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j][k-1], solid_data[i+1][j+1][k], solid_data[i+1][j+1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j+1][k-1], solid_data[i+1][j][k-1], solid_data[i+1][j+1][k-1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j+1][k], solid_data[i][j+1][k-1], solid_data[i+1][j+1][k-1]),
            ],
            [
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j][k+1], solid_data[i+1][j+1][k], solid_data[i+1][j+1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i][j+1][k+1], solid_data[i+1][j][k+1], solid_data[i+1][j+1][k+1]),
                Self::generate_ambient_occlusion_for_vertex(solid_data[i+1][j+1][k], solid_data[i][j+1][k+1], solid_data[i+1][j+1][k+1]),
            ],
        ]
    }
    
    fn generate_ambient_occlusion_for_vertex(is_side_1_solid: bool, is_side_2_solid: bool, is_corner_solid: bool) -> u8 {
        if is_side_1_solid && is_side_2_solid {
            return 3;
        }
    
        return is_side_1_solid as u8 + is_side_2_solid as u8 + is_corner_solid as u8;
    }
    
    fn generate_cube(pos: WorldPosition, tex_coords_arr: &[TextureCoordinates; 6], ambient_occlusion_on_vertices: &[[u8;3]; 8], adjacent_blocks_data: &[bool;6], index: u32) -> [Vec<Vertex>; 6] {
        let positions = pos.generate_vertex_world_positions();
        let mut vertices_arr = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),];
        let all_vertices_arr = 
        [
            [
            Vertex::new(positions[0], tex_coords_arr[0].offset(0, 1), ambient_occlusion_on_vertices[0][0], index),
            Vertex::new(positions[1], tex_coords_arr[0].offset(1, 1), ambient_occlusion_on_vertices[1][0], index),
            Vertex::new(positions[2], tex_coords_arr[0].offset(0, 0), ambient_occlusion_on_vertices[2][0], index),
            Vertex::new(positions[3], tex_coords_arr[0].offset(1, 0), ambient_occlusion_on_vertices[3][0], index),
            ].to_vec(),
            [
            Vertex::new(positions[4], tex_coords_arr[1].offset(1, 1), ambient_occlusion_on_vertices[4][0], index),
            Vertex::new(positions[5], tex_coords_arr[1].offset(0, 1), ambient_occlusion_on_vertices[5][0], index),
            Vertex::new(positions[6], tex_coords_arr[1].offset(1, 0), ambient_occlusion_on_vertices[6][0], index),
            Vertex::new(positions[7], tex_coords_arr[1].offset(0, 0), ambient_occlusion_on_vertices[7][0], index),
            ].to_vec(),
            [
            Vertex::new(positions[0], tex_coords_arr[2].offset(1, 1), ambient_occlusion_on_vertices[0][1], index),
            Vertex::new(positions[2], tex_coords_arr[2].offset(1, 0), ambient_occlusion_on_vertices[2][1], index),
            Vertex::new(positions[4], tex_coords_arr[2].offset(0, 1), ambient_occlusion_on_vertices[4][1], index),
            Vertex::new(positions[6], tex_coords_arr[2].offset(0, 0), ambient_occlusion_on_vertices[6][1], index),
            ].to_vec(),
            [
            Vertex::new(positions[1], tex_coords_arr[3].offset(0, 1), ambient_occlusion_on_vertices[1][1], index),
            Vertex::new(positions[3], tex_coords_arr[3].offset(0, 0), ambient_occlusion_on_vertices[3][1], index),
            Vertex::new(positions[5], tex_coords_arr[3].offset(1, 1), ambient_occlusion_on_vertices[5][1], index),
            Vertex::new(positions[7], tex_coords_arr[3].offset(1, 0), ambient_occlusion_on_vertices[7][1], index),
            ].to_vec(),
            [
            Vertex::new(positions[2], tex_coords_arr[4].offset(0, 1), ambient_occlusion_on_vertices[2][2], index),
            Vertex::new(positions[3], tex_coords_arr[4].offset(1, 1), ambient_occlusion_on_vertices[3][2], index),
            Vertex::new(positions[6], tex_coords_arr[4].offset(0, 0), ambient_occlusion_on_vertices[6][2], index),
            Vertex::new(positions[7], tex_coords_arr[4].offset(1, 0), ambient_occlusion_on_vertices[7][2], index),
            ].to_vec(),
            [
            Vertex::new(positions[0], tex_coords_arr[5].offset(0, 0), ambient_occlusion_on_vertices[0][2], index),
            Vertex::new(positions[1], tex_coords_arr[5].offset(1, 0), ambient_occlusion_on_vertices[1][2], index),
            Vertex::new(positions[4], tex_coords_arr[5].offset(0, 1), ambient_occlusion_on_vertices[4][2], index),
            Vertex::new(positions[5], tex_coords_arr[5].offset(1, 1), ambient_occlusion_on_vertices[5][2], index),
            ].to_vec()
        ];
    
        for i in 0..6 {
            if !adjacent_blocks_data[i] {
                vertices_arr[i] = all_vertices_arr[i].clone();
            }
        }
    
        vertices_arr
    }
    
    fn generate_cube_indices(adjacent_blocks_data: &[bool;6], ambient_occlusion_on_vertices: &[[u8;3]; 8]) -> [Vec<u32>;6] {
        let mut indices_arr = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),];
        let all_indices_arr = 
        [
            // Front Face
            match ambient_occlusion_on_vertices[0][0] + ambient_occlusion_on_vertices[3][0] <= ambient_occlusion_on_vertices[1][0] + ambient_occlusion_on_vertices[2][0] {
                true => [
                    0,1,3,
                    0,3,2,
                    ].to_vec(),
                false => [
                    0,1,2,
                    1,3,2,
                    ].to_vec(),
            },
            // Back Face
            match ambient_occlusion_on_vertices[5][0] + ambient_occlusion_on_vertices[6][0] <= ambient_occlusion_on_vertices[4][0] + ambient_occlusion_on_vertices[7][0] {
                true => 
                [    
                1,0,2,
                1,2,3,
                ].to_vec(),
                false => 
                [    
                1,0,3,
                0,2,3,
                ].to_vec(),
            },
            // Left Face
            match ambient_occlusion_on_vertices[4][1] + ambient_occlusion_on_vertices[2][1] <= ambient_occlusion_on_vertices[0][1] + ambient_occlusion_on_vertices[6][1] {
                true => 
                [
                2,0,1,
                2,1,3,
                ].to_vec(),
                false => 
                [
                2,0,3,
                0,1,3,
                ].to_vec(),
            },
            // Right Face
            match ambient_occlusion_on_vertices[1][1] + ambient_occlusion_on_vertices[7][1] <= ambient_occlusion_on_vertices[3][1] + ambient_occlusion_on_vertices[5][1] {
                true => 
                [
                0,2,3,
                0,3,1,
                ].to_vec(),
                false => 
                [
                0,2,1,
                2,3,1,
                ].to_vec(),
            },
            // Top Face
            match ambient_occlusion_on_vertices[2][2] + ambient_occlusion_on_vertices[7][2] <= ambient_occlusion_on_vertices[3][2] + ambient_occlusion_on_vertices[6][2] {
                true => 
                [
                0,1,3,
                0,3,2,
                ].to_vec(),
                false => 
                [
                0,1,2,
                1,3,2,
                ].to_vec(),
            },
            // Bottom Face
            match ambient_occlusion_on_vertices[4][2] + ambient_occlusion_on_vertices[1][2] <= ambient_occlusion_on_vertices[0][2] + ambient_occlusion_on_vertices[5][2] {
                true => 
                [
                1,0,2,
                1,2,3
                ].to_vec(),
                false => 
                [
                1,0,3,
                0,2,3
                ].to_vec(),
            },
        ];
    
        for i in 0..6 {
            if !adjacent_blocks_data[i] {
                indices_arr[i] = all_indices_arr[i].clone();
            }
        }
    
        indices_arr
    }
}

