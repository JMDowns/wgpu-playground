pub struct SurfaceState {
    pub surface: wgpu::Surface,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub screen_color: wgpu::Color,
    pub config: wgpu::SurfaceConfiguration,
}