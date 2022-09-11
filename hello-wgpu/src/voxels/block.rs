use fundamentals::{enums::block_type::BlockType, texture_coords::TextureCoordinates};
use derivables::dictionaries::block_type_to_texture_coordinates::BLOCK_TYPE_TO_TEXTURE_COORDINATES;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Block {
    pub block_type: u16
}

impl Block {
    pub fn new(bt: BlockType) -> Self {
        Block { block_type: bt as u16 }
    }

    pub fn is_air(&self) -> bool {
        self.block_type == 0
    }

    pub fn get_texture_coords(&self) -> &[TextureCoordinates; 6] {
        let btype_option = num::FromPrimitive::from_u16(self.block_type);
        let btype = match btype_option {
            Some(bt) => bt,
            None => BlockType::AIR
        };
        BLOCK_TYPE_TO_TEXTURE_COORDINATES.get(&btype).unwrap()
    }
}