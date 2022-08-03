use fundamentals::enums::block_type::BlockType;
pub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = 
::phf::Map {
    key: 2980949210194914378,
    disps: &[
        (1, 0),
    ],
    entries: &[
        ("GRASS", BlockType::GRASS,),
        ("DIRT", BlockType::DIRT,),
        ("WOOD", BlockType::WOOD,),
        ("WHITE", BlockType::WHITE,),
    ],
};

