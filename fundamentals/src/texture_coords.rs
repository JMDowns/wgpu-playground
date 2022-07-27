#[derive(Clone, Copy)]
pub struct TextureCoordinates {
    pub tx: f32,
    pub ty: f32,
}

impl TextureCoordinates {
    pub fn new(tx: f32, ty: f32) -> Self {
        TextureCoordinates { tx, ty}
    }

    pub fn offset(&self, w: f32, h: f32) -> Self {
        TextureCoordinates { tx: self.tx+w, ty: self.ty+h }
    }
}

impl std::fmt::Display for TextureCoordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(tx={}, ty={})", self.tx, self.ty)
    }
}