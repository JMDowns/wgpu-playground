mod texture;
mod camera;
mod voxels;
mod state;
mod tasks;
mod thread_task_manager;
mod gpu_manager;

use fundamentals::loge;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use state::State;

pub async fn run() {
    fundamentals::logger::CustomLogger::init(fundamentals::logger::LoggerInitArgs {
        debug_path_string: String::from("debug.log"),
        info_path_string: String::from("info.log"),
        warn_path_string: String::from("warn.log"),
        error_path_string: String::from("error.log"),
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window));
    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion{ delta, },
            .. // We're not using device_id currently
        } => state.handle_mouse_motion(delta),

        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }

        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let now = instant::Instant::now();
            let dt = now - last_render_time;
            if dt.as_millis() > 0 {
                last_render_time = now;
                state.process_input();
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    //Reconfigure if surface is lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.gpu_manager.surface_state.size),
                    //System is out of memory, so we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(_e) => {
                        loge!("{:?}", _e)
                    }
                }
            }
        }

        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually request it.
            state.process_tasks();
            window.request_redraw();
        }
        _ => {}
    });
}