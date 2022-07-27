use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BlockFormat {
    pub block_type: String,
    pub texture_filename: String,
}