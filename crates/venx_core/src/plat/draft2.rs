use std::collections::HashSet;

use crate::voxel::cpu::facade::Idx;

#[derive(bitcode::Encode)]
struct Nodes {
    list: Vec<Node>,
    set: HashSet<()>,
}

#[derive(bitcode::Encode, Eq, PartialEq, Hash)]
enum Node {
    Branch {
        mirrored: bool,
        children: [Idx; 2],
    },
    Leaf {
        // max
        block: u32,
        // max
        amount: u32,
    },
}
#[derive(bitcode::Encode)]
struct Index {
    pub indices: Vec<Idx>,
}

#[test]
fn test_data_structure() {
    //let set = HashSet::new();
}
