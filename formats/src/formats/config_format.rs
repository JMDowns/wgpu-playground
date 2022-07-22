use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigFormat {
    pub num_threads_specified: usize,
    pub use_all_system_threads: bool,
    pub render_radius: usize
}