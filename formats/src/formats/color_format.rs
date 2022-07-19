use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ColorFormat {
    pub r: f32,
    pub g: f32,
    pub b: f32
}