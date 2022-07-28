use fundamentals::enums::block_type::BlockType;
pub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = 
::phf::Map {
    key: 15467950696543387533,
    disps: &[
        (1, 0),
    ],
    entries: &[
        ("DIRT", BlockType::DIRT,),
        ("WOOD", BlockType::WOOD,),
        ("GRASS", BlockType::GRASS,),
    ],
};

