pub struct SurfaceState<'a> {
    pub surface: wgpu::Surface<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub screen_color: wgpu::Color,
    pub config: wgpu::SurfaceConfiguration,
}