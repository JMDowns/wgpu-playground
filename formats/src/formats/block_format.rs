use serde::{Serialize, Deserialize};
use super::color_format::ColorFormat;

#[derive(Serialize, Deserialize)]
pub struct BlockFormat {
    pub block_type: String,
    pub color: ColorFormat
}