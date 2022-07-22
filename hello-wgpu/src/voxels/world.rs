use std::collections::HashMap;

use fundamentals::enums::block_type::BlockType;
use super::chunk::Chunk;
use super::position::Position;
use crate::voxels::vertex::Vertex;
use wgpu::util::DeviceExt;

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
    pub fn generate_vi_buffers_at(&self, pos: &Position, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32) {

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
                            generate_cube(self.chunks[&pos].get_absolute_coords_usize(i, j, k), block.get_color())
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

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        (vertex_buffer, index_buffer, indices.len() as u32)
    }
}

fn generate_cube(pos: Position, wcolor: wgpu::Color) -> Vec<Vertex> {
    [
        Vertex::new(Position { x: pos.x, y: pos.y, z: pos.z}, wcolor),
        Vertex::new(Position { x: pos.x-1, y: pos.y, z: pos.z}, wcolor),
        Vertex::new(Position { x: pos.x-1, y: pos.y-1, z: pos.z}, wcolor),
        Vertex::new(Position { x: pos.x, y: pos.y-1, z: pos.z}, wcolor),
        Vertex::new(Position { x: pos.x, y: pos.y, z: pos.z-1}, wcolor),
        Vertex::new(Position { x: pos.x-1, y: pos.y, z: pos.z-1}, wcolor),
        Vertex::new(Position { x: pos.x-1, y: pos.y-1, z: pos.z-1}, wcolor),
        Vertex::new(Position { x: pos.x, y: pos.y-1, z: pos.z-1}, wcolor),
    ].to_vec()
}

fn generate_cube_indices(num_of_cube: u32) -> Vec<u32> {
    [
        // Front Face
        0+8*num_of_cube,1+8*num_of_cube,2+8*num_of_cube,
        0+8*num_of_cube,2+8*num_of_cube,3+8*num_of_cube,
        // Back Face
        5+8*num_of_cube,4+8*num_of_cube,7+8*num_of_cube,
        5+8*num_of_cube,7+8*num_of_cube,6+8*num_of_cube,
        // Rnum_of_cubeght Face
        1+8*num_of_cube,5+8*num_of_cube,6+8*num_of_cube,
        1+8*num_of_cube,6+8*num_of_cube,2+8*num_of_cube,
        // Left Face
        4+8*num_of_cube,0+8*num_of_cube,3+8*num_of_cube,
        4+8*num_of_cube,3+8*num_of_cube,7+8*num_of_cube,
        // Bottom Face
        2+8*num_of_cube,6+8*num_of_cube,7+8*num_of_cube,
        2+8*num_of_cube,7+8*num_of_cube,3+8*num_of_cube,
        // Bottom Face
        5+8*num_of_cube,1+8*num_of_cube,0+8*num_of_cube,
        5+8*num_of_cube,0+8*num_of_cube,4+8*num_of_cube,
    ].to_vec()
}