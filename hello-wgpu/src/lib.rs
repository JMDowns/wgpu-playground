mod texture;
mod camera;
mod voxels;
mod state;
mod tasks;
cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        mod thread_task_manager;
    }
}    

mod gpu_manager;

use fundamentals::loge;
use winit::{
    event::*, event_loop::{self, ControlFlow, EventLoop, EventLoopBuilder}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window
};
use state::{AppState, GraphicsBuilder, GraphicsResources, MaybeGraphicsResources, State};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;



#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            let window = web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();

            let canvas = document.create_element("canvas").unwrap_throw();
            canvas.set_id("wasm-example");
            canvas.set_attribute("width", "500").unwrap_throw();
            canvas.set_attribute("height", "500").unwrap_throw();

            let body = document
                .get_elements_by_tag_name("body")
                .item(0)
                .unwrap_throw();
            body.append_with_node_1(canvas.unchecked_ref())
                .unwrap_throw();
        } else {
            fundamentals::logger::CustomLogger::init(fundamentals::logger::LoggerInitArgs {
                debug_path_string: String::from("debug.log"),
                info_path_string: String::from("info.log"),
                warn_path_string: String::from("warn.log"),
                error_path_string: String::from("error.log"),
            });
        }
    }    
    

    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut state = AppState {
        state: None,
        window: None,
        graphics: MaybeGraphicsResources::Loading(GraphicsBuilder::new(event_loop.create_proxy()))
    };

    let _ = event_loop.run_app(&mut state);
}