#[derive(Clone, Copy)]
pub struct TextureCoordinates {
    pub tx: u32,
    pub ty: u32,
}

impl TextureCoordinates {
    pub fn new(tx: u32, ty: u32) -> Self {
        TextureCoordinates { tx, ty}
    }

    pub fn offset(&self, w: u32, h: u32) -> Self {
        TextureCoordinates { tx: self.tx+w, ty: self.ty+h }
    }
}

impl std::fmt::Display for TextureCoordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(tx={}, ty={})", self.tx, self.ty)
    }
}