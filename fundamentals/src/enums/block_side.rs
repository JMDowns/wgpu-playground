use strum_macros::EnumIter;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, EnumIter)]
pub enum BlockSide {
    FRONT = 0,
    BACK = 1,
    LEFT = 2,
    RIGHT = 3,
    TOP = 4,
    BOTTOM = 5,
}