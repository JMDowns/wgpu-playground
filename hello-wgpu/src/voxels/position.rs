use std::fmt::Display;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Position { x, y, z}
    }

    pub fn generate_vertex_positions(&self) -> [Position; 8] {
        [
            Position::new(self.x, self.y, self.z),
            Position::new(self.x, self.y, self.z+1),
            Position::new(self.x, self.y+1, self.z),
            Position::new(self.x, self.y+1, self.z+1),
            Position::new(self.x+1, self.y, self.z),
            Position::new(self.x+1, self.y, self.z+1),
            Position::new(self.x+1, self.y+1, self.z),
            Position::new(self.x+1, self.y+1, self.z+1),
        ]
    }

    pub fn generate_neighborhood1_positions(&self) -> [[[Position; 3]; 3]; 3] {
        let mut positions = [[[Position::new(self.x,self.y,self.z); 3]; 3]; 3];
        for k in 0..3 as i32 {
            for j in 0..3 as i32 {
                for i in 0..3 as i32{
                    positions[i as usize][j as usize][k as usize] = Position::new(self.x-i+1, self.y-j+1, self.z-k+1);
                }
            }
        }
        positions
    }

    pub fn generate_neighborhood_n_positions(&self, n: i32) -> Vec<Position> {
        let mut pos_vec = Vec::new();

        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(Position::new(self.x-n, self.y+i, self.z+j))
            }
        }
        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(Position::new(self.x+n, self.y+i, self.z+j))
            }
        }
        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(Position::new(self.x+i, self.y+n, self.z+j))
            }
        }
        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(Position::new(self.x+i, self.y-n, self.z+j))
            }
        }
        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(Position::new(self.x+i, self.y+j, self.z-n))
            }
        }
        for i in -n..n+1 {
            for j in -n..n+1 {
                pos_vec.push(Position::new(self.x+i, self.y+j, self.z-n))
            }
        }

        pos_vec
    }

    pub fn to_perlin_pos(&self, scale_factor: f64) -> [f64;3] {
        [(self.x as f64 * scale_factor), (self.y as f64 * scale_factor), (self.z as f64 * scale_factor)]
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}