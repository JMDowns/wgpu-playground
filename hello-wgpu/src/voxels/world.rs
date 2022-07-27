use std::collections::HashMap;

use fundamentals::enums::block_type::BlockType;
use super::chunk::Chunk;
use super::position::Position;
use fundamentals::texture_coords::TextureCoordinates;
use crate::voxels::vertex::Vertex;
use wgpu::util::DeviceExt;
use fundamentals::consts::{TEXTURE_HEIGHT, TEXTURE_WIDTH};

pub struct World {
    chunks: HashMap<Position, Chunk>,
}

impl World {
    pub fn new(radius: i32) -> Self {

        let mut chunks = HashMap::new();
        for x in -radius..radius+1 {
            for y in -radius..radius+1 {
                for z in -radius..radius+1{
                    let pos = Position::new(x,y,z);
                    chunks.insert(pos, Chunk::random(&pos));
                }
            }
        }

        World { chunks }
    }
    pub fn generate_vi_vecs_at(&self, pos: &Position) -> (Vec<Vertex>, Vec<u32>, u32) {

        let mut num_of_cube = 0;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for k in 0..16 {
            for j in 0..16 {
                for i in 0..16 {
                    let block = self.chunks[&pos].get_block_at(i, j, k);
                    if block.get_block_type() != BlockType::AIR {
                        vertices = [
                            vertices,
                            generate_cube(self.chunks[&pos].get_absolute_coords_usize(i, j, k), block.get_texture_coords())
                        ].concat();
                        indices = [
                            indices,
                            generate_cube_indices(num_of_cube)
                        ].concat();
                        num_of_cube += 1;
                    }
                    
                }
            }
        }

        let indices_length = indices.len() as u32;

        (vertices, indices, indices_length)
    }
}

fn generate_cube(pos: Position, tex_coords_arr: &[TextureCoordinates; 6]) -> Vec<Vertex> {
    let positions = pos.generate_positions();
    [
        Vertex::new(positions[0], tex_coords_arr[0].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[1], tex_coords_arr[0].offset(0.0, 0.0)),
        Vertex::new(positions[2], tex_coords_arr[0].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[3], tex_coords_arr[0].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),

        Vertex::new(positions[4], tex_coords_arr[1].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[5], tex_coords_arr[1].offset(0.0, 0.0)),
        Vertex::new(positions[6], tex_coords_arr[1].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[7], tex_coords_arr[1].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),

        Vertex::new(positions[0], tex_coords_arr[2].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[3], tex_coords_arr[2].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        Vertex::new(positions[4], tex_coords_arr[2].offset(0.0, 0.0)),
        Vertex::new(positions[7], tex_coords_arr[2].offset(0.0, TEXTURE_HEIGHT)),

        Vertex::new(positions[1], tex_coords_arr[3].offset(0.0, 0.0)),
        Vertex::new(positions[2], tex_coords_arr[3].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[5], tex_coords_arr[3].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[6], tex_coords_arr[3].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),

        Vertex::new(positions[0], tex_coords_arr[4].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[1], tex_coords_arr[4].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        Vertex::new(positions[4], tex_coords_arr[4].offset(0.0, 0.0)),
        Vertex::new(positions[5], tex_coords_arr[4].offset(TEXTURE_WIDTH, 0.0)),

        Vertex::new(positions[2], tex_coords_arr[5].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[3], tex_coords_arr[5].offset(0.0, 0.0)),
        Vertex::new(positions[6], tex_coords_arr[5].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        Vertex::new(positions[7], tex_coords_arr[5].offset(0.0, TEXTURE_HEIGHT)),
    ].to_vec()
}

fn generate_cube_indices(num_of_cube: u32) -> Vec<u32> {
    [
        // Front Face
        0+24*num_of_cube,1+24*num_of_cube,2+24*num_of_cube,
        0+24*num_of_cube,2+24*num_of_cube,3+24*num_of_cube,
        // Back Face
        5+24*num_of_cube,4+24*num_of_cube,7+24*num_of_cube,
        5+24*num_of_cube,7+24*num_of_cube,6+24*num_of_cube,
        // Left Face
        10+24*num_of_cube,8+24*num_of_cube,9+24*num_of_cube,
        10+24*num_of_cube,9+24*num_of_cube,11+24*num_of_cube,
        // Rnight Face
        12+24*num_of_cube,14+24*num_of_cube,15+24*num_of_cube,
        12+24*num_of_cube,15+24*num_of_cube,13+24*num_of_cube,
        // Top Face
        19+24*num_of_cube,17+24*num_of_cube,16+24*num_of_cube,
        19+24*num_of_cube,16+24*num_of_cube,18+24*num_of_cube,
        // Bottom Face
        20+24*num_of_cube,22+24*num_of_cube,23+24*num_of_cube,
        20+24*num_of_cube,23+24*num_of_cube,21+24*num_of_cube,
    ].to_vec()
}