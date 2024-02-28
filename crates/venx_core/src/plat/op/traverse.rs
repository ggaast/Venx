use core::{
    iter::Rev,
    ops::{Range, RangeInclusive},
};

use spirv_std::glam::{uvec3, UVec3};

use crate::{
    plat::{layer::layer::Layer, node::Node, raw_plat::RawPlat, stack::EStack},
    utils::l2s,
};

/// Data-type used by traversal callbacks
#[derive(Clone)]
pub struct Props<'a> {
    /// Position of node in global 3d coords
    pub position: &'a UVec3,
    /// If false, than position is always UVec3::ZERO (Makes algorithm a bit faster)
    pub positioned: bool,
    /// Idx of parent node for given node. If 0, than there is no parents (works only for root node)
    pub parent_idx: &'a usize,
    /// Actual Node data
    pub node: &'a Node,
    /// Node idx
    pub node_idx: usize,
    /// Level of node
    pub level: usize,
    /// Forwarded entry index
    pub entry: u32,
    /// By default each callback prop has `drop_tree = false`.
    /// If you want to drop traversing of current node and all its children, set to `true`
    /// Be aware, it wont drop traversal of entire graph
    pub drop_tree: bool,
}

impl Props<'_> {
    pub fn local_p(&self) -> UVec3 {
        todo!()
    }
}

impl RawPlat<'_> {
    // pub fn traverse_unpositioned<F>(
    //     &self,
    //     layer_opts: LayerOpts,
    //     entry_opts: EntryOpts,
    //     callback: &mut F,
    // ) where
    //     F: FnMut(&mut Props),
    // {
    //     todo!()
    //     // Iterate over all layers and nodes
    //     // self.opts(
    //     //     None,
    //     //     layer_opts,
    //     //     entry_opts,
    //     //     true,
    //     //     &mut |_plat, (layer, layer_id), entry| {
    //     //         layer.traverse(
    //     //             entry,
    //     //             entry as usize,
    //     //             UVec3::ZERO,
    //     //             false,
    //     //             self.depth,
    //     //             callback,
    //     //         );
    //     //         return None as Option<()>;
    //     //     },
    //     // );
    // }

    // /// Traversing all nodes on all levels with voxel overlapping
    // /// layers and voxels can overlap
    // /// So if you specify a single layer, there are no overlaps
    // /// Also region_position is just some value in global space within this region
    // /// Dont traverse from level == depth, use normal `traverse`
    // pub fn traverse_region<F>(
    //     &self,
    //     region_position: UVec3,
    //     region_level: usize,
    //     entry_opts: EntryOpts,
    //     layer_opts: LayerOpts,
    //     callback: &mut F,
    // ) where
    //     F: FnMut(&mut Props),
    // {
    //     let fork_level = 4;
    //     assert!(region_level > fork_level);

    //     for layer_idx in 0..4 {
    //         let layer = &self[layer_idx];

    //         let node_idx =
    //             layer.get_node_idx_gpu(region_position * l2s(region_level), region_level);

    //         if node_idx != 0 {
    //             layer.traverse(0, node_idx, UVec3::ZERO, true, region_level, callback)
    //         }
    //     }
    // }
}

