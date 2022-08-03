use num_derive::FromPrimitive;
use phf_shared;
use std::fmt;
use core::hash::{Hash, Hasher};
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive)]
pub enum BlockType
{
	AIR = 0,
	WOOD,
	WHITE,
	DIRT,
	GRASS,
}

impl phf_shared::FmtConst for BlockType {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockType::{:?}", self)
    }
}
impl phf_shared::PhfHash for BlockType {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}
impl phf_shared::PhfBorrow<BlockType> for BlockType {
    fn borrow(&self) -> &BlockType {
        self
    }
}
