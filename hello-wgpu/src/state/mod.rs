pub mod flag_state;
pub mod input_manager;
mod task_manager;

use fundamentals::consts::MOUSE_SENSITIVITY;
use flag_state::FlagState;
use fundamentals::loge;
use log::info;
use pollster::FutureExt;
use wasm_bindgen::UnwrapThrowExt;
use wgpu::Device;
use wgpu::Instance;
use wgpu::Queue;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::WindowId;
use crate::gpu_manager::GPUManager;

use crate::camera;
use crate::tasks::Task;
use crate::voxels::world::World;
use std::future::Future;
use std::sync::Arc;
use std::sync::RwLock;

use fundamentals::consts::MOVEMENT_SPEED;

use winit::{
    event::*,
    window::Window,
};

use self::input_manager::InputManager;
use self::task_manager::TaskManager;

pub struct AppState<'a> {
    pub state: Option<State<'a>>,
    pub window: Option<Rc<Window>>,
    pub graphics: MaybeGraphicsResources
}

pub struct State<'a> {
    pub gpu_manager: GPUManager<'a>,
    flag_state: FlagState,
    input_manager: InputManager,
    task_manager: TaskManager,
    camera_controller: camera::CameraController,
    pub world: Arc<RwLock<World>>,
    pub last_render_time: web_time::Instant
}

#[cfg(target_arch = "wasm32")]
type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
type Rc<T> = std::sync::Arc<T>;

fn create_graphics(event_loop: &ActiveEventLoop) -> impl Future<Output = GraphicsResources> + 'static {
    #[allow(unused_mut)]
    let mut window_attrs = Window::default_attributes();

    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let canvas = document.get_element_by_id("wasm-example").unwrap_throw();
        let html_canvas_element = canvas.unchecked_into();
        window_attrs = window_attrs.with_canvas(Some(html_canvas_element));
    }

    let window = Rc::new(event_loop.create_window(window_attrs).unwrap());

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        #[cfg(not(target_arch="wasm32"))]
        backends: wgpu::Backends::PRIMARY,
        #[cfg(target_arch="wasm32")]
        backends: wgpu::Backends::BROWSER_WEBGPU,
        ..Default::default()
    });
    //let instance = wgpu::Instance::default();

    let surface = instance.create_surface(window.clone()).unwrap();

    async move {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: if cfg!(target_arch = "wasm32") {
                        wgpu::Features::empty()
                    } else {
                        wgpu::Features::POLYGON_MODE_LINE 
                        | wgpu::Features::MULTI_DRAW_INDIRECT 
                        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING 
                        | wgpu::Features::TEXTURE_BINDING_ARRAY 
                        | wgpu::Features::VERTEX_WRITABLE_STORAGE
                    },
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);

        GraphicsResources {
            window,
            surface,
            device,
            queue,
            instance,
            config
        }
    }
}

pub enum MaybeGraphicsResources {
    Loading(GraphicsBuilder),
    Loaded(GraphicsResources),
}

pub struct GraphicsResources {
    window: Rc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    instance: wgpu::Instance,
    config: wgpu::SurfaceConfiguration,
}

pub struct GraphicsBuilder {
    event_loop_proxy: Option<EventLoopProxy<GraphicsResources>>,
}

impl GraphicsBuilder {
    pub fn new(event_loop_proxy: EventLoopProxy<GraphicsResources>) -> Self {
        Self {
            event_loop_proxy: Some(event_loop_proxy),
        }
    }

