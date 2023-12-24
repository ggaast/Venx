use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct PlatData {
    shared: Shared,
    region: Regi,
}

impl PlatData {
    pub fn to(self) -> (Shared, Regi) {
        (self.shared, self.region)
    }
}
#[derive(Debug, bitcode::Encode)]
pub struct Shared {
    pub array: Vec<Node>,
}

#[derive(Debug, bitcode::Encode)]
pub struct Regi {
    pub list: Vec<Wrapper>,
}

impl Regi {
    pub fn from_vec(vec: Vec<(u16, u16)>) -> Self {
        let mut list = vec![];
        for (block, amount) in vec.into_iter() {
            list.push(w(block.into()));
            list.push(w(amount.into()));
        }
        Regi { list }
    }
    pub fn merge(mut self, merge: usize) -> PlatData {
        let mut sh = Shared { array: vec![] };

        for i in 0..merge {
            log::info!("Merge is done by {i}/{merge}");
            let mut table: HashMap<(u32, u32), (u32, Option<u32>)> = HashMap::new();
            let mut to_remove = HashSet::new();
            // let chunks: Vec<&mut [u32]> = self.list.chunks_mut(2).collect();

            for i in (0..(self.list.len())).step_by(2) {
                if i + 1 >= self.list.len() {
                    break;
                }

                let (left, right) = (self.list[i], self.list[i + 1]);

                if let Some((first_idx, sh_idx_opt)) = table.get(&(left.val, right.val)) {
                    if let Some(sh_idx) = sh_idx_opt {
                        self.list[i].val = *sh_idx;
                        to_remove.insert(i + 1);
                        to_remove.insert((first_idx + 1) as usize);
                        self.list[(first_idx + 1) as usize].val = 115;
                    } else {
                        let in_shared_idx = sh.array.len();

                        sh.array.push(Node(left.val, right.val));
                        to_remove.insert(i + 1);
                        to_remove.insert((first_idx + 1) as usize);
                        self.list[i + 1].val = 115;

                        self.list[i].val = in_shared_idx as u32;
                        self.list[*first_idx as usize].val = in_shared_idx as u32;
                        let (.., sh_idx_opt) = table.get_mut(&(left.val, right.val)).unwrap();
                        *sh_idx_opt = Some(in_shared_idx as u32);
                    }
                } else if let Some((first_idx, sh_idx_opt)) = table.get(&(right.val, left.val)) {
                    if let Some(sh_idx) = sh_idx_opt {
                        self.list[i].val = *sh_idx;
                        to_remove.insert(i + 1);
                        to_remove.insert((first_idx + 1) as usize);
                        self.list[(first_idx + 1) as usize].val = 115;
                    } else {
                        let in_shared_idx = sh.array.len();

                        sh.array.push(Node(left.val, right.val));
                        to_remove.insert(i + 1);
                        to_remove.insert((first_idx + 1) as usize);
                        self.list[i + 1].val = 115;

                        self.list[i].val = in_shared_idx as u32;
                        self.list[*first_idx as usize].val = in_shared_idx as u32;
                        let (.., sh_idx_opt) = table.get_mut(&(right.val, left.val)).unwrap();
                        *sh_idx_opt = Some(in_shared_idx as u32);
                    }
                } else {
                    table.insert((left.val, right.val), (i as u32, None));
                }
            }
            self.list
                .retain(with_index(|index, _| !to_remove.contains(&index)));
        }

        PlatData {
            shared: sh,
            region: self,
        }
    }
}
fn with_index<T, F>(mut f: F) -> impl FnMut(&T) -> bool
where
    F: FnMut(usize, &T) -> bool,
{
    let mut i = 0;
    move |item| (f(i, item), i += 1).0
}
#[derive(Debug, bitcode::Encode)]
pub struct Node(pub u32, pub u32);

#[test]
fn new() {
    let line = vec![
        (88, 414),
        (414, 88),
        //
        // (44, 88),
        // (44, 88),
        // //
        // (88, 414),
        // (88, 414),
        // //
        // (44, 88),
        // (44, 818),
    ];
    let r = Regi::from_vec(line);

    let d = r.merge(5);

    dbg!(d);
}

#[test]
fn test_bitcode() {
    let a: Vec<u8> = bitcode::encode(&vec![89_u32; 200]).unwrap();
    let b: Vec<u8> = bitcode::encode(&vec![Wrapper { val: 89 }; 200]).unwrap();
    dbg!(a.len(), b.len());
}
#[derive(bitcode::Encode, Clone, Copy, Debug)]
pub struct Wrapper {
    #[bitcode_hint(expected_range = "0..67108864")]
    pub val: u32,
}
fn w(v: u32) -> Wrapper {
    Wrapper { val: v }
}
