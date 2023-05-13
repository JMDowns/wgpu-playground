use num_derive::FromPrimitive;
use phf_shared;
use std::fmt;
use core::hash::{Hash, Hasher};
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive)]
pub enum BlockType {
	AIR = 0,
	TRANSPARENT,
	WOOD,
	DIRT,
	GRASS,
	WHITE,
}
pub type BlockTypeSize = u8;
impl BlockType {
    pub fn get_block_type_from_int(btype: BlockTypeSize) -> Self {
        let btype_option = num::FromPrimitive::from_u8(btype as BlockTypeSize);
        btype_option.unwrap()
    }
   pub fn get_random_type() -> Self {
       num::FromPrimitive::from_u8(fastrand::u8(1..6)).unwrap()
   }
}
impl phf_shared::FmtConst for BlockType {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockType::{:?}", self)
    }
}
impl phf_shared::PhfHash for BlockType {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as BlockTypeSize).hash(state);
    }
}
impl phf_shared::PhfBorrow<BlockType> for BlockType {
    fn borrow(&self) -> &BlockType {
        self
    }
}