impl Layer<'_> {
    /// Depth-first traversal of layer.
    /// `entry: u32`, `from_node_position: UVec3` are used to adjust data in `Props`
    pub fn traverse<F>(
        &self,
        entry: u32,
        from_node_idx: usize,
        from_node_position: UVec3,
        positioned: bool,
        from_level: usize,
        callback: &mut F,
    ) where
        F: FnMut(&mut Props),
    {
        // TODO: use assert!
        if cfg!(feature = "bitcode_support") {
            assert_ne!(from_node_idx, 0);
        }

        // Emulate stack with max depth 21 (max graph depth)
        // Why? This code should compile to SpirV
        let mut stack: EStack<(
            /* 0 node_idx */
            usize,
            /* 1 parent_idx */
            usize,
            /* 2 node_position */
            UVec3,
            /* level */
            usize,
            /* voxel_id */
            usize,
            /* index (progress of iterator in specific node) */
            usize,
        )> = EStack::new((from_node_idx, 0, from_node_position, from_level, 0, 0));

        loop {
            // Read without pulling it
            let (node_idx, parent_idx, mut position, level, voxel_id, index) = stack.read();
            // Exit
            if *index > 7 && *level == from_level {
                break;
            }

            // Some cache going on here
            let node = &self[*node_idx];

            if node.is_fork() {
                if *index % 2 != 0 {
                    panic!()
                }

                // Out of bound
                if *index == 8 {
                    let flag = node.flag;
                    if flag > 0 {
                        // Switch to next fork in chain
                        *node_idx = flag as usize;
                        // Reset index
                        *index = 0;

                        continue;
                    } else if flag == -3 {
                        stack.pop();
                        continue;
                    } else {
                        panic!()
                    }
                }
                let voxel_id = &node.children[*index];
                let child_id = &node.children[*index + 1];

                *index += 2;

                if *child_id != 0 {
                    // if *index == 4 {
                    //     panic!();
                    // }
                    let (node_idx, level) = (node_idx.clone(), level.clone());

                    stack.push((
                        *child_id as usize,
                        node_idx,
                        position.clone(),
                        level,
                        *voxel_id as usize,
                        0,
                    ));

                    continue;
                } else {
                    // Exit fork chain
                    stack.pop();
                    continue;
                }
            }

            // Iterated over all children
            if *index > 7 {
                stack.pop();
                continue;
            }

            // Call for each enter once
            // If remove, it will call this callback 7 extra times
            if *index == 0 {
                let mut props = Props {
                    // TODO: Make use of positions
                    position: &position,
                    positioned,
                    parent_idx: &parent_idx,
                    node: &node,
                    node_idx: *node_idx,
                    entry: *voxel_id as u32,
                    level: *level,
                    drop_tree: false,
                };

                // let ret = callback(props);
                callback(&mut props);

                // Drop subtree traversal
                if props.drop_tree || *level == 0 {
                    stack.pop();
                    continue;
                }
            }

            let size = l2s(*level) / 2;

            // Actual node idx in layer.nodes
            let child_id = &node.children[*index];

            // Increment ahead, so if child_id == 0, it will still do some progress
            *index += 1;

            if *child_id != 0 && *level > 0 {
                // TODO: Profile, it might be slow to handle position this way
                if positioned {
                    position += Node::get_child_position(*index as u32 - 1) * size;
                }

                // TODO: Do we need this cache?
                let (node_idx, level, voxel_id) =
                    (node_idx.clone(), level.clone(), voxel_id.clone());

                stack.push((
                    *child_id as usize,
                    node_idx,
                    position.clone(),
                    level - 1,
                    voxel_id,
                    0,
                ));
            }
        }
    }
    /// Very smarty method ^_^ Its been rewritten so many times, that i have already lost the count.
    ///
    /// I think i have never seen more complicated control flow :/
    ///
    /// Its very optimal and has just single loop, which produces reasonable spv shaders
    ///
    /// Positioned depth first traversal
    ///
    /// `levels`: 0..depth is full traverse
    ///
    /// `levels`: 1..5 and `from_node_position`: (x, y, z) will traverse just region given on specified position and level
    /// until level 1
    pub fn traverse_new<F>(
        &self,
        mut from_node_position: UVec3,
        levels: RangeInclusive<usize>,
        mut callback: F,
    ) where
        F: FnMut(Props),
    {
        let from_level = *levels.end();
        let until_level = *levels.start();
        let fork_level = 4;

        from_node_position *= l2s(from_level);

        assert!(from_level >= 5);

        let node_idx = self.get_node(from_node_position, from_level, None);

        if node_idx.is_none() {
            return;
        }

        // Emulate stack with max depth 21 (max graph depth)
        // (Depth is bounded to [NodeAddr], which is essentially single u64,
        // each index is 3 bits. So it can be only 21 indexes in single u64)
        // Why? This code should compile to SpirV
        let mut stack: EStack<(
            /* 0 node_idx */
            usize,
            /* 1 parent_idx */
            usize,
            /* 2 node_position */
            UVec3,
            /* level */
            usize,
            /* voxel_id */
            usize,
            /* index (progress of iterator in specific node) */
            usize,
        )> = EStack::new((node_idx.node_idx, 0, from_node_position, from_level, 0, 0));

        loop {
            // Read without pulling it
            let (node_idx, parent_idx, mut position, ref level, voxel_id, index) = stack.read();

            let level = *level;
            // Exit
            if *index > 7 && level == from_level {
                break;
            }

            if *index > 7 && level < 3 {
                stack.pop();
                continue;
            }

            // Iterated over all children
            if *index > 7 {
                stack.pop();
                continue;
            }

            // Magic happening here :)
            // Child node idx in case its normal Node
            // Child 0-256 on level1 in case its level-2 Node
            // Child 0-1 on level0
            let child = if level == 2 {
                // Return index between 0 and 256 to NodeL2
                self.level_2[*node_idx].index_l1(*index)
            } else if level == 1 {
                // We have all data incoded in node_idx.
                // So we will just use it to determine is there any voxel (1) or not (0)
                // TODO: Move to [NodeL2] struct
                ((*node_idx & (1 << (*index))) >> (*index)) as u32
                //                             ^^^^^^^^^^^^ this here makes our child nice 0 or 1
            } else {
                self[*node_idx][*index] as u32
            };

            if child != 0 && level > 0 {
                let size = l2s(level) / 2;
                let mut push_position = position + Node::get_child_position(*index as u32) * size;
                let mut push_voxel_id: usize = *voxel_id;
                let mut push_level: usize = level - 1;
                let mut push_node_idx: usize = child as usize;
                let mut call_closure: bool = true;
                let push_index: usize = 0;
                let push_parent_idx: usize = *node_idx;

                if level == fork_level && self[*node_idx].is_fork() {
                    let node = &self[*node_idx];

                    if !node.is_fork() {
                        panic!("WTF?");
                    }

                    push_voxel_id = node.children[*index] as usize;
                    push_node_idx = node.children[*index + 1] as usize;
                    call_closure = false;
                    push_level = level;
                    push_position = position;
                    *index += 2;

                    // Out of bound
                    if *index > 7 {
                        let flag = node.flag;
                        if flag > 0 {
                            // Switch to next fork in chain
                            *node_idx = flag as usize;
                            // Reset index
                            *index = 0;
                        } else if flag == -3 {
                            // All done
                            stack.pop();
                        } else {
                            panic!()
                        }
                    }
                } else if level == fork_level && *voxel_id == 0 {
                    panic!();
                } else {
                    *index += 1;
                }

                if call_closure {
                    let props = Props {
                        position: &push_position,
                        positioned: true,
                        parent_idx: &push_parent_idx,
                        node: &Node::default(),
                        node_idx: push_node_idx as usize,
                        entry: push_voxel_id as u32,
                        level: push_level,
                        drop_tree: false,
                    };
                    callback(props);
                }

                if level > 1 {
                    stack.push((
                        push_node_idx,
                        push_parent_idx,
                        push_position,
                        push_level,
                        push_voxel_id,
                        push_index,
                    ));
                }
            } else if level == fork_level && self[*node_idx].is_fork() {
                // All done
                stack.pop();
                continue;
            } else {
                *index += 1;
            }
        }
    }

    /// Depth-first traversal of layer.
    /// `entry: u32`, `from_node_position: UVec3` are used to adjust data in `Props`
    pub fn traverse_gpu<F>(
        &self,
        entry: u32,
        from_node_idx: usize,
        from_node_position: UVec3,
        positioned: bool,
        from_level: usize,
        mut callback: F,
    ) where
        // level, entry, position
        F: FnMut((usize, usize, UVec3)),
    {
        // TODO: use assert!
        if cfg!(feature = "bitcode_support") {
            assert_ne!(from_node_idx, 0);
        }

        // Emulate stack with max depth 21 (max graph depth)
        // Why? This code should compile to SpirV
        let mut stack: EStack<(
            /* 0 node_idx */
            usize,
            /* 1 parent_idx */
            usize,
            /* 2 node_position */
            UVec3,
            /* level */
            usize,
            /* voxel_id */
            usize,
            /* index (progress of iterator in specific node) */
            usize,
        )> = EStack::new((from_node_idx, 0, from_node_position, from_level, 0, 0));

        loop {
            // Read without pulling it
            let (node_idx, parent_idx, mut position, level, voxel_id, index) = stack.read();
            // Exit
            if *index > 7 && *level == from_level {
                break;
            }

            // Some cache going on here
            let node = &self[*node_idx];

            if node.is_fork() {
                if *index % 2 != 0 {
                    panic!()
                }

                // Out of bound
                if *index == 8 {
                    let flag = node.flag;
                    if flag > 0 {
                        // Switch to next fork in chain
                        *node_idx = flag as usize;
                        // Reset index
                        *index = 0;

                        continue;
                    } else if flag == -3 {
                        stack.pop();
                        continue;
                    } else {
                        panic!()
                    }
                }
                let voxel_id = &node.children[*index];
                let child_id = &node.children[*index + 1];

                *index += 2;

                if *child_id != 0 {
                    // if *index == 4 {
                    //     panic!();
                    // }
                    let (node_idx, level) = (node_idx.clone(), level.clone());

                    stack.push((
                        *child_id as usize,
                        node_idx,
                        position.clone(),
                        level,
                        *voxel_id as usize,
                        0,
                    ));

                    continue;
                } else {
                    // Exit fork chain
                    stack.pop();
                    continue;
                }
            }

            // Iterated over all children
            if *index > 7 {
                stack.pop();
                continue;
            }

            // Call for each enter once
            // If remove, it will call this callback 7 extra times
            if *index == 0 {
                // let mut props = Props {
                //     // TODO: Make use of positions
                //     position: &position,
                //     positioned,
                //     parent_idx: &parent_idx,
                //     node: &node,
                //     node_idx: *node_idx,
                //     entry: *voxel_id as u32,
                //     level: *level,
                //     drop_tree: false,
                // };

                // let ret = callback(props);
                callback((*level, *voxel_id, position));

                // // Drop subtree traversal
                // if props.drop_tree || *level == 0 {
                //     stack.pop();
                //     continue;
                // }
            }

            let size = l2s(*level) / 2;

            // Actual node idx in layer.nodes
            let child_id = &node.children[*index];

            // Increment ahead, so if child_id == 0, it will still do some progress
            *index += 1;

            if *child_id != 0 && *level > 0 {
                // TODO: Profile, it might be slow to handle position this way
                if positioned {
                    position += Node::get_child_position(*index as u32 - 1) * size;
                }

                // TODO: Do we need this cache?
                let (node_idx, level, voxel_id) =
                    (node_idx.clone(), level.clone(), voxel_id.clone());

                stack.push((
                    *child_id as usize,
                    node_idx,
                    position.clone(),
                    level - 1,
                    voxel_id,
                    0,
                ));
            }
        }
    }
}
#[cfg(feature = "bitcode_support")]
#[cfg(test)]
mod tests {
    extern crate alloc;
    extern crate std;
    use crate::{plat::layer::layer::Lr, test_utils::set_rand_plat, *};
    use core::borrow::{Borrow, BorrowMut};
    use std::{collections::HashSet, dbg, println};

