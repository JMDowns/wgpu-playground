use derivables::{vertex::Vertex, block::Block};
use fundamentals::{world_position::WorldPosition, enums::{block_side::BlockSide, block_type::BlockType}, consts::CHUNK_DIMENSION};
use super::chunk::{Chunk, ChunkBlockIterator};


#[derive(Debug, Clone, Copy, PartialEq)]
struct Face {
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

    fn merge_up(&self, other: &Face) -> Option<Face> {
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

    fn merge_right(&self, other: &Face) -> Option<Face> {
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

    fn merge_left(&self, other: &Face) -> Option<Face> {
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

#[derive(Debug)]
pub struct Mesh {
    pub front: (Vec<Vertex>, Vec<u32>, u32),
    pub back: (Vec<Vertex>, Vec<u32>, u32),
    pub left: (Vec<Vertex>, Vec<u32>, u32),
    pub right: (Vec<Vertex>, Vec<u32>, u32),
    pub top: (Vec<Vertex>, Vec<u32>, u32),
    pub bottom: (Vec<Vertex>, Vec<u32>, u32),
}

impl Mesh {
    pub fn new() -> Self {
        Mesh { 
            front: (Vec::new(), Vec::new(), 0), 
            back: (Vec::new(), Vec::new(), 0), 
            left: (Vec::new(), Vec::new(), 0), 
            right: (Vec::new(), Vec::new(), 0),
            top: (Vec::new(), Vec::new(), 0),
            bottom: (Vec::new(), Vec::new(), 0) 
        }
    }

    pub fn cull(chunk: &Chunk, index: u32) -> Self {
        let mut mesh = Mesh::new();

        let mut cbi = ChunkBlockIterator::new(chunk);

        while let Some(((i,j,k), block)) = cbi.get_next_block() {
            let adjacent_blocks_data = Self::generate_adjacent_blocks(&chunk, i, j, k);
            mesh.add_vertices(
                Self::generate_cube(WorldPosition::new(i as i32-1,j as i32-1,k as i32-1), block.get_texture_indices(), &adjacent_blocks_data, index), 
                Self::generate_cube_indices(&adjacent_blocks_data),
            );
        }

        mesh
    }

    pub fn cull_side(chunk: &Chunk, index: u32, side: BlockSide) -> (Vec<Vertex>, Vec<u32>, u32) {
        let mut mesh_side_vertices = Vec::new();
        let mut mesh_side_indices = Vec::new();
        let mut mesh_side_indices_count = 0;

        let mut cbi = ChunkBlockIterator::new(chunk);

        while let Some(((i,j,k), block)) = cbi.get_next_block() {
            if !Mesh::is_adjacent_blocks_solid_side(chunk, i, j, k, side) {
                mesh_side_vertices.append(&mut Self::generate_cube_side(WorldPosition::new(i as i32-1,j as i32-1,k as i32-1), block.get_texture_indices(), index, side));
                let mut index_vec = Self::generate_cube_indices_side(side);
                mesh_side_indices_count += index_vec.len() as u32;
                mesh_side_indices.append(&mut index_vec);
            }
            
        }

        (mesh_side_vertices, mesh_side_indices, mesh_side_indices_count)
    }

    fn greedy_merge_front_and_modify_vecs(front_faces_yz_slices_index_x: &mut Vec<Vec<Face>>, front_faces_yz_slices_index_xbefore: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>) {
        for x in 0..CHUNK_DIMENSION as usize {
            match (front_faces_yz_slices_index_xbefore[x].len(), front_faces_yz_slices_index_x[x].len()) {
                (0, _) => {
                    front_faces_yz_slices_index_xbefore[x].extend(front_faces_yz_slices_index_x[x].drain(..));
                }
                (_, 0) => {
                    for face in front_faces_yz_slices_index_xbefore[x].drain(..) {
                        faces_to_make.push(face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = front_faces_yz_slices_index_xbefore[x][before_index];
                        let current_face = front_faces_yz_slices_index_x[x][current_index];
                        if (before_face.ul.2 == 4 && before_face.ul.1 == 16 && before_face.block_side == BlockSide::FRONT) {
                            println!("Before Face: {:?}", before_face);
                            println!("Other Face: {:?}", current_face);
                        } 
                        if let Some(merged_face) = before_face.merge_right(&current_face) {
                            front_faces_yz_slices_index_x[x][current_index] = merged_face;
                            before_index = before_index + 1;
                            current_index = current_index + 1;
                        } else {
                            if before_face.ul.1 == current_face.ul.1 {
                                faces_to_make.push(front_faces_yz_slices_index_xbefore[x][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_face.ul.1 < current_face.ul.1 {
                                while (before_index < before_len && front_faces_yz_slices_index_xbefore[x][before_index].ul.1 < current_face.ul.1) {
                                    faces_to_make.push(front_faces_yz_slices_index_xbefore[x][before_index]);
                                    before_index += 1;
                                }
                            } else if before_face.ul.1 > current_face.ul.1 {
                                while (current_index < current_len && front_faces_yz_slices_index_x[x][current_index].ul.1 < before_face.ul.1) {
                                    current_index += 1;
                                }
                                if current_index == current_len {
                                    faces_to_make.push(front_faces_yz_slices_index_xbefore[x][before_index]);
                                    before_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(front_faces_yz_slices_index_xbefore[x][i]);
                    }

                    front_faces_yz_slices_index_xbefore[x].clear();
                    front_faces_yz_slices_index_xbefore[x].extend(front_faces_yz_slices_index_x[x].drain(..));
                }
            }
        }
    }

    fn greedy_merge_back_and_modify_vecs(back_faces_yz_slices_index_x: &mut Vec<Vec<Face>>, back_faces_yz_slices_index_xbefore: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>) {
        for x in 0..CHUNK_DIMENSION as usize {
            match (back_faces_yz_slices_index_xbefore[x].len(), back_faces_yz_slices_index_x[x].len()) {
                (0, _) => {
                    back_faces_yz_slices_index_xbefore[x].extend(back_faces_yz_slices_index_x[x].drain(..));
                }
                (_, 0) => {
                    for face in back_faces_yz_slices_index_xbefore[x].drain(..) {
                        faces_to_make.push(face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = back_faces_yz_slices_index_xbefore[x][before_index];
                        let current_face = back_faces_yz_slices_index_x[x][current_index];
                        if let Some(merged_face) = before_face.merge_left(&current_face) {
                            back_faces_yz_slices_index_x[x][current_index] = merged_face;
                            before_index = before_index + 1;
                            current_index = current_index + 1;
                        } else {
                            if before_face.ul.1 == current_face.ul.1 {
                                faces_to_make.push(back_faces_yz_slices_index_xbefore[x][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_face.ul.1 < current_face.ul.1 {
                                while (before_index < before_len && back_faces_yz_slices_index_xbefore[x][before_index].ul.1 < current_face.ul.1) {
                                    faces_to_make.push(back_faces_yz_slices_index_xbefore[x][before_index]);
                                    before_index += 1;
                                }
                            } else if before_face.ul.1 > current_face.ul.1 {
                                while (current_index < current_len && back_faces_yz_slices_index_x[x][current_index].ul.1 < before_face.ul.1) {
                                    current_index += 1;
                                }
                                if current_index == current_len {
                                    faces_to_make.push(back_faces_yz_slices_index_xbefore[x][before_index]);
                                    before_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(back_faces_yz_slices_index_xbefore[x][i]);
                    }

                    back_faces_yz_slices_index_xbefore[x].clear();
                    back_faces_yz_slices_index_xbefore[x].extend(back_faces_yz_slices_index_x[x].drain(..));
                }
            }
        }
    }

    fn greedy_merge_left_and_modify_vecs(left_faces_xy_slices_index_z: &mut Vec<Vec<Face>>, left_faces_xy_slices_index_zbefore: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>) {
        for z in 0..CHUNK_DIMENSION as usize {
            match (left_faces_xy_slices_index_zbefore[z].len(), left_faces_xy_slices_index_z[z].len()) {
                (0, _) => {
                    left_faces_xy_slices_index_zbefore[z].extend(left_faces_xy_slices_index_z[z].drain(..));
                }
                (_, 0) => {
                    for face in left_faces_xy_slices_index_zbefore[z].drain(..) {
                        faces_to_make.push(face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = left_faces_xy_slices_index_zbefore[z][before_index];
                        let current_face = left_faces_xy_slices_index_z[z][current_index];
                        if let Some(merged_face) = before_face.merge_up(&current_face) {
                            left_faces_xy_slices_index_z[z][current_index] = merged_face;
                            before_index = before_index + 1;
                            current_index = current_index + 1;
                        } else {
                            if before_face.ul.0 == current_face.ul.0 {
                                faces_to_make.push(left_faces_xy_slices_index_zbefore[z][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_face.ul.0 < current_face.ul.0 {
                                while (before_index < before_len && left_faces_xy_slices_index_zbefore[z][before_index].ul.0 < current_face.ul.0) {
                                    faces_to_make.push(left_faces_xy_slices_index_zbefore[z][before_index]);
                                    before_index += 1;
                                }
                            } else if before_face.ul.0 > current_face.ul.0 {
                                while (current_index < current_len && left_faces_xy_slices_index_z[z][current_index].ul.0 < before_face.ul.0) {
                                    current_index += 1;
                                }
                                if current_index == current_len {
                                    faces_to_make.push(left_faces_xy_slices_index_zbefore[z][before_index]);
                                    before_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(left_faces_xy_slices_index_zbefore[z][i]);
                    }

                    left_faces_xy_slices_index_zbefore[z].clear();
                    left_faces_xy_slices_index_zbefore[z].extend(left_faces_xy_slices_index_z[z].drain(..));
                }
            }
        }
    }
    fn greedy_merge_right_and_modify_vecs(right_faces_xy_slices_index_z: &mut Vec<Vec<Face>>, right_faces_xy_slices_index_zbefore: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>) {
        for z in 0..CHUNK_DIMENSION as usize {
            match (right_faces_xy_slices_index_zbefore[z].len(), right_faces_xy_slices_index_z[z].len()) {
                (0, _) => {
                    right_faces_xy_slices_index_zbefore[z].extend(right_faces_xy_slices_index_z[z].drain(..));
                }
                (_, 0) => {
                    for face in right_faces_xy_slices_index_zbefore[z].drain(..) {
                        faces_to_make.push(face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = right_faces_xy_slices_index_zbefore[z][before_index];
                        let current_face = right_faces_xy_slices_index_z[z][current_index];
                        if let Some(merged_face) = before_face.merge_up(&current_face) {
                            right_faces_xy_slices_index_z[z][current_index] = merged_face;
                            before_index = before_index + 1;
                            current_index = current_index + 1;
                        } else {
                            if before_face.ur.0 == current_face.ur.0 {
                                faces_to_make.push(right_faces_xy_slices_index_zbefore[z][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_face.ur.0 < current_face.ur.0 {
                                while (before_index < before_len && right_faces_xy_slices_index_zbefore[z][before_index].ur.0 < current_face.ur.0) {
                                    faces_to_make.push(right_faces_xy_slices_index_zbefore[z][before_index]);
                                    before_index += 1;
                                }
                            } else if before_face.ur.0 > current_face.ur.0 {
                                while (current_index < current_len && right_faces_xy_slices_index_z[z][current_index].ur.0 < before_face.ur.0) {
                                    current_index += 1;
                                }
                                if current_index == current_len {
                                    faces_to_make.push(right_faces_xy_slices_index_zbefore[z][before_index]);
                                    before_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(right_faces_xy_slices_index_zbefore[z][i]);
                    }

                    right_faces_xy_slices_index_zbefore[z].clear();
                    right_faces_xy_slices_index_zbefore[z].extend(right_faces_xy_slices_index_z[z].drain(..));
                }
            }
        }
    }
    fn greedy_merge_top_and_modify_vecs(top_faces_xz_slices_index_y: &mut Vec<Vec<Face>>, top_faces_xz_slices_index_ybefore: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>) {
        for y in 0..CHUNK_DIMENSION as usize {
            match (top_faces_xz_slices_index_ybefore[y].len(), top_faces_xz_slices_index_y[y].len()) {
                (0, _) => {
                    top_faces_xz_slices_index_ybefore[y].extend(top_faces_xz_slices_index_y[y].drain(..));
                }
                (_, 0) => {
                    for face in top_faces_xz_slices_index_ybefore[y].drain(..) {
                        faces_to_make.push(face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = top_faces_xz_slices_index_ybefore[y][before_index];
                        let current_face = top_faces_xz_slices_index_y[y][current_index];
                        if let Some(merged_face) = before_face.merge_right(&current_face) {
                            top_faces_xz_slices_index_y[y][current_index] = merged_face;
                            before_index = before_index + 1;
                            current_index = current_index + 1;
                        } else {
                            if before_face.ur.0 == current_face.ur.0 {
                                faces_to_make.push(top_faces_xz_slices_index_ybefore[y][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_face.ur.0 < current_face.ur.0 {
                                while (before_index < before_len && top_faces_xz_slices_index_ybefore[y][before_index].ur.0 < current_face.ur.0) {
                                    faces_to_make.push(top_faces_xz_slices_index_ybefore[y][before_index]);
                                    before_index += 1;
                                }
                            } else if before_face.ur.0 > current_face.ur.0 {
                                while (current_index < current_len && top_faces_xz_slices_index_y[y][current_index].ur.0 < before_face.ur.0) {
                                    current_index += 1;
                                }
                                if current_index == current_len {
                                    faces_to_make.push(top_faces_xz_slices_index_ybefore[y][before_index]);
                                    before_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(top_faces_xz_slices_index_ybefore[y][i]);
                    }

                    top_faces_xz_slices_index_ybefore[y].clear();
                    top_faces_xz_slices_index_ybefore[y].extend(top_faces_xz_slices_index_y[y].drain(..));
                    
                }
            }
        }
    }
    fn greedy_merge_bottom_and_modify_vecs(bottom_faces_xz_slices_index_y: &mut Vec<Vec<Face>>, bottom_faces_xz_slices_index_ybefore: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>) {
        for y in 0..CHUNK_DIMENSION as usize {
            match (bottom_faces_xz_slices_index_ybefore[y].len(), bottom_faces_xz_slices_index_y[y].len()) {
                (0, _) => {
                    bottom_faces_xz_slices_index_ybefore[y].extend(bottom_faces_xz_slices_index_y[y].drain(..));
                }
                (_, 0) => {
                    for face in bottom_faces_xz_slices_index_ybefore[y].drain(..) {
                        faces_to_make.push(face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = bottom_faces_xz_slices_index_ybefore[y][before_index];
                        let current_face = bottom_faces_xz_slices_index_y[y][current_index];
                        if let Some(merged_face) = before_face.merge_left(&current_face) {
                            bottom_faces_xz_slices_index_y[y][current_index] = merged_face;
                            before_index = before_index + 1;
                            current_index = current_index + 1;
                        } else {
                            if before_face.ur.0 == current_face.ur.0 {
                                faces_to_make.push(bottom_faces_xz_slices_index_ybefore[y][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_face.ur.0 < current_face.ur.0 {
                                while (before_index < before_len && bottom_faces_xz_slices_index_ybefore[y][before_index].ur.0 < current_face.ur.0) {
                                    faces_to_make.push(bottom_faces_xz_slices_index_ybefore[y][before_index]);
                                    before_index += 1;
                                }
                            } else if before_face.ur.0 > current_face.ur.0 {
                                while (current_index < current_len && bottom_faces_xz_slices_index_y[y][current_index].ur.0 < before_face.ur.0) {
                                    current_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(bottom_faces_xz_slices_index_ybefore[y][i]);
                    }

                    bottom_faces_xz_slices_index_ybefore[y].clear();
                    bottom_faces_xz_slices_index_ybefore[y].extend(bottom_faces_xz_slices_index_y[y].drain(..));
                    
                }
            }
        }
    }

    fn greedy_merge_and_modify_vecs(current_layer: &mut Vec<Vec<Face>>, before_layer: &mut Vec<Vec<Face>>, faces_to_make: &mut Vec<Face>, side: BlockSide) {
        for layer_index in 0..CHUNK_DIMENSION as usize {
            if layer_index == 0 && side == BlockSide::FRONT && before_layer[layer_index].contains(&Face { lr: (0, 12, 8), ll: (0, 12, 7), ur: (0, 16, 8), ul: (0, 16, 7), block_type_int: 1, block_side: BlockSide::FRONT }) {
                println!("Current layer: {:?}", current_layer);
                println!("Before layer: {:?}", before_layer);
            }
            match (before_layer[layer_index].len(), current_layer[layer_index].len()) {
                (0, _) => {
                    before_layer[layer_index].extend(current_layer[layer_index].drain(..));
                }
                (_, 0) => {
                    for before_face in before_layer[layer_index].drain(..) {
                        if (before_face.ul.2 == 4 && before_face.ul.1 == 16 && before_face.block_side == BlockSide::FRONT) {
                            println!("Before Face: {:?}", before_face);
                           // println!("Other Face: {:?}", current_face);
                        } 
                        faces_to_make.push(before_face);
                    }
                }
                (before_len, current_len) => {
                    let mut before_index = 0;
                    let mut current_index = 0;

                    while (before_index < before_len && current_index < current_len) {
                        let before_face = before_layer[layer_index][before_index];
                        let current_face = current_layer[layer_index][current_index];
                        if (before_face.ul.2 == 4 && before_face.ul.1 == 16 && before_face.block_side == BlockSide::FRONT) {
                            println!("Before Face: {:?}", before_face);
                            println!("Other Face: {:?}", current_face);
                        } 
                        let merged_face_option = match side {
                            BlockSide::FRONT => {
                                before_face.merge_right(&current_face)
                            }
                            BlockSide::BACK => {
                                before_face.merge_left(&current_face)
                            }
                            BlockSide::LEFT => {
                                before_face.merge_up(&current_face)
                            }
                            BlockSide::RIGHT => {
                                before_face.merge_up(&current_face)
                            }
                            BlockSide::TOP => {
                                before_face.merge_right(&current_face)
                            }
                            BlockSide::BOTTOM => {
                                before_face.merge_left(&current_face)
                            }
                        };
                        if let Some(merged_face) = merged_face_option {
                            current_layer[layer_index][current_index] = merged_face;
                            before_index += 1;
                            current_index += 1;
                        } else {
                            let (before_boundary, current_boundary) = match side {
                                BlockSide::FRONT => {
                                    (before_face.ul.1, current_face.ul.1)
                                }
                                BlockSide::BACK => {
                                    (before_face.ul.1, current_face.ul.1)
                                }
                                BlockSide::LEFT => {
                                    (before_face.ul.0, current_face.ul.0)
                                }
                                BlockSide::RIGHT => {
                                    (before_face.ur.0, current_face.ur.0)
                                }
                                BlockSide::TOP => {
                                    (before_face.ur.0, current_face.ur.0)
                                }
                                BlockSide::BOTTOM => {
                                    (before_face.ur.0, current_face.ur.0)
                                }
                            };
                            if before_boundary == current_boundary {
                                faces_to_make.push(before_layer[layer_index][before_index]);
                                before_index += 1;
                                current_index += 1;
                            } else if before_boundary < current_boundary {
                                let before_boundary_2 = match side {
                                    BlockSide::FRONT => {
                                        before_layer[layer_index][before_index].ul.1
                                    }
                                    BlockSide::BACK => {
                                        before_layer[layer_index][before_index].ul.1
                                    }
                                    BlockSide::LEFT => {
                                        before_layer[layer_index][before_index].ul.0
                                    }
                                    BlockSide::RIGHT => {
                                        before_layer[layer_index][before_index].ur.0
                                    }
                                    BlockSide::TOP => {
                                        before_layer[layer_index][before_index].ur.0
                                    }
                                    BlockSide::BOTTOM => {
                                        before_layer[layer_index][before_index].ur.0
                                    }
                                };
                                while (before_index < before_len && before_boundary_2 < current_boundary) {
                                    faces_to_make.push(before_layer[layer_index][before_index]);
                                    before_index += 1;
                                }
                            } else if before_boundary > current_boundary {
                                let current_boundary_2 = match side {
                                    BlockSide::FRONT => {
                                        current_layer[layer_index][current_index].ul.1
                                    }
                                    BlockSide::BACK => {
                                        current_layer[layer_index][current_index].ul.1
                                    }
                                    BlockSide::LEFT => {
                                        current_layer[layer_index][current_index].ul.0
                                    }
                                    BlockSide::RIGHT => {
                                        current_layer[layer_index][current_index].ur.0
                                    }
                                    BlockSide::TOP => {
                                        current_layer[layer_index][current_index].ur.0
                                    }
                                    BlockSide::BOTTOM => {
                                        current_layer[layer_index][current_index].ur.0
                                    }
                                };
                                while (current_index < current_len && current_boundary_2 < before_boundary) {
                                    current_index += 1;
                                }
                                if current_index == current_len {
                                    faces_to_make.push(before_layer[layer_index][before_index]);
                                    before_index += 1;
                                }
                            }
                        }
                    }

                    for i in before_index..before_len {
                        faces_to_make.push(before_layer[layer_index][i]);
                    }

                    before_layer[layer_index].clear();
                    before_layer[layer_index].extend(current_layer[layer_index].drain(..));
                    
                }
            }
        }
    }

    pub fn greedy(chunk: &Chunk, index: u32) -> Self {
        let mut mesh = Mesh::new();
        let mut cbi = ChunkBlockIterator::new(chunk);

        let mut front_faces_yz_slices_index_x: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut front_faces_yz_slices_index_xbefore: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut back_faces_yz_slices_index_x: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut back_faces_yz_slices_index_xbefore: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut left_faces_xy_slices_index_z: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut left_faces_xy_slices_index_zbefore: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut right_faces_xy_slices_index_z: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut right_faces_xy_slices_index_zbefore: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut top_faces_xz_slices_index_y: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut top_faces_xz_slices_index_ybefore: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut bottom_faces_xz_slices_index_y: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];
        let mut bottom_faces_xz_slices_index_ybefore: Vec<Vec<Face>> = vec![Vec::new(); CHUNK_DIMENSION as usize];

        let mut faces_to_make = Vec::new();

        let mut current_x = 0;
        let mut current_y = 0;
        let mut current_z = 0;

        let mut merged_front = 0;

        while let Some(((i,j,k), block)) = cbi.get_next_block() {
            if current_y < j-1 {
                Self::greedy_merge_and_modify_vecs(&mut left_faces_xy_slices_index_z, &mut left_faces_xy_slices_index_zbefore, &mut faces_to_make, BlockSide::LEFT);
                Self::greedy_merge_and_modify_vecs(&mut right_faces_xy_slices_index_z, &mut right_faces_xy_slices_index_zbefore, &mut faces_to_make, BlockSide::RIGHT);
            }
            if current_z < k-1 {
                if (merged_front == 6 || merged_front == 7 || merged_front == 8) {
                    println!("Current vec for {}th time: {:?}", merged_front,front_faces_yz_slices_index_x );
                    println!("Before vec for {}th time: {:?}", merged_front,front_faces_yz_slices_index_xbefore );
                }
                Self::greedy_merge_and_modify_vecs(&mut front_faces_yz_slices_index_x, &mut front_faces_yz_slices_index_xbefore, &mut faces_to_make, BlockSide::FRONT);
                println!("Merged front for the {}th time!", merged_front);
                merged_front += 1;
                Self::greedy_merge_and_modify_vecs(&mut back_faces_yz_slices_index_x, &mut back_faces_yz_slices_index_xbefore, &mut faces_to_make, BlockSide::BACK);
                Self::greedy_merge_and_modify_vecs(&mut top_faces_xz_slices_index_y, &mut top_faces_xz_slices_index_ybefore, &mut faces_to_make, BlockSide::TOP);
                Self::greedy_merge_and_modify_vecs(&mut bottom_faces_xz_slices_index_y, &mut bottom_faces_xz_slices_index_ybefore, &mut faces_to_make, BlockSide::BOTTOM);
            }
            current_x = i-1;
            current_y = j-1;
            current_z = k-1;
            let adjacent_blocks_data = Self::generate_adjacent_blocks(&chunk, i, j, k);

            if !adjacent_blocks_data[0] {
                let front_face = Face::new(current_x,current_y,current_z, block.block_type as usize, BlockSide::FRONT);
                match front_faces_yz_slices_index_x[current_x].last() {
                    Some(face) => {
                        if let Some(merged_face) = face.merge_up(&front_face) {
                            front_faces_yz_slices_index_x[current_x].pop();
                            front_faces_yz_slices_index_x[current_x].push(merged_face);
                        } else {
                            front_faces_yz_slices_index_x[current_x].push(front_face);
                        }
                    },
                    None => {
                        front_faces_yz_slices_index_x[current_x].push(front_face);
                    }
                }
            }
            if !adjacent_blocks_data[1] {
                let back_face = Face::new(current_x,current_y,current_z, block.block_type as usize, BlockSide::BACK);
                match back_faces_yz_slices_index_x[current_x].last() {
                    Some(face) => {
                        if let Some(merged_face) = face.merge_up(&back_face) {
                            back_faces_yz_slices_index_x[current_x].pop();
                            back_faces_yz_slices_index_x[current_x].push(merged_face);
                        } else {
                            back_faces_yz_slices_index_x[current_x].push(back_face);
                        }
                    },
                    None => {
                        back_faces_yz_slices_index_x[current_x].push(back_face);
                    }
                }
            }
            if !adjacent_blocks_data[2] {
                let left_face = Face::new(current_x,current_y,current_z, block.block_type as usize, BlockSide::LEFT);
                match left_faces_xy_slices_index_z[current_z].last() {
                    Some(face) => {
                        if let Some(merged_face) = face.merge_left(&left_face) {
                            left_faces_xy_slices_index_z[current_z].pop();
                            left_faces_xy_slices_index_z[current_z].push(merged_face);
                        } else {
                            left_faces_xy_slices_index_z[current_z].push(left_face);
                        }
                    },
                    None => {
                        left_faces_xy_slices_index_z[current_z].push(left_face);
                    }
                }
            }
            if !adjacent_blocks_data[3] {
                let right_face = Face::new(current_x,current_y,current_z, block.block_type as usize, BlockSide::RIGHT);
                match right_faces_xy_slices_index_z[current_z].last() {
                    Some(face) => {
                        if let Some(merged_face) = face.merge_right(&right_face) {
                            right_faces_xy_slices_index_z[current_z].pop();
                            right_faces_xy_slices_index_z[current_z].push(merged_face);
                        } else {
                            right_faces_xy_slices_index_z[current_z].push(right_face);
                        }
                    },
                    None => {
                        right_faces_xy_slices_index_z[current_z].push(right_face);
                    }
                }
            }
            if !adjacent_blocks_data[4] {
                let top_face = Face::new(current_x,current_y,current_z, block.block_type as usize, BlockSide::TOP);
                match top_faces_xz_slices_index_y[current_y].last() {
                    Some(face) => {
                        if let Some(merged_face) = face.merge_up(&top_face) {
                            top_faces_xz_slices_index_y[current_y].pop();
                            top_faces_xz_slices_index_y[current_y].push(merged_face);
                        } else {
                            top_faces_xz_slices_index_y[current_y].push(top_face);
                        }
                    },
                    None => {
                        top_faces_xz_slices_index_y[current_y].push(top_face);
                    }
                }
            }
            if !adjacent_blocks_data[5] {
                let bottom_face = Face::new(current_x,current_y,current_z, block.block_type as usize, BlockSide::BOTTOM);
                match bottom_faces_xz_slices_index_y[current_y].last() {
                    Some(face) => {
                        if let Some(merged_face) = face.merge_up(&bottom_face) {
                            bottom_faces_xz_slices_index_y[current_y].pop();
                            bottom_faces_xz_slices_index_y[current_y].push(merged_face);
                        } else {
                            bottom_faces_xz_slices_index_y[current_y].push(bottom_face);
                        }
                    },
                    None => {
                        bottom_faces_xz_slices_index_y[current_y].push(bottom_face);
                    }
                }
            }
        }

        Self::greedy_merge_front_and_modify_vecs(&mut front_faces_yz_slices_index_x, &mut front_faces_yz_slices_index_xbefore, &mut faces_to_make);
        Self::greedy_merge_back_and_modify_vecs(&mut back_faces_yz_slices_index_x, &mut back_faces_yz_slices_index_xbefore, &mut faces_to_make);
        Self::greedy_merge_left_and_modify_vecs(&mut left_faces_xy_slices_index_z, &mut left_faces_xy_slices_index_zbefore, &mut faces_to_make);
        Self::greedy_merge_right_and_modify_vecs(&mut right_faces_xy_slices_index_z, &mut right_faces_xy_slices_index_zbefore, &mut faces_to_make);
        Self::greedy_merge_top_and_modify_vecs(&mut top_faces_xz_slices_index_y, &mut top_faces_xz_slices_index_ybefore, &mut faces_to_make);
        Self::greedy_merge_bottom_and_modify_vecs(&mut bottom_faces_xz_slices_index_y, &mut bottom_faces_xz_slices_index_ybefore, &mut faces_to_make);

        for face_vec in front_faces_yz_slices_index_xbefore {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in front_faces_yz_slices_index_x {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in back_faces_yz_slices_index_xbefore {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in back_faces_yz_slices_index_x {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in left_faces_xy_slices_index_zbefore {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in left_faces_xy_slices_index_z {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in right_faces_xy_slices_index_zbefore {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in right_faces_xy_slices_index_z {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in top_faces_xz_slices_index_ybefore {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in top_faces_xz_slices_index_y {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in bottom_faces_xz_slices_index_ybefore {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        for face_vec in bottom_faces_xz_slices_index_y {
            for face in face_vec {
                faces_to_make.push(face);
            }
        }

        let mut vertex_vec = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut index_vec = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut num_faces_generated = vec![0;6];

        for face in faces_to_make {
            let face_index = face.block_side as usize;
            vertex_vec[face_index].extend(Self::generate_face_vertices(&face, index));
            index_vec[face_index].extend(Self::generate_face_indices(&face, index, num_faces_generated[face_index]));
            num_faces_generated[face_index] += 1;
        }

        mesh.add_vertices(vertex_vec, index_vec);

        mesh
    }

    pub fn add_vertices(&mut self, mut block_vertices: [Vec<Vertex>; 6], block_indices: [Vec<u32>; 6]) {
        self.front.1.append(&mut block_indices[0].iter().map(|e| (e+self.front.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.back.1.append(&mut block_indices[1].iter().map(|e| (e+self.back.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.left.1.append(&mut block_indices[2].iter().map(|e| (e+self.left.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.right.1.append(&mut block_indices[3].iter().map(|e| (e+self.right.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.top.1.append(&mut block_indices[4].iter().map(|e| (e+self.top.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());
        self.bottom.1.append(&mut block_indices[5].iter().map(|e| (e+self.bottom.0.len() as u32) % fundamentals::consts::NUM_VERTICES_IN_BUCKET).collect());

        self.front.0.append(&mut block_vertices[0]);
        self.back.0.append(&mut block_vertices[1]);
        self.left.0.append(&mut block_vertices[2]);
        self.right.0.append(&mut block_vertices[3]);
        self.top.0.append(&mut block_vertices[4]);
        self.bottom.0.append(&mut block_vertices[5]);

        self.front.2 = self.front.1.len() as u32;
        self.back.2 = self.back.1.len() as u32;
        self.left.2 = self.left.1.len() as u32;
        self.right.2 = self.right.1.len() as u32;
        self.top.2 = self.top.1.len() as u32;
        self.bottom.2 = self.bottom.1.len() as u32;
    }

    fn generate_adjacent_blocks(chunk: &Chunk, i: usize, j: usize, k: usize) -> [bool; 6] {
        let mut adjacency_data = [false;6];
        adjacency_data[0] = chunk.is_block_solid(i-1, j, k);
        adjacency_data[1] = chunk.is_block_solid(i+1, j, k);
        adjacency_data[2] = chunk.is_block_solid(i, j, k-1);
        adjacency_data[3] = chunk.is_block_solid(i, j, k+1);
        adjacency_data[4] = chunk.is_block_solid(i, j+1, k);
        adjacency_data[5] = chunk.is_block_solid(i, j-1, k);
        adjacency_data
    }

    fn is_adjacent_blocks_solid_side(chunk: &Chunk, i: usize, j: usize, k: usize, side: BlockSide) -> bool {
        match side {
            BlockSide::FRONT => chunk.is_block_solid(i-1, j, k),
            BlockSide::BACK => chunk.is_block_solid(i+1, j, k),
            BlockSide::LEFT => chunk.is_block_solid(i, j, k-1),
            BlockSide::RIGHT => chunk.is_block_solid(i, j, k+1),
            BlockSide::TOP => chunk.is_block_solid(i, j+1, k),
            BlockSide::BOTTOM => chunk.is_block_solid(i, j-1, k)
        }
    }

    fn generate_face_vertices(face: &Face, index: u32) -> Vec<Vertex> {
        let texture_indices = &Block::get_texture_indices_from_int(face.block_type_int);
        let (texture_index, u_offset, v_offset) = match face.block_side {
            BlockSide::FRONT => {
                (0, (face.lr.2-face.ll.2) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::BACK => {
                (1, (face.ll.2-face.lr.2) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::LEFT => {
                (2, (face.ll.0-face.lr.0) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::RIGHT => {
                (3, (face.lr.0-face.ll.0) as u8, (face.ul.1-face.ll.1) as u8)
            }
            BlockSide::TOP => {
                (4, (face.lr.2-face.ll.2) as u8, (face.ul.0-face.ll.0) as u8)
            }
            BlockSide::BOTTOM => {
                (5, (face.ll.2-face.lr.2) as u8, (face.ul.0-face.ll.0) as u8)
            }
        };

        [
            Vertex::new(WorldPosition::new(face.ll.0 as i32, face.ll.1 as i32, face.ll.2 as i32), texture_indices[texture_index], 0, v_offset, index),
            Vertex::new(WorldPosition::new(face.lr.0 as i32, face.lr.1 as i32, face.lr.2 as i32), texture_indices[texture_index], u_offset, v_offset, index),
            Vertex::new(WorldPosition::new(face.ul.0 as i32, face.ul.1 as i32, face.ul.2 as i32), texture_indices[texture_index], 0, 0, index),
            Vertex::new(WorldPosition::new(face.ur.0 as i32, face.ur.1 as i32, face.ur.2 as i32), texture_indices[texture_index], u_offset, 0, index)
        ].to_vec()
        
    }

    fn generate_face_indices(face: &Face, index: u32, num_faces_generated: u32) -> Vec<u32> {
        [
            0+num_faces_generated*4,1+num_faces_generated*4,3+num_faces_generated*4,
            0+num_faces_generated*4,3+num_faces_generated*4,2+num_faces_generated*4,
        ].to_vec()
    }
    
    fn generate_cube(pos: WorldPosition, tex_index_arr: &[usize; 6], adjacent_blocks_data: &[bool;6], index: u32) -> [Vec<Vertex>; 6] {
        let positions = pos.generate_vertex_world_positions();
        let mut vertices_arr = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),];
        let all_vertices_arr = 
        [
            //Front
            [
            Vertex::new(positions[0], tex_index_arr[0], 0, 1, index),
            Vertex::new(positions[1], tex_index_arr[0], 1, 1, index),
            Vertex::new(positions[2], tex_index_arr[0], 0, 0, index),
            Vertex::new(positions[3], tex_index_arr[0], 1, 0, index),
            ].to_vec(),
            //Back
            [
            Vertex::new(positions[4], tex_index_arr[1], 1, 1, index),
            Vertex::new(positions[5], tex_index_arr[1], 0, 1, index),
            Vertex::new(positions[6], tex_index_arr[1], 1, 0, index),
            Vertex::new(positions[7], tex_index_arr[1], 0, 0, index),
            ].to_vec(),
            //Left
            [
            Vertex::new(positions[0], tex_index_arr[2], 1, 1, index),
            Vertex::new(positions[2], tex_index_arr[2], 1, 0, index),
            Vertex::new(positions[4], tex_index_arr[2], 0, 1, index),
            Vertex::new(positions[6], tex_index_arr[2], 0, 0, index),
            ].to_vec(),
            //Right
            [
            Vertex::new(positions[1], tex_index_arr[3], 0, 1, index),
            Vertex::new(positions[3], tex_index_arr[3], 0, 0, index),
            Vertex::new(positions[5], tex_index_arr[3], 1, 1, index),
            Vertex::new(positions[7], tex_index_arr[3], 1, 0, index),
            ].to_vec(),
            //Top
            [
            Vertex::new(positions[2], tex_index_arr[4], 0, 1, index),
            Vertex::new(positions[3], tex_index_arr[4], 1, 1, index),
            Vertex::new(positions[6], tex_index_arr[4], 0, 0, index),
            Vertex::new(positions[7], tex_index_arr[4], 1, 0, index),
            ].to_vec(),
            //Bottom
            [
            Vertex::new(positions[0], tex_index_arr[5], 0, 0, index),
            Vertex::new(positions[1], tex_index_arr[5], 1, 0, index),
            Vertex::new(positions[4], tex_index_arr[5], 0, 1, index),
            Vertex::new(positions[5], tex_index_arr[5], 1, 1, index),
            ].to_vec()
        ];
    
        for i in 0..6 {
            if !adjacent_blocks_data[i] {
                vertices_arr[i] = all_vertices_arr[i].clone();
            }
        }
    
        vertices_arr
    }
    fn generate_cube_indices(adjacent_blocks_data: &[bool;6]) -> [Vec<u32>;6] {
        let mut indices_arr = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),];
        let all_indices_arr = 
        [
            // Front Face
            [
            0,1,3,
            0,3,2,
            ].to_vec(),
                
            // Back Face
            [    
            1,0,2,
            1,2,3,
            ].to_vec(),
                
            // Left Face
            [
            2,0,1,
            2,1,3,
            ].to_vec(),
                
            // Right Face
            [
            0,2,3,
            0,3,1,
            ].to_vec(),
                
            // Top Face
            [
            0,1,3,
            0,3,2,
            ].to_vec(),
            
            // Bottom Face
            [
            1,0,2,
            1,2,3
            ].to_vec()
                
        ];
    
        for i in 0..6 {
            if !adjacent_blocks_data[i] {
                indices_arr[i] = all_indices_arr[i].clone();
            }
        }
    
        indices_arr
    }

    fn generate_cube_side(pos: WorldPosition, tex_index_arr: &[usize; 6], index: u32, side: BlockSide) -> Vec<Vertex> {
        let positions = pos.generate_vertex_world_positions();
        match side {
            BlockSide::FRONT => {
                [
                    Vertex::new(positions[0], tex_index_arr[0], 0, 1, index),
                    Vertex::new(positions[1], tex_index_arr[0], 1, 1, index),
                    Vertex::new(positions[2], tex_index_arr[0], 0, 0, index),
                    Vertex::new(positions[3], tex_index_arr[0], 1, 0, index),
                ].to_vec()
            },
            BlockSide::BACK => {
                [
                    Vertex::new(positions[4], tex_index_arr[1], 1, 1, index),
                    Vertex::new(positions[5], tex_index_arr[1], 0, 1, index),
                    Vertex::new(positions[6], tex_index_arr[1], 1, 0, index),
                    Vertex::new(positions[7], tex_index_arr[1], 0, 0, index),
                ].to_vec()
            },
            BlockSide::LEFT => {
                [
                    Vertex::new(positions[0], tex_index_arr[2], 1, 1, index),
                    Vertex::new(positions[2], tex_index_arr[2], 1, 0, index),
                    Vertex::new(positions[4], tex_index_arr[2], 0, 1, index),
                    Vertex::new(positions[6], tex_index_arr[2], 0, 0, index),
                ].to_vec()
            },
            BlockSide::RIGHT => {
                [
                    Vertex::new(positions[1], tex_index_arr[3], 0, 1, index),
                    Vertex::new(positions[3], tex_index_arr[3], 0, 0, index),
                    Vertex::new(positions[5], tex_index_arr[3], 1, 1, index),
                    Vertex::new(positions[7], tex_index_arr[3], 1, 0, index),
                ].to_vec()
            },
            BlockSide::TOP => {
                [
                    Vertex::new(positions[2], tex_index_arr[4], 0, 1, index),
                    Vertex::new(positions[3], tex_index_arr[4], 1, 1, index),
                    Vertex::new(positions[6], tex_index_arr[4], 0, 0, index),
                    Vertex::new(positions[7], tex_index_arr[4], 1, 0, index),
                ].to_vec()
            },
            BlockSide::BOTTOM => {
                [
                    Vertex::new(positions[0], tex_index_arr[5], 0, 0, index),
                    Vertex::new(positions[1], tex_index_arr[5], 1, 0, index),
                    Vertex::new(positions[4], tex_index_arr[5], 0, 1, index),
                    Vertex::new(positions[5], tex_index_arr[5], 1, 1, index),
                ].to_vec()
            }
        }
    }
    
    fn generate_cube_indices_side(side: BlockSide) -> Vec<u32> {
        match side {
            BlockSide::FRONT => {
                [
                    0,1,3,
                    0,3,2,
                ].to_vec()
            },
            BlockSide::BACK => {
                [    
                    1,0,2,
                    1,2,3,
                ].to_vec()
            },
            BlockSide::LEFT => {
                [
                    2,0,1,
                    2,1,3,
                ].to_vec()
            },
            BlockSide::RIGHT => {
                [
                    0,2,3,
                    0,3,1,
                ].to_vec()
            },
            BlockSide::TOP => {
                [
                    0,2,3,
                    0,3,1,
                ].to_vec()
            },
            BlockSide::BOTTOM => {
                [
                    1,0,2,
                    1,2,3
                ].to_vec()
            }
        }
    }
}