    fn build_and_send(&mut self, event_loop: &ActiveEventLoop) {
        let Some(event_loop_proxy) = self.event_loop_proxy.take() else {
            // event_loop_proxy is already spent - we already constructed Graphics
            return;
        };

        #[cfg(target_arch = "wasm32")]
        {
            let gfx_fut = create_graphics(event_loop);
            wasm_bindgen_futures::spawn_local(async move {
                let gfx = gfx_fut.await;
                assert!(event_loop_proxy.send_event(gfx).is_ok());
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let gfx = pollster::block_on(create_graphics(event_loop));
            assert!(event_loop_proxy.send_event(gfx).is_ok());
        }
    }
}

impl<'a> State<'a> {
    pub fn new(surface: Surface<'a>, size: PhysicalSize<u32>, device: Device, queue: Queue, config: SurfaceConfiguration) -> Self {
        let gpu_manager = GPUManager::new(surface, size, device, queue, config);

        let camera_controller = camera::CameraController::new(MOVEMENT_SPEED, MOUSE_SENSITIVITY);
        let world = Arc::new(RwLock::new(World::new()));

        let mut task_manager = TaskManager::new();
        for pos in gpu_manager.chunk_index_state.read().unwrap().chunk_index_array.iter().rev() {
            task_manager.push_task(Task::GenerateChunk { chunk_position: *pos, world: world.clone() });
        }

        Self {
            gpu_manager,
            flag_state: FlagState {
                should_render_wireframe: false,
                has_moved: false,
            },
            input_manager: InputManager::new(),
            task_manager,
            world, 
            camera_controller,
            last_render_time: web_time::Instant::now()
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu_manager.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let (has_taken_input, should_switch_wireframe, should_rotate_subvoxel) = self.input_manager.input(event);
        if should_switch_wireframe {
            self.flag_state.should_render_wireframe = !self.flag_state.should_render_wireframe;
        }
        if (should_rotate_subvoxel) {
            self.gpu_manager.rotate_subvoxel_object(0);
        }
        has_taken_input
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.input_manager.handle_mouse_motion(delta);
    }

    pub fn process_input(&mut self) {
        self.flag_state.has_moved = self.camera_controller.process_mouse(&mut self.input_manager.input_state);
        self.flag_state.has_moved = self.camera_controller.process_keyboard(&self.input_manager.input_state) || self.flag_state.has_moved;
        self.gpu_manager.process_input(&self.flag_state);
    }

    pub fn update(&mut self, dt: web_time::Duration) {
        if (self.camera_controller.has_updates()) {
            self.gpu_manager.update_camera_and_reset_conroller(&mut self.camera_controller, dt);
        }
    }

    pub fn process_tasks(&mut self) {
        self.task_manager.process_tasks(self.world.clone(), &mut self.gpu_manager);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.gpu_manager.render()
    }
}

impl<'a> ApplicationHandler<GraphicsResources> for AppState<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let MaybeGraphicsResources::Loading(builder) = &mut self.graphics {
            builder.build_and_send(event_loop);
        }
    }
    /*
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

    

        if let None = self.state {
            


            #[cfg(target_arch = "wasm32")]
            {
                // Winit prevents sizing with CSS, so we have to set
                // the size manually when on web.
                use winit::dpi::PhysicalSize;
                let _ = self.window.as_ref().unwrap().request_inner_size(PhysicalSize::new(450, 400));
                
                use winit::platform::web::WindowExtWebSys;
                web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| {
                        let dst = doc.get_element_by_id("wasm-example")?;
                        let canvas = web_sys::Element::from(self.window.as_ref().unwrap().canvas()?);
                        dst.append_child(&canvas).ok()?;
                        Some(())
                    })
                    .expect("Couldn't append canvas to document body.");
            }

            let size = self.window.as_ref().unwrap().inner_size();

            unsafe {
                let ptr = self as *mut Self;
                let appstate = &mut *ptr;
                let surface = instance.create_surface(appstate.window.as_ref().unwrap()).unwrap();
                appstate.state = Some(State::new(instance, surface, size));
            };
        }
    }
    */

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if let None  = self.window.as_ref() {
            return;
        }
        let window = self.window.as_ref().unwrap();
        let state = self.state.as_mut().unwrap();
        if window_id == window.id() && !state.input(&event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    let now = web_time::Instant::now();
                    let dt = now - state.last_render_time;
                    if dt.as_millis() > 0 {
                        state.last_render_time = now;
                        state.process_input();
                        state.update(dt);
                        match state.render() {
                            Ok(_) => {}
                            //Reconfigure if surface is lost
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.gpu_manager.surface_state.size),
                            //System is out of memory, so we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(_e) => {
                                loge!("{:?}", _e)
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        if let Some(ref mut state) = self.state {
            match event {
                DeviceEvent::MouseMotion{ delta, } => state.handle_mouse_motion(delta),
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            let state = self.state.as_mut().unwrap();
            window.request_redraw();
            state.process_tasks();
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: GraphicsResources) {
        self.state = Some(State::new(graphics.surface, graphics.window.inner_size(), graphics.device, graphics.queue, graphics.config));
        self.window = Some(graphics.window);
    }
}