    use alloc::{
        borrow::ToOwned,
        boxed::Box,
        vec::{self, Vec},
    };
    use rand::{thread_rng, Rng};
    use spirv_std::glam::{uvec3, UVec3};

    use crate::{
        plat::{
            chunk::chunk::Chunk,
            node::{Node, NodeAddr},
            raw_plat::{LayerIndex, RawPlat},
        },
        utils::l2s,
    };

    use self::test_utils::gen_rand_mtx;

    #[test]
    fn traverse_region_zero() {
        quick_raw_plat!(plat, depth 6);
        // Base
        plat[Lr::BASE].set(uvec3(14, 14, 14), 1);
        plat[Lr::BASE].set(uvec3(0, 0, 0), 2);
        plat[Lr::BASE].set(uvec3(5, 15, 5), 3);
        // Out
        plat[Lr::BASE].set(uvec3(40, 40, 40), 1);

        let mut seq = alloc::vec![];
        traverse_region!(plat, rng 0..=5, pos UVec3::ZERO, {
                |p|{
                    if p.level == 0 {
                    seq.push(p.position.clone());
                }
            }
        });

        assert_eq!(seq, [uvec3(14, 14, 14), uvec3(0, 0, 0), uvec3(5, 15, 5),]);
    }

    #[test]
    fn traverse_region_one() {
        quick_raw_plat!(plat, depth 6);
        // Base
        plat[Lr::BASE].set(uvec3(14, 14, 14), 1);
        plat[Lr::BASE].set(uvec3(0, 0, 0), 2);
        plat[Lr::BASE].set(uvec3(5, 15, 5), 3);
        // Out
        plat[Lr::BASE].set(uvec3(40, 40, 40), 1);

        let mut seq = alloc::vec![];
        traverse_region!(plat, rng 0..=5, pos UVec3::ONE, {
                |p|{
                    if p.level == 0 {
                    seq.push(p.position.clone());
                }
            }
        });

        assert_eq!(seq, [uvec3(40, 40, 40),]);
    }

    #[test]
    fn traverse_region_one_deep() {
        quick_raw_plat!(plat, depth 15);
        // Base
        plat[Lr::BASE].set(uvec3(14, 14, 14), 1);
        plat[Lr::BASE].set(uvec3(0, 0, 0), 2);
        plat[Lr::BASE].set(uvec3(5, 15, 5), 3);
        // Out
        plat[Lr::BASE].set(uvec3(40, 40, 40), 1);

        let mut seq = alloc::vec![];
        traverse_region!(plat, rng 0..=5, pos UVec3::ONE, {
                |p|{
                    if p.level == 0 {
                    seq.push(p.position.clone());
                }
            }
        });

        assert_eq!(seq, [uvec3(40, 40, 40),]);
    }

    // #[test]
    // fn traverse_check_props_node_idx() {
    //     quick_raw_plat!(plat, depth 6, len 1_000);

    //     plat[0].set((0, 12, 8).into(), 22);
    //     plat[0].set((1, 12, 8).into(), 32);
    //     plat[0].set((2, 12, 8).into(), 52);
    //     plat[0].set((5, 12, 4).into(), 12);
    //     plat[0].set((1, 2, 11).into(), 2);

    //     // plat.traverse(super::EntryOpts::All, LayerOpts::All, &mut |p| {
    //     //     assert_eq!(
    //     //         plat[0].get_node_idx_gpu(*p.position, p.level, Some(p.entry as usize)),
    //     //         p.node_idx
    //     //     )
    //     // });
    // }

    // #[test]
    // fn traverse_check_props_node() {
    //     quick_raw_plat!(plat, depth 6, len 1_000);

    //     plat[0].set((0, 12, 8).into(), 22);
    //     plat[0].set((1, 12, 8).into(), 32);
    //     plat[0].set((2, 12, 8).into(), 52);
    //     plat[0].set((5, 12, 4).into(), 12);
    //     plat[0].set((1, 2, 11).into(), 2);

    //     // plat.traverse(super::EntryOpts::All, LayerOpts::All, &mut |p| {
    //     //     assert_eq!(
    //     //         &plat[0].nodes[plat[0].get_node_idx_gpu(*p.position, p.level, None)],
    //     //         p.node
    //     //     )
    //     // });
    // }

    // #[test]
    // fn traverse_check_props_parent_idx() {
    //     quick_raw_plat!(plat, depth 6, len 100);

    //     plat[0].set((0, 12, 8).into(), 2);
    //     plat[0].set((1, 12, 8).into(), 2);
    //     plat[0].set((2, 12, 8).into(), 2);
    //     plat[0].set((5, 12, 4).into(), 2);
    //     plat[0].set((1, 2, 11).into(), 2);

    //     let mut seq = alloc::vec![];
    //     let mut parent_helper = [UVec3::ZERO; 8];
    //     traverse!(plat, {
    //         |p| {
    //             parent_helper[p.level] = *p.position;

    //             seq.push((*p.parent_idx, (p.level), p.node_idx));
    //         }
    //     });

    //     // assert_eq!(seq, alloc::vec![])
    // }

    #[test]
    fn traverse_many() {
        quick_raw_plat!(plat, depth 6, len 1_000);

        // 0 0 0
        plat[0].set((0, 12, 8).into(), 22);
        plat[0].set((1, 12, 8).into(), 32);
        plat[0].set((2, 12, 8).into(), 52);
        plat[0].set((5, 12, 4).into(), 12);
        plat[0].set((1, 2, 11).into(), 2);

        // 1 1 0
        plat[0].set((34, 34, 8).into(), 32);
        plat[0].set((35, 32, 8).into(), 22);
        plat[0].set((50, 50, 4).into(), 12);
        plat[0].set((55, 60, 4).into(), 2);

        // 1 0 1
        plat[0].set((56, 2, 60).into(), 22);
        plat[0].set((57, 2, 60).into(), 32);
        plat[0].set((58, 2, 60).into(), 52);
        plat[0].set((59, 2, 60).into(), 12);
        plat[0].set((60, 2, 60).into(), 11);

        // 0 0 1
        plat[0].set((2, 16, 60).into(), 52);

        let mut seq = alloc::vec![];

        traverse!(plat, {
            |p| {
                if p.level == 0 {
                    seq.push(*p.position);
                }
            }
        });

        assert_eq!(
            &seq,
            &alloc::vec![
                uvec3(0, 12, 8),
                uvec3(1, 12, 8),
                uvec3(2, 12, 8),
                uvec3(5, 12, 4),
                uvec3(1, 2, 11),
                //
                uvec3(34, 34, 8),
                uvec3(35, 32, 8),
                uvec3(50, 50, 4),
                uvec3(55, 60, 4),
                //
                uvec3(2, 16, 60),
                //
                uvec3(56, 2, 60),
                uvec3(57, 2, 60),
                uvec3(58, 2, 60),
                uvec3(59, 2, 60),
                uvec3(60, 2, 60)
            ]
        );
    }

    #[test]
    fn traverse_check_path_by_levels() {
        quick_raw_plat!(plat, depth 10, len 10);

        plat[0].set((7, 20, 5).into(), 52);

        let mut seq = alloc::vec![];

        traverse!(plat, {
            |p| {
                seq.push(p.level);
            }
        });

        assert_eq!(&seq, &alloc::vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn traverse_check_path_by_flags() {
        quick_raw_plat!(plat, depth 10, len 160);

        set_rand_plat::<16>(&mut plat, 99);

        let mut seq = alloc::vec![];

        traverse!(plat, {
            |p| {
                if p.level > 2 {
                    seq.push(plat[0][p.node_idx].flag);
                    if plat[0][p.node_idx].flag != 0 {
                        dbg!(p.level);
                    }
                }
            }
        });

        assert_eq!(&seq, &alloc::vec![0; seq.len()]);
    }

    #[test]
    fn traverse_check_path_by_node_idx() {
        quick_raw_plat!(plat, depth 10, len 10060);

        set_rand_plat::<32>(&mut plat, 90);

        let mut seq = alloc::vec![];
        let mut seq2 = alloc::vec![];

        traverse!(plat, {
            |p| {
                if p.level > 2 {
                    // let node = plat[0][p.node_idx];
                    // assert!(!node.is_fork());

                    seq.push(p.node_idx);
                    seq2.push(
                        plat[0]
                            .get_node(
                                *p.position,
                                p.level,
                                if p.entry == 0 {
                                    None
                                } else {
                                    Some(p.entry as usize)
                                },
                            )
                            .node_idx,
                    );
                }
            }
        });

        assert_eq!(&seq, &seq2);
    }

    #[test]
    fn traverse_check_count() {
        quick_raw_plat!(plat, depth 10, len 200_060);

        set_rand_plat::<64>(&mut plat, 90);

        let mut count = 0;

        traverse!(plat, {
            |p| {
                // Only nodes on levels 3..depth are actually stored
                // nodes on level 2 stored separetely on layer.level_2
                // 0 and 1 levels dont actually exist. So we dont count it
                if p.level > 2 {
                    count += 1;
                }
            }
        });

        let mut fork_count = 0;

        for node in plat[0].nodes.iter() {
            if node.is_fork() {
                fork_count += 1;
            }
        }

        dbg!(fork_count);

        assert_ne!(fork_count, 0);
        assert_ne!(count, 0);
        assert_ne!(plat[0].free(), 0);
        assert_ne!(plat[0].nodes.len(), 0);
        assert_eq!(plat[0].nodes.len(), plat[0].free() + count + fork_count);
    }

    #[test]
    fn traverse_check_register() {
        // quick_raw_plat!(plat, depth 10, len 200_060);

        // set_rand_plat::<64>(&mut plat, 90);

        // let mut register = HashSet::new();

        // traverse!(plat, {
        //     |p| {
        //         // Only nodes on levels 3..depth are actually stored
        //         // nodes on level 2 stored separetely on layer.level_2
        //         // 0 and 1 levels dont actually exist. So we dont count it
        //         if p.level > 2 {
        //             register.insert(p.node_idx);
        //         }
        //     }
        // });

        // let mut fork_count = 0;

        // for node in plat[0].nodes.iter() {
        //     if node.is_fork() {
        //         fork_count += 1;
        //     }
        // }

        // dbg!(fork_count);

        // assert_ne!(fork_count, 0);
        // assert_ne!(count, 0);
        // assert_ne!(plat[0].free(), 0);
        // assert_ne!(plat[0].nodes.len(), 0);
        // assert_eq!(plat[0].nodes.len(), plat[0].free() + count + fork_count);
    }

    #[test]
    fn traverse_compare_count_with_mtx() {
        quick_raw_plat!(plat, depth 10, len 200_060);

        let mtx = set_rand_plat::<64>(&mut plat, 90);
        let mut voxels_in_mtx = 0;

        for l in mtx.iter() {
            for l in l {
                for v in l {
                    if *v != 0 {
                        voxels_in_mtx += 1;
                    }
                }
            }
        }

        let mut voxels_from_traverse = 0;

        traverse!(plat, {
            |p| {
                if p.level == 0 {
                    voxels_from_traverse += 1;
                }
            }
        });

        assert_eq!(voxels_in_mtx, voxels_from_traverse);
    }

    #[test]
    fn traverse_check_leaked_forks() {
        quick_raw_plat!(plat, depth 10, len 1000_060);

        set_rand_plat::<64>(&mut plat, 50);

        let mut leaked = alloc::vec![];

        traverse!(plat, {
            |p| {
                if p.level > 2 {
                    let node = plat[0][p.node_idx];
                    if node.is_fork() {
                        leaked.push((node, p.node_idx, p.level));
                    }
                }
            }
        });

        assert_eq!(leaked, alloc::vec![]);
    }

    #[test]
    fn traverse_check_l1_n_l0() {
        quick_raw_plat!(plat, depth 10, len 100);

        plat[0].set((0, 12, 8).into(), 22);
        plat[0].set((1, 12, 8).into(), 32);
        plat[0].set((2, 12, 8).into(), 52);
        plat[0].set((5, 12, 4).into(), 12);
        plat[0].set((1, 2, 11).into(), 2);

        traverse!(plat, {
            |p| {
                if p.level == 1 {
                    assert!(p.node_idx < 256);
                } else if p.level == 0 {
                    assert_eq!(p.node_idx, 1);
                }
            }
        });
    }

    #[test]
    fn test_traverse_region_positions_deep() {
        quick_raw_plat!(plat, depth 8, len 1_000);

        // 0 0 0
        plat[0].set((0, 12, 8).into(), 22);
        plat[0].set((1, 12, 8).into(), 32);
        plat[0].set((2, 12, 8).into(), 52);
        plat[0].set((5, 12, 4).into(), 12);
        plat[0].set((1, 2, 11).into(), 2);

        // 1 1 0
        plat[0].set((34, 34, 8).into(), 32);
        plat[0].set((35, 32, 8).into(), 22);
        plat[0].set((50, 50, 4).into(), 12);
        plat[0].set((55, 60, 4).into(), 2);

        // 1 0 1
        plat[0].set((56, 2, 60).into(), 22);
        plat[0].set((57, 2, 60).into(), 32);
        plat[0].set((58, 2, 60).into(), 52);
        plat[0].set((59, 2, 60).into(), 12);
        plat[0].set((60, 2, 60).into(), 12);

        // 0 0 1
        plat[0].set((2, 16, 60).into(), 52);

        let mut seq = alloc::vec![];

        traverse_region!(plat, rng 0..=5, pos uvec3(0, 0, 0), { |p| {
            if p.level == 0 {
                seq.push(*p.position);
            }
        }});

        assert_eq!(
            &seq,
            &alloc::vec![
                uvec3(0, 12, 8),
                uvec3(1, 12, 8),
                uvec3(2, 12, 8),
                uvec3(5, 12, 4),
                uvec3(1, 2, 11)
            ]
        );

        let mut seq = alloc::vec![];

        traverse_region!(plat, rng 0..=5, pos uvec3(1, 1, 0), {
            |p|{
                if p.level == 0 {
                seq.push(*p.position);
        }}});

        assert_eq!(
            &seq,
            &alloc::vec![
                uvec3(34, 34, 8),
                uvec3(35, 32, 8),
                uvec3(50, 50, 4),
                uvec3(55, 60, 4),
            ]
        );

        let mut seq = alloc::vec![];

        traverse_region!(plat, rng 0..=5, pos uvec3(1, 0, 1), {|p|{                if p.level == 0 {
            seq.push(*p.position);
        }}});

        assert_eq!(
            &seq,
            &alloc::vec![
                uvec3(56, 2, 60),
                uvec3(57, 2, 60),
                uvec3(58, 2, 60),
                uvec3(59, 2, 60),
                uvec3(60, 2, 60)
            ]
        );

        let mut seq = alloc::vec![];
        traverse_region!(plat, rng 0..=5, pos uvec3(0, 0, 1), {|p|{                if p.level == 0 {
            seq.push(*p.position);
        }}});

        assert_eq!(&seq, &alloc::vec![uvec3(2, 16, 60)]);
    }

    #[test]
    fn traverse_region_full() {
        quick_raw_plat!(plat, depth 7, len 100_000_000);

        let mtx = gen_rand_mtx::<128>(50);

        for x in 0..128 {
            for y in 0..128 {
                for z in 0..128 {
                    let voxel_id = mtx[x][y][z];
                    plat[0].set(uvec3(x as u32, y as u32, z as u32), voxel_id);
                }
            }
        }
        traverse_region!(plat, rng 0..=6, pos UVec3::ZERO, {|p|{
            if p.level == 0 {
            assert_eq!(
                p.entry,
                mtx[p.position.x as usize][p.position.y as usize][p.position.z as usize]
            );

            assert_eq!(p.entry, plat.get_voxel(*p.position).voxel_id as u32);
        }}});

        // traverse_region!(plat, rng 0..=6, pos UVec3::ZERO, {|p|{           }});

        traverse_region!(plat, rng 0..=6, pos uvec3(0, 1, 0), {|p|{                  if p.level == 0 {
            assert_eq!(
                p.entry,
                mtx[p.position.x as usize][p.position.y as usize]
                    [p.position.z as usize]
            );

            assert_eq!(
                p.entry,
                plat.get_voxel(*p.position).voxel_id as u32
            );
        }}});

        traverse_region!(plat, rng 0..=6, pos uvec3(1, 1, 1), {|p|{                      if p.level == 0 {
            assert_eq!(
                p.entry,
                mtx[p.position.x as usize][p.position.y as usize]
                    [p.position.z as usize ]
            );

            assert_eq!(
                p.entry,
                plat.get_voxel(*p.position).voxel_id as u32
            );
        }     }});

        traverse_region!(plat, rng 0..=5, pos uvec3(1, 1, 1), {|p|{     if p.level == 0 {
            assert_eq!(
                p.entry,
                mtx[p.position.x as usize][p.position.y as usize]
                    [p.position.z as usize]
            );

            assert_eq!(
                p.entry,
                plat.get_voxel(*p.position).voxel_id as u32
            );
        }       }});

        traverse_region!(plat, rng 0..=5, pos  uvec3(2, 2, 2), {|p|{                     if p.level == 0 {
            assert_eq!(
                p.entry,
                mtx[p.position.x as usize][p.position.y as usize]
                    [p.position.z as usize ]
            );

            assert_eq!(
                p.entry,
                plat.get_voxel(*p.position).voxel_id as u32
            );
        }      }});

        traverse_region!(plat, rng 0..=5, pos         uvec3(3, 2, 1), {|p|{      
            if p.level == 0 {
                assert_eq!(
                    p.entry,
                    mtx[p.position.x as usize][p.position.y as usize]
                        [p.position.z as usize]
                );

                assert_eq!(
                    p.entry,
                    plat.get_voxel(*p.position ).voxel_id as u32
                );
        } }});
    }

    #[test]
    #[allow(unused_braces)]
    fn traverse_noise_64x64x64() {
        quick_raw_plat!(plat, depth 6, len 589834, len2 262145, lenrest 2);

        use rand::seq::SliceRandom;
        let mut ids: Vec<u32> = (1..=(64 * 64 * 64)).collect();
        ids.shuffle(&mut thread_rng());

        let mut idx = 0;
        for x in 0..64 {
            for y in 0..64 {
                for z in 0..64 {
                    plat[0].set(uvec3(x, y, z), ids[idx]);
                    idx += 1;
                }
            }
        }

        traverse!(plat, { |_p| {} });
    }

    #[test]
    fn count_on_each_level() {
        /*
            Nodes:
            Free-head - 1
            l6 - 1
            l5 - 8
            fork - 8^2 * (16^3 / 4) (2^6 * 2^10 = 2^14 = 65536)
            l4 - 8^2 * 16^3 (2^6 * 2^12 = 2^18 = 262144)
            l3 - 2^18 (262144)

           Total: 589834

           Nodes L2:
            Free-head - 1
            l2: 2^18 (262144)

           Total: 262145

        */
        quick_raw_plat!(plat, depth 6, len 589834, len2 262145, lenrest 2);

        use rand::seq::SliceRandom;
        let mut ids: Vec<u32> = (1..=(64 * 64 * 64)).collect();
        ids.shuffle(&mut thread_rng());

        let mut idx = 0;
        for x in 0..64 {
            for y in 0..64 {
                for z in 0..64 {
                    plat[0].set(uvec3(x, y, z), ids[idx]);
                    idx += 1;
                }
            }
        }
        let mut counts = [0; 8];

        traverse!(plat, {
            |p| {
                counts[p.level] += 1;
            }
        });

        assert_eq!(counts, [262144, 262144, 262144, 262144, 262144, 8, 1, 1]);
    }

    #[test]
    fn test_gen_rand_mtx() {
        let _ = gen_rand_mtx::<2>(50);
        let _ = gen_rand_mtx::<4>(0);
        let _ = gen_rand_mtx::<8>(100);
        let _ = gen_rand_mtx::<16>(50);
        let _ = gen_rand_mtx::<32>(50);
        let _ = gen_rand_mtx::<64>(50);
    }

    #[test]
    fn traverse_single() {
        quick_raw_plat!(plat, depth 5);
        // Base
        plat[Base].set(uvec3(7, 20, 5), 1);

        let mut seq = alloc::vec![];

        traverse!(plat, {
            |p| {
                if p.level == 0 {
                    seq.push((p.position.clone(), p.entry));
                }
            }
        });

        assert_eq!(seq, [(uvec3(7, 20, 5), 1)]);
    }

    #[test]
    fn traverse() {
        quick_raw_plat!(plat, depth 5);
        // Base
        plat[Base].set(uvec3(14, 14, 14), 1);
        plat[Base].set(uvec3(0, 0, 0), 1);
        plat[Base].set(uvec3(5, 15, 5), 1);
        plat[Base].set(uvec3(0, 10, 0), 1);
        plat[Base].set(uvec3(15, 15, 15), 1);

        let mut seq = alloc::vec![];

        traverse!(plat, {
            |p| {
                if p.level == 0 {
                    seq.push(p.position.clone());
                }
            }
        });

        assert_eq!(
            seq,
            [
                uvec3(0, 0, 0),
                uvec3(0, 10, 0),
                uvec3(5, 15, 5),
                uvec3(14, 14, 14),
                uvec3(15, 15, 15)
            ]
        );
    }
}
