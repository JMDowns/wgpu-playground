use fundamentals::enums::block_type::BlockType;
use derivables::dictionaries::block_type_to_color::BLOCK_TYPE_TO_COLOR;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Block {
    pub block_type: u16
}

impl Block {
    pub fn new(bt: BlockType) -> Self {
        Block { block_type: bt as u16 }
    }

    pub fn get_block_type(&self) -> BlockType {
        let btype = num::FromPrimitive::from_u16(self.block_type);
        match btype {
            Some(bt) => bt,
            None => BlockType::AIR
        }
    }

    pub fn get_color(&self) -> wgpu::Color {
        let btype = num::FromPrimitive::from_u16(self.block_type);
        match btype {
            Some(bt) => {
                match BLOCK_TYPE_TO_COLOR.get(&(bt)) {
                    Some(color) => *color,
                    None => wgpu::Color::TRANSPARENT
                }
            },
            None => wgpu::Color::TRANSPARENT
        }
    }
}