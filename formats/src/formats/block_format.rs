use serde::{Serialize, Deserialize};
use crate::formats::texture_format::TextureFormat;

#[derive(Serialize, Deserialize)]
pub struct BlockFormat {
    pub block_type: String,
    pub texture: TextureFormat,
}