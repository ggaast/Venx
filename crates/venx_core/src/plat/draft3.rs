use std::{collections::HashMap, mem::ManuallyDrop};

#[derive(bitcode::Encode, Debug, Clone, PartialEq)]
pub struct Data {
    pub list: Vec<Branch>,
    pub heads: Vec<Branch>,
}

#[derive(bitcode::Encode)]
pub enum Node {
    Leaf { block: u32, amount: u32 },
    Branch { left: u32, right: u32 },
}

pub struct Leaf {
    pub block: u32,
    pub amount: u32,
}
#[derive(bitcode::Encode, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Branch {
    pub left: usize,
    pub right: usize,
}

impl Branch {
    pub fn swap(self) -> Self {
        Branch {
            left: self.right,
            right: self.left,
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Data {
            list: vec![bl(0, 0); 2],
            heads: vec![],
        }
    }
    pub fn insert(&mut self, idx: usize, block: u32, amount: u32, level: u8) -> bool {
        let node = &self.list[idx];
        let mut level = DEPTH;

        if node.left == 0 {}

        if level == 0 {}

        if self.insert(self.list[idx].left, block, amount, level - 1) {
            return true;
        } else if self.insert(self.list[idx].right, block, amount, level - 1) {
            return true;
        }

        false
    }
    pub fn merge(&mut self) {
        let mut split_amount = 0;
        for i in 1..2 {
            let mut table: HashMap<Branch, usize> = HashMap::new();
            for branch in self.heads.clone() {
                visit_node(
                    self,
                    branch.left,
                    DEPTH - 1,
                    i,
                    &mut table,
                    &mut split_amount,
                );
                visit_node(
                    self,
                    branch.right,
                    DEPTH - 1,
                    i,
                    &mut table,
                    &mut split_amount,
                );
            }
        }

        log::info!("{split_amount}");
        log::info!("Len before: {}", self.list.len());
        self.list.drain(..(split_amount));
        log::info!("Len after: {}", self.list.len());

        fn visit_node(
            data: &mut Data,
            idx: usize,
            level: u8,
            to_level: u8,
            table: &mut HashMap<Branch, usize>,
            split_amount: &mut usize,
        ) {
            let node = &mut data.list[idx];
            if level == to_level {
                if node.left != 0 {
                    let branch = &data.list[data.list[idx].left];
                    if let Some(new_idx) = table.get(&branch) {
                        let old_idx = data.list[idx].left;
                        let old = &mut data.list[old_idx];
                        old.left = 0;
                        old.right = 0;

                        data.list[idx].left = *new_idx;
                        *split_amount += 1;
                    } else if let Some(new_idx) = table.get(&branch.clone().swap()) {
                        let old_idx = data.list[idx].left;
                        let old = &mut data.list[old_idx];
                        old.left = 0;
                        old.right = 0;

                        data.list[idx].left = *new_idx;

                        *split_amount += 1;
                    } else {
                        table.insert(branch.clone(), data.list[idx].left);
                    }
                }
                if data.list[idx].right != 0 {
                    let branch = &data.list[data.list[idx].right];
                    if let Some(new_idx) = table.get(&branch) {
                        let old_idx = data.list[idx].right;
                        let old = &mut data.list[old_idx];
                        old.left = 0;
                        old.right = 0;
                        data.list[idx].right = *new_idx;
                        *split_amount += 1;
                    } else if let Some(new_idx) = table.get(&branch.clone().swap()) {
                        let old_idx = data.list[idx].right;
                        let old = &mut data.list[old_idx];
                        old.left = 0;
                        old.right = 0;
                        data.list[idx].right = *new_idx;
                        *split_amount += 1;
                    } else {
                        table.insert(branch.clone(), data.list[idx].right);
                    }
                }

                return;
            } else {
                let left = node.left;
                let right = node.right;

                if left != 0 {
                    visit_node(data, left, level - 1, to_level, table, split_amount);
                }
                if right != 0 {
                    visit_node(data, right, level - 1, to_level, table, split_amount);
                }
            }
        }
    }
    pub fn from_vec(&mut self, vec: Vec<Branch>) {
        let node = self.list.last_mut().unwrap();

        let chunks: Vec<&[Branch]> = vec.chunks(1 << DEPTH).collect();

        for (i, chunk) in chunks.into_iter().enumerate() {
            if chunk.len() != 1 << DEPTH {
                return;
            }

            let new_left = bl(0, 0);
            let new_right = bl(0, 0);
            let left_idx = self.list.len();
            let right_idx = left_idx + 1;
            let new_node = bl(left_idx, right_idx);
            self.heads.push(new_node);
            self.list.push(new_left);
            self.list.push(new_right);
            let mut chunk_ptr = 0;
            visit_node(self, &mut chunk_ptr, left_idx, DEPTH - 1, chunk);
            visit_node(self, &mut chunk_ptr, right_idx, DEPTH - 1, chunk);
        }
        fn visit_node(
            data: &mut Data,
            ch_ptr: &mut usize,
            idx: usize,
            level: u8,
            chunk: &[Branch],
        ) {
            let node = &mut data.list[idx];
            if level == 0 {
                *node = chunk[*ch_ptr].clone();
                *ch_ptr += 1;
                return;
            } else {
                let new_left = Branch { left: 0, right: 0 };
                let new_idx = data.list.len();
                data.list.push(new_left);
                data.list[idx].left = new_idx;
                visit_node(data, ch_ptr, new_idx, level - 1, chunk);

                let new_right = Branch { left: 0, right: 0 };
                let new_idx = data.list.len();
                data.list.push(new_right);
                data.list[idx].right = new_idx;
                visit_node(data, ch_ptr, new_idx, level - 1, chunk);

                // visit_node(data, ch_ptr, , level, chunk)
            }
        }
    }
}
const DEPTH: u8 = 3;
#[test]
fn insert_vec() {
    let vec = vec![
        bl(44, 11),
        bl(1, 1300),
        bl(4, 10660),
        bl(5, 1010),
        bl(7, 400),
        bl(23, 6600),
        bl(511, 14300),
        bl(444, 4),
    ];

    let mut data = Data::new();
    data.from_vec(vec);
    let cloned = data.clone();
    data.merge();
    //dbg!(data);
    assert_eq!(cloned, data);

    // dbg!(data);
}
#[test]
fn merge() {
    let vec = vec![
        bl(44, 11),
        bl(11, 44),
        bl(44, 11),
        bl(11, 44),
        bl(44, 11),
        bl(11, 44),
        bl(44, 11),
        bl(11, 44),
        //
        // bl(44, 11),
        // bl(44, 11),
        // bl(40, 10),
        // bl(44, 11),
        // //
        // bl(11, 44),
        // bl(11, 44),
        // bl(11, 44),
        // bl(11, 44),
    ];

    let mut data = Data::new();
    data.from_vec(vec);
    data.merge();
    dbg!(data);
}

pub fn bl(ty: usize, amount: usize) -> Branch {
    Branch {
        left: ty,
        right: amount,
    }
}
