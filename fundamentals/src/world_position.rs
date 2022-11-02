use std::fmt::Display;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Hash, PartialEq, Eq, Clone, Copy, Pod, Zeroable, Debug)]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        WorldPosition { x, y, z}
    }

    pub fn from(p3: cgmath::Point3<f32>) -> Self {
        WorldPosition { x: p3.x.floor() as i32, y: p3.y.floor() as i32, z: p3.z.floor() as i32}
    }

    pub fn generate_vertex_world_positions(&self) -> [WorldPosition; 8] {
        [
            WorldPosition::new(self.x, self.y, self.z),
            WorldPosition::new(self.x, self.y, self.z+1),
            WorldPosition::new(self.x, self.y+1, self.z),
            WorldPosition::new(self.x, self.y+1, self.z+1),
            WorldPosition::new(self.x+1, self.y, self.z),
            WorldPosition::new(self.x+1, self.y, self.z+1),
            WorldPosition::new(self.x+1, self.y+1, self.z),
            WorldPosition::new(self.x+1, self.y+1, self.z+1),
        ]
    }

    pub fn generate_vertex_world_positions_front(&self) -> [WorldPosition; 4] {
        [
            WorldPosition::new(self.x, self.y, self.z),
            WorldPosition::new(self.x, self.y, self.z+1),
            WorldPosition::new(self.x, self.y+1, self.z),
            WorldPosition::new(self.x, self.y+1, self.z+1)
        ]
    }

    pub fn generate_neighborhood1_world_positions(&self) -> [[[WorldPosition; 3]; 3]; 3] {
        let mut world_positions = [[[WorldPosition::new(self.x,self.y,self.z); 3]; 3]; 3];
        for k in 0..3 as i32 {
            for j in 0..3 as i32 {
                for i in 0..3 as i32{
                    world_positions[i as usize][j as usize][k as usize] = WorldPosition::new(self.x+i-1, self.y+j-1, self.z+k-1);
                }
            }
        }
        world_positions
    }

    pub fn generate_neighborhood_n_world_positions(&self, n: i32) -> Vec<WorldPosition> {
        let mut pos_vec = Vec::new();

        if n == 0 {
            return vec![*self];
        }

        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(WorldPosition::new(self.x-n, self.y+i, self.z+j))
            }
        }
        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(WorldPosition::new(self.x+n, self.y+i, self.z+j))
            }
        }
        for i in -n+1..n {
            for j in -n..n+1 {
                pos_vec.push(WorldPosition::new(self.x+i, self.y+n, self.z+j))
            }
        }
        for i in -n+1..n {
            for j in -n..n+1 {
                pos_vec.push(WorldPosition::new(self.x+i, self.y-n, self.z+j))
            }
        }
        for i in -n+1..n {
            for j in -n+1..n {
                pos_vec.push(WorldPosition::new(self.x+i, self.y+j, self.z+n))
            }
        }
        for i in -n+1..n {
            for j in -n+1..n {
                pos_vec.push(WorldPosition::new(self.x+i, self.y+j, self.z-n))
            }
        }

        pos_vec
    }

    pub fn to_perlin_pos(&self, scale_factor: f64) -> [f64;3] {
        [(self.x as f64 * scale_factor), (self.y as f64 * scale_factor), (self.z as f64 * scale_factor)]
    }

    pub fn get_position_incremented_by(&self, x: i32, y: i32, z: i32) -> Self {
        WorldPosition::new(self.x+x, self.y+y, self.z+z)
    }
}

impl Display for WorldPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}