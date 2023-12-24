use anyhow::{bail, Result};
use bevy::{prelude, utils::HashMap};
use fastanvil::{complete, Chunk, Region};
use flate2::{write::ZlibEncoder, Compression};
use glam::{uvec3, Vec2, Vec3};
use rand::Rng;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    fs::{self, File},
    hash::{Hash, Hasher},
    io::Write,
    ops::Range,
    path::PathBuf,
};

use bmp::{Image, Pixel};

use crate::{
    plat::{
        draft3::{bl, Branch, Data},
        draft4::{PlatData, Regi},
        draft5::World,
        draft6::Wd,
    },
    voxel::{
        interfaces::{canvas::CanvasInterface, layer::LayerInterface},
        segment::Segment,
    },
};

use super::Plat;

pub type RegionX = i32;
pub type RegionZ = RegionX;

impl Plat {
    pub fn load_mca<'a>(
        dir_path: &'a str,
        region_range: (Range<RegionX>, Range<RegionZ>),
    ) -> Result<Self> {
        let rgs = from_dir(PathBuf::from(dir_path), region_range)?;

        let mut plat = Plat::new(10, 4, 9);
        type HOut = u8;
        fn hash(s: &str) -> HOut {
            let t = s;
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            let o = s.finish();
            o as HOut
        }
        //   let mut images: Vec<Image> = (0..380).map(|_| Image::new(512, 512)).collect();

        //   let mut test_segment: Vec<(u32, u32)> = vec![(0, 0)];

        //let mut map = HashMap::new();

        //    let mut id = 1;

        // let mut len_set = HashSet::new();
        let mut w = World::new(20);
        let mut canvas_handle = plat.new_canvas("Shit");
        for (rg_pos, mut region) in rgs {
            let mut segment = Segment::new(9);
            for ch_x in 0..32 {
                for ch_z in 0..32 {
                    if let Ok(Some(data)) = region.read_chunk(ch_x, ch_z) {
                        let complete_chunk = complete::Chunk::from_bytes(&data).unwrap();
                        // let mut archi_chunk = vec![];
                        for x in 0..16 {
                            for y in 0..380 {
                                for z in 0..16 {
                                    let mut was = false;
                                    if let Some(block) = complete_chunk.block(x, y - 60, z) {
                                        if block.name() != "minecraft:air" {
                                            let mut id = hash(block.name());

                                            // if block.name() == "minecraft:air" {
                                            //     id = 0;
                                            // } else if let Some(get_id) = map.get(&hash(block.name())) {
                                            //     id = *get_id;
                                            // } else {
                                            //     id = (map.len() + 1) as u8;
                                            //     map.insert(hash(block.name()), id);
                                            // }
                                            // archi_chunk.push(hash);
                                            // len_set.insert(hash);

                                            // if block.name() == "minecraft:grass_block" {
                                            //     images[y as usize].set_pixel(
                                            //         (ch_x * 16 + x) as u32,
                                            //         (ch_z * 16 + z) as u32,
                                            //         bmp::px!(20, 20, 90),
                                            //     );
                                            // }

                                            // if test_segment.last().unwrap().0 == hash {
                                            //     // if test_segment.last_mut().unwrap().1 == 32 {
                                            //     //     test_segment.push((hash, 1));
                                            //     // } else {
                                            //     test_segment.last_mut().unwrap().1 += 1;
                                            //     //  }
                                            // } else {
                                            //     test_segment.push((hash, 1));
                                            // }
                                            //}
                                            //     // dbg!(block.name());
                                            //     let block_id = match block.name() {
                                            //         "minecraft:dirt" => 1,
                                            //         "minecraft:grass_block" => 2,
                                            //         "minecraft:stone" => 3,
                                            //         "minecraft:water" => 8,
                                            //         _ => 404,
                                            //     };

                                            segment.set(
                                                uvec3(x as u32, y as u32, z as u32)
                                                    + uvec3(ch_x as u32 * 16, 0, ch_z as u32 * 16),
                                                id.into(),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        //w.add_chunk(archi_chunk);
                    }
                }
            }
            // dbg!(images.len());
            // canvas.insert_segment(segment, uvec3(rg_pos[0] as u32, 0, rg_pos[1] as u32));
            plat.get_canvas_mut(canvas_handle)
                .insert_segment(segment, uvec3(rg_pos[0] as u32, 0, rg_pos[1] as u32));
            // //plat.insert_segment(segment, uvec3(rg_pos[0] as u32, 0, rg_pos[1] as u32));
            //    plat.new_canvas("Imported minecraft map", segment);
            // for (i, img) in images.iter().enumerate() {
            //     img.save(format!("expr/grass_block/slice_{i}.bmp"))?;
            // }
            // // img.save("expr/slices_unoptimized/slice.bmp");
            // panic!();
        }
        // log::info!("Total Loses: {}, total merges: {}", w.loses, w.merges);

        // let mut under_32 = 0;
        // let mut over_32 = 0;
        // // for (ty, amount) in &test_segment {
        // //     output_sizes.push((*amount) as u16);
        // //     if amount > &32 {
        // //         over_32 += 1;
        // //     } else {
        // //         under_32 += 1;
        // //     }
        // // }
        // dbg!(under_32, over_32);

        // let mut map = HashSet::new();
        // let mut map_amount = HashSet::new();

        // let chunks: Vec<&[(u32, u32)]> = test_segment.chunks(7).collect();

        // for chunk in chunks {
        //     let mut raw = vec![];
        //     let mut raw_amount = vec![];
        //     for (ty, amount) in chunk {
        //         raw.push(ty);
        //         raw_amount.push((*amount).clamp(0, 16));
        //     }
        //     map.insert(raw);
        //     map_amount.insert(raw_amount);
        // }

        // dbg!(map.len());
        // dbg!(map_amount.len());

        // let mut rng = rand::thread_rng();

        // let mut output_sizes: Vec<(u32, u32)> = vec![];

        // for _ in 0..(test_segment.len() / 15) {
        //     output_sizes.push((rng.gen(), rng.gen()));
        // }

        // let (w, sh) = Wd::from_vec(test_segment);

        // save(img., "expr/d7/bmp_slice", "image.bmp").unwrap();
        // save(sh, "expr/d6/30x", "shared").unwrap();

        // let clone = test_segment.clone();
        // let chunk_size = 2;
        //
        //
        //
        //
        // let chunks: Vec<&[(u32, u32)]> = test_segment.chunks(2).collect();
        // let mut table: HashMap<((u32, u32), (u32, u32)), usize> = HashMap::new();
        // log::info!("Pairs in total: {}", chunks.len());
        // let mut amount = 0;
        // for chunk in chunks {
        //     if chunk.len() != 2 {
        //         break;
        //     }
        //     if let Some(how_many) = table.get_mut(&(chunk[0], chunk[1])) {
        //         amount += 1;
        //         *how_many += 1;
        //     } else {
        //         table.insert((chunk[0], chunk[1]), 1);
        //     }
        // }
        // let mut max = 0;
        // for (el, amount) in table.iter() {
        //     if amount > &max {
        //         max = *amount;
        //     }

        //     log::info!("PMerges: {amount}");
        // }
        // log::info!("Maximum merges: {max}; Table len: {}", table.len());
        // log::info!("Pairs merged: {amount}");

        // panic!();
        // log::info!("Looking for unique raw leafs");
        // let mut table: HashMap<(u32, u32), usize> = HashMap::new();
        // log::info!("Total branches {}", test_segment.len());
        // let mut amount = 0;
        // for branch in &test_segment {
        //     if let Some(how_many) = table.get_mut(branch) {
        //         amount += 1;
        //         *how_many += 1;
        //     } else {
        //         table.insert(*branch, 1);
        //     }
        // }
        // log::info!("Non unique branches: {amount}");
        // let mut max = 0;
        // for (el, amount) in table.iter() {
        //     if amount > &max {
        //         max = *amount;
        //     }

        //     log::info!("Merges: {amount}");
        // }
        // log::info!("Maximum merges: {max}; Table len: {}", table.len());
        // panic!();
        //
        //
        //
        // // dbg!(&chunks[0..100]);
        // // panic!();
        // let mut index = vec![];
        // let mut nodes = vec![];
        // // let mut index: Vec<u32> = vec![];
        // // let mut nodes = vec![];
        // let mut map: HashMap<&[(u32, u32)], usize> = HashMap::new();
        // let mut merge_map: HashMap<(usize, usize), usize> = HashMap::new();
        // let mut merged_amount = 0;
        // let mut merged_amount0 = 0;
        // let mut merged_amount1 = 0;
        // let mut merged_amount2 = 0;
        // let mut merged_amount3 = 0;
        // let mut total = 0;
        // for (i, chunk) in chunks.into_iter().enumerate() {
        //     total += 1;
        //     let mut found = false;
        //     if chunk.len() != chunk_size {
        //         break;
        //     }
        //     for i in 0..1 {
        //         let mut cloned = chunk.clone().to_vec();
        //         match i {
        //             0 => (),
        //             1 => cloned.swap(0, 1),
        //             // 2 => cloned.swap(1, 2),
        //             // 3 => cloned.swap(0, 2),
        //             _ => (),
        //         }
        //         if let Some(idx) = map.get(cloned.as_slice()) {
        //             index.push(*idx);
        //             merged_amount += 1;
        //             match i {
        //                 0 => merged_amount0 += 1,
        //                 1 => merged_amount1 += 1,
        //                 2 => merged_amount2 += 1,
        //                 3 => merged_amount3 += 1,
        //                 _ => panic!(),
        //             }

        //             found = true;
        //             break;
        //         }
        //     }
        //     if !found {
        //         map.insert(chunk, i);
        //         for (block, amount) in chunk {
        //             nodes.push(*block as usize);
        //             nodes.push(*amount as usize);
        //         }
        //     }
        // }

        // dbg!(
        //     (
        //         merged_amount,
        //         merged_amount0,
        //         merged_amount1,
        //         merged_amount2,
        //         merged_amount3
        //     ),
        //     total,
        //     // output.len(),
        //     map.len(),
        // );

        // dbg!(len_set.len());
        // for i in 0..1 {
        //     let temp = index.clone();
        //     let chunks: Vec<&[usize]> = temp.chunks(2).collect();
        //     index.clear();

        //     for chunk in chunks {
        //         let mut found = false;
        //         if chunk.len() != chunk_size {
        //             break;
        //         }
        //         for i in 0..2 {
        //             let mut cloned = chunk.clone().to_vec();
        //             match i {
        //                 0 => (),
        //                 1 => cloned.swap(0, 1),
        //                 // 2 => cloned.swap(1, 2),
        //                 // 3 => cloned.swap(0, 2),
        //                 _ => (),
        //             }
        //             if let Some(idx) = merge_map.get(&(cloned[0], cloned[1])) {
        //                 index.push(*idx);
        //                 merged_amount += 1;
        //                 match i {
        //                     0 => merged_amount0 += 1,
        //                     1 => merged_amount1 += 1,
        //                     2 => merged_amount2 += 1,
        //                     3 => merged_amount3 += 1,
        //                     _ => panic!(),
        //                 }
        //                 // dbg!("Yeah");
        //                 found = true;
        //                 break;
        //             }
        //         }
        //         if !found {
        //             merge_map.insert((chunk[0], chunk[1]), nodes.len());
        //             for el in chunk {
        //                 nodes.push(*el as usize);
        //                 nodes.push(*el as usize);
        //             }
        //         }
        //     }
        // }

        //
        // let mut rng = rand::thread_rng();
        // for el in &test_segment {

        // }
        //

        // let mut finalv = vec![];
        // for (left, right) in test_segment.into_iter() {
        //     finalv.push(left);
        //     finalv.push(right);
        // }
        // let r = Regi::from_vec(test_segment);
        // let d = r.merge(10);
        // let (sh, r) = d.to();
        // log::info!("Region lenght: {}", r.list.len());

        // let encoded: Vec<u8> = bitcode::encode(&r).unwrap();
        // let mut file = File::create(format!("expr/8reg/{name}/region"))?;
        // file.write_all(&encoded)?;
        // let encoded: Vec<u8> = bitcode::encode(&nodes).unwrap();
        // let mut file = File::create(format!("expr/all/new/{name}/nodes"))?;
        // file.write_all(&encoded)?;
        // panic!();

        Ok(plat)
    }
}

pub fn save<T: bitcode::Encode>(t: T, path: &str, name: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(path)?;

    // let mut data = Data::new();
    // data.from_vec(test_segment);

    // data.merge();

    let encoded: Vec<u8> = bitcode::encode(&t).unwrap();

    let mut file = File::create(format!("{path}/{name}"))?;
    file.write_all(&encoded)?;
    Ok(())
}

fn save_zlib<T: bitcode::Encode>(t: T, path: &str, name: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(path)?;

    // let mut data = Data::new();
    // data.from_vec(test_segment);

    // data.merge();
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    let encoded: Vec<u8> = bitcode::encode(&t).unwrap();
    e.write_all(&encoded)?;
    let compressed_bytes = e.finish()?;
    let mut file = File::create(format!("{path}/{name}"))?;
    file.write_all(&compressed_bytes)?;
    Ok(())
}

fn pos_from_name(name: &str) -> Option<[f32; 2]> {
    let parts: Vec<_> = name.split(".").collect();

    if parts.len() >= 3
        && parts[0] == "r"
        && parts[1].parse::<i32>().is_ok() // confirm that the second and third parts are nums
        && parts[2].parse::<i32>().is_ok()
    {
        Some([
            parts[1].parse().expect("Checked in the conditional"),
            parts[2].parse().expect("Checked in the conditional"),
        ])
    } else {
        None
    }
}
fn from_dir(
    dir: PathBuf,
    region_range: (Range<RegionX>, Range<RegionZ>),
) -> anyhow::Result<Vec<([i32; 2], Region<std::fs::File>)>> {
    let start = (region_range.0.start, region_range.1.start);
    let end = (region_range.0.end, region_range.1.end);

    let dir = fs::read_dir(dir)?;
    let mut out = Vec::new();
    for path in dir {
        let path = path?.path();
        let name = path.file_name();
        if let Some(name) = name {
            let coords = pos_from_name(name.to_str().unwrap());
            if let Some(coords) = coords {
                let x = coords[0] as i32;
                let z = coords[1] as i32;
                // if (x, z) >= start && (x, z) < end {
                let file = std::fs::File::open(path).unwrap();

                let region = Region::from_stream(file).unwrap();
                out.push(([x - start.0, z - start.1], region));
                // }

                continue;
            }
        }
        bail!("File path did not contain coords: {:?}", path);
    }
    Ok(out)
}
