use fundamentals::enums::block_type::BlockType;
pub static BLOCK_TYPE_TO_COLOR: phf::Map<BlockType, wgpu::Color> = 
::phf::Map {
    key: 7485420634051515786,
    disps: &[
        (3, 0),
    ],
    entries: &[
        (BlockType::GRASS, wgpu::Color { r: 0.0, g: 0.321, b: 0.0, a: 0.0}),
        (BlockType::STONE, wgpu::Color { r: 0.411, g: 0.411, b: 0.411, a: 0.0}),
        (BlockType::WATER, wgpu::Color { r: 0.0, g: 0.0, b: 0.5, a: 0.0}),
        (BlockType::DIRT, wgpu::Color { r: 0.627, g: 0.321, b: 0.176, a: 0.0}),
        (BlockType::DEAD_GRASS, wgpu::Color { r: 0.0, g: 0.1, b: 0.0, a: 0.0}),
    ],
};

