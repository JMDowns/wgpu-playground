pub struct TextureState {
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: texture::Texture,
    pub depth_texture: texture::Texture,
}