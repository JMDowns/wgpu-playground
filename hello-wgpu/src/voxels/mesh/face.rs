use fundamentals::enums::block_side::BlockSide;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    pub lr: (usize, usize, usize),
    pub ll: (usize, usize, usize),
    pub ur: (usize, usize, usize),
    pub ul: (usize, usize, usize),
    pub block_type_int: usize,
    pub block_side: BlockSide
}

impl Face {
    pub fn new(i: usize, j: usize, k: usize, block_type_int: usize, block_side: BlockSide) -> Self {
        match block_side {
            BlockSide::FRONT => {
                Face {
                    ll: (i, j, k),
                    lr: (i, j, k+1),
                    ul: (i, j+1, k),
                    ur: (i, j+1, k+1),
                    block_type_int,
                    block_side
                }
            },

            BlockSide::BACK => {
                Face {
                    ll: (i+1, j, k+1),
                    lr: (i+1, j, k),
                    ul: (i+1, j+1, k+1),
                    ur: (i+1, j+1, k),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::LEFT => {
                Face {
                    ll: (i+1, j, k),
                    lr: (i, j, k),
                    ul: (i+1, j+1, k),
                    ur: (i, j+1, k),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::RIGHT => {
                Face {
                    ll: (i, j, k+1),
                    lr: (i+1, j, k+1),
                    ul: (i, j+1, k+1),
                    ur: (i+1, j+1, k+1),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::TOP => {
                Face {
                    ll: (i, j+1, k),
                    lr: (i, j+1, k+1),
                    ul: (i+1, j+1, k),
                    ur: (i+1, j+1, k+1),
                    block_type_int,
                    block_side
                }
            }

            BlockSide::BOTTOM => {
                Face {
                    ll: (i, j, k+1),
                    lr: (i, j, k),
                    ul: (i+1, j, k+1),
                    ur: (i+1, j, k),
                    block_type_int,
                    block_side
                }
            }
        }
        
    }

    pub fn merge_up(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.ul == other.ll && self.ur == other.lr {
            return Some(Face {
                ul: other.ul,
                ur: other.ur,
                ll: self.ll,
                lr: self.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int
            });
        }

        None
    }

    pub fn merge_right(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.lr == other.ll && self.ur == other.ul {
            return Some(Face {
                ul: self.ul,
                ur: other.ur,
                ll: self.ll,
                lr: other.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int
            });
        }

        None
    }

    pub fn merge_left(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.ll == other.lr && self.ul == other.ur {
            return Some(Face {
                ul: other.ul,
                ur: self.ur,
                ll: other.ll,
                lr: self.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int
            });
        }

        None
    }
}