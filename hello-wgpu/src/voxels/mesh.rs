use crate::voxels::vertex::Vertex;
use wgpu::util::DeviceExt;
use fundamentals::texture_coords::TextureCoordinates;
use super::position::Position;
use fundamentals::consts::{TEXTURE_HEIGHT, TEXTURE_WIDTH};
use fundamentals::enums::block_type::BlockType;
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

    pub fn stupid(chunk: &Chunk) -> Self {

        let mut mesh = Mesh::new();

        for k in 0..16 {
            for j in 0..16 {
                for i in 0..16 {
                    let block = chunk.get_block_at(i, j, k);
                    if block.get_block_type() != BlockType::AIR {
                        mesh.add_vertices(
                            generate_cube(chunk.get_absolute_coords_usize(i, j, k), block.get_texture_coords()), 
                            generate_cube_indices()
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

    pub fn add_mesh(&mut self, mut mesh: Mesh) {
        mesh.front.1 = mesh.front.1.iter().map(|v| v+self.front.0.len() as u32).collect();
        mesh.back.1 = mesh.back.1.iter().map(|v| v+self.back.0.len() as u32).collect();
        mesh.left.1 = mesh.left.1.iter().map(|v| v+self.left.0.len() as u32).collect();
        mesh.right.1 = mesh.right.1.iter().map(|v| v+self.right.0.len() as u32).collect();
        mesh.top.1 = mesh.top.1.iter().map(|v| v+self.top.0.len() as u32).collect();
        mesh.bottom.1 = mesh.bottom.1.iter().map(|v| v+self.bottom.0.len() as u32).collect();

        self.front.0.append(&mut mesh.front.0);
        self.back.0.append(&mut mesh.back.0);
        self.left.0.append(&mut mesh.left.0);
        self.right.0.append(&mut mesh.right.0);
        self.top.0.append(&mut mesh.top.0);
        self.bottom.0.append(&mut mesh.bottom.0);

        self.front.1.append(&mut mesh.front.1);
        self.back.1.append(&mut mesh.back.1);
        self.left.1.append(&mut mesh.left.1);
        self.right.1.append(&mut mesh.right.1);
        self.top.1.append(&mut mesh.top.1);
        self.bottom.1.append(&mut mesh.bottom.1);

        self.front.2 = self.front.1.len() as u32;
        self.back.2 = self.back.1.len() as u32;
        self.left.2 = self.left.1.len() as u32;
        self.right.2 = self.right.1.len() as u32;
        self.top.2 = self.top.1.len() as u32;
        self.bottom.2 = self.bottom.1.len() as u32;
    }

    pub fn get_vertex_buffers(&self, device: &wgpu::Device) -> [wgpu::Buffer; 6] {
        [
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Front mesh"),
                    contents: bytemuck::cast_slice(&self.front.0),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Back mesh"),
                    contents: bytemuck::cast_slice(&self.back.0),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Left mesh"),
                    contents: bytemuck::cast_slice(&self.left.0),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Right mesh"),
                    contents: bytemuck::cast_slice(&self.right.0),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Top mesh"),
                    contents: bytemuck::cast_slice(&self.top.0),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Bottom mesh"),
                    contents: bytemuck::cast_slice(&self.bottom.0),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
        ]
    }

    pub fn get_index_buffers(&self, device: &wgpu::Device) -> [wgpu::Buffer; 6] {
        [
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Front Mesh"),
                    contents: bytemuck::cast_slice(&self.front.1),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Back Mesh"),
                    contents: bytemuck::cast_slice(&self.back.1),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Left Mesh"),
                    contents: bytemuck::cast_slice(&self.left.1),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Right Mesh"),
                    contents: bytemuck::cast_slice(&self.right.1),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Top Mesh"),
                    contents: bytemuck::cast_slice(&self.top.1),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Bottom Mesh"),
                    contents: bytemuck::cast_slice(&self.bottom.1),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
        ]
    }

    pub fn get_index_buffers_lengths(&self) -> [u32;6] {
        [
            self.front.2,
            self.back.2,
            self.left.2,
            self.right.2,
            self.top.2,
            self.bottom.2,
        ]
    }
}

pub fn generate_cube(pos: Position, tex_coords_arr: &[TextureCoordinates; 6]) -> [Vec<Vertex>; 6] {
    let positions = pos.generate_positions();
    [
        [
        Vertex::new(positions[0], tex_coords_arr[0].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[1], tex_coords_arr[0].offset(0.0, 0.0)),
        Vertex::new(positions[2], tex_coords_arr[0].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[3], tex_coords_arr[0].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        ].to_vec(),
        [
        Vertex::new(positions[4], tex_coords_arr[1].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[5], tex_coords_arr[1].offset(0.0, 0.0)),
        Vertex::new(positions[6], tex_coords_arr[1].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[7], tex_coords_arr[1].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        ].to_vec(),
        [
        Vertex::new(positions[0], tex_coords_arr[2].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[3], tex_coords_arr[2].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        Vertex::new(positions[4], tex_coords_arr[2].offset(0.0, 0.0)),
        Vertex::new(positions[7], tex_coords_arr[2].offset(0.0, TEXTURE_HEIGHT)),
        ].to_vec(),
        [
        Vertex::new(positions[1], tex_coords_arr[3].offset(0.0, 0.0)),
        Vertex::new(positions[2], tex_coords_arr[3].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[5], tex_coords_arr[3].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[6], tex_coords_arr[3].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        ].to_vec(),
        [
        Vertex::new(positions[0], tex_coords_arr[4].offset(0.0, TEXTURE_HEIGHT)),
        Vertex::new(positions[1], tex_coords_arr[4].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        Vertex::new(positions[4], tex_coords_arr[4].offset(0.0, 0.0)),
        Vertex::new(positions[5], tex_coords_arr[4].offset(TEXTURE_WIDTH, 0.0)),
        ].to_vec(),
        [
        Vertex::new(positions[2], tex_coords_arr[5].offset(TEXTURE_WIDTH, 0.0)),
        Vertex::new(positions[3], tex_coords_arr[5].offset(0.0, 0.0)),
        Vertex::new(positions[6], tex_coords_arr[5].offset(TEXTURE_WIDTH, TEXTURE_HEIGHT)),
        Vertex::new(positions[7], tex_coords_arr[5].offset(0.0, TEXTURE_HEIGHT)),
        ].to_vec()
    ]
}

pub fn generate_cube_indices() -> [Vec<u32>;6] {
    [
        // Front Face
        [
        0,1,2,
        0,2,3,
        ].to_vec(),
        // Back Face
        [
        1,0,3,
        1,3,2,
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
        3,1,0,
        3,0,2,
        ].to_vec(),
        // Bottom Face
        [
        0,2,3,
        0,3,1,
        ].to_vec(),
    ]
}