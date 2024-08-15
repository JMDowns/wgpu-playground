mod texture;
mod camera;
mod voxels;
mod state;
mod tasks;
mod thread_task_manager;
mod gpu_manager;

use fundamentals::loge;
use winit::{
    event::*, event_loop::{ControlFlow, EventLoop, EventLoopBuilder}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window
};
use state::State;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;



#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            fundamentals::logger::CustomLogger::init(fundamentals::logger::LoggerInitArgs {
                debug_path_string: String::from("debug.log"),
                info_path_string: String::from("info.log"),
                warn_path_string: String::from("warn.log"),
                error_path_string: String::from("error.log"),
            });
        }
    }    
    

    let event_loop = EventLoopBuilder::new().build().unwrap();
    let window = event_loop.create_window(Window::default_attributes()).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));
        
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }


    let mut state = State::new(&window).await;

    let _ = event_loop.run_app(&mut state);
}