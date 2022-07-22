use std::fmt::Display;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl Position {
    pub fn origin() -> Self {
        Position { x: 0, y: 0, z: 0}
    }

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Position { x, y, z}
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}