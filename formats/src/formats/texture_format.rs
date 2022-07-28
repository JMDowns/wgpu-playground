use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TextureFormat {
    front: String,
    back: String,
    left: String,
    right: String,
    top: String,
    bottom: String,
}

impl TextureFormat {
    pub fn to_vec(&self) -> Vec<String> {
        vec![self.front.clone(), self.back.clone(), self.left.clone(), self.right.clone(), self.top.clone(), self.bottom.clone()]
    }
}