use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ControlsFormat {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub forward: String,
    pub backward: String,

    pub mouse_sensitivity: f64,
}