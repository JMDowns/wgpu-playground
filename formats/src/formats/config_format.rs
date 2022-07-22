use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigFormat {
    pub num_threads: u8
}