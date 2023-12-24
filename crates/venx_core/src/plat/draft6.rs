use std::collections::HashMap;

use super::mca_converter::save;

#[derive(bitcode::Encode, Debug)]
pub struct SharedTypes {
    pub list: Vec<[u32; LEN]>,
}
#[derive(bitcode::Encode, Debug)]
pub struct Wd {
    pub list: Vec<u32>,
}

const LEN: usize = 30;

impl Wd {
    pub fn from_vec(vec: Vec<(u32, u32)>) -> (Self, SharedTypes) {
        let chunks: Vec<&[(u32, u32)]> = vec.chunks(LEN).collect();
        let mut output: Vec<u32> = vec![];
        let mut tys_links: HashMap<[u32; LEN], usize> = HashMap::new();
        let mut len = 0;

        let mut shared_types = SharedTypes { list: vec![] };

        let amount_links = SharedAmounts::new().links;

        for chunk in chunks {
            if chunk.len() != LEN {
                break;
            }
            let mut raw = [0; LEN];
            for (i, (ty, ..)) in chunk.iter().enumerate() {
                raw[i] = *ty;
            }

            if let Some(link) = tys_links.get(&raw) {
                output.push(*link as u32);
            } else {
                tys_links.insert(raw, len);
                shared_types.list.push(raw);
                output.push(len as u32);
                len += 1;
            }
            let sub_chunks: Vec<&[(u32, u32)]> = chunk.chunks(5).collect();

            for sub_chunk in sub_chunks {
                let mut raw = [a(32); 5];
                for (i, (.., amount)) in sub_chunk.iter().enumerate() {
                    if amount > &32 {
                        output.push(*amount);
                    } else {
                        raw[i] = a(*amount.clamp(&1, &32) as u8);
                    }
                }

                let link = amount_links.get(&raw).unwrap();
                output.push(*link as u32);
            }
        }
        (Self { list: output }, shared_types)
    }
}

#[derive(bitcode::Encode)]
pub struct SharedAmounts {
    //    pub list: Vec<[Amount; 5]>,
    pub links: HashMap<[Amount; 5], usize>,
}

impl SharedAmounts {
    pub fn new() -> Self {
        let mut list = 0;
        let mut links = HashMap::new();
        for a0 in 1..33 {
            println!("{a0}/32");

            for a1 in 1..33 {
                for a2 in 1..33 {
                    for a3 in 1..33 {
                        for a4 in 1..33 {
                            let amounts = [a(a0), a(a1), a(a2), a(a3), a(a4)];
                            links.insert(amounts.clone(), list);
                            list += 1;
                        }
                    }
                }
            }
        }

        Self { links }
    }
}

#[derive(bitcode::Encode, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Amount {
    #[bitcode_hint(expected_range = "0..33")]
    pub val: u8,
}

fn a(v: u8) -> Amount {
    Amount { val: v }
}

#[test]
fn gen() {
    let sh_a = SharedAmounts::new();
    // dbg!(sh_a.list.len());
    save(sh_a, "expr/d6", "shared_amounts_links").unwrap();
}

#[test]
fn merge() {
    let v = vec![
        (15, 21),
        (11, 1),
        (12, 2),
        (13, 11),
        (14, 1),
        (15, 21),
        (16, 1),
        (17, 21),
        (18, 1),
        (19, 21),
        (10, 1),
    ];

    let (a, b) = Wd::from_vec(v);

    dbg!(a, b);
}
