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

    pub fn generate_positions(&self) -> [Position; 8] {
        [
            Position::new(self.x, self.y, self.z),
            Position::new(self.x-1, self.y, self.z),
            Position::new(self.x-1, self.y-1, self.z),
            Position::new(self.x, self.y-1, self.z),
            Position::new(self.x, self.y, self.z-1),
            Position::new(self.x-1, self.y, self.z-1),
            Position::new(self.x-1, self.y-1, self.z-1),
            Position::new(self.x, self.y-1, self.z-1),
        ]
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