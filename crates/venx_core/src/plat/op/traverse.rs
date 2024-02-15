use spirv_std::glam::{uvec3, UVec3};

use crate::{
    plat::{layer::layer::Layer, node::Node, raw_plat::RawPlat, stack::EStack},
    utils::l2s,
};

use super::{EntryOpts, LayerOpts};

/// Data-type used by traversal callbacks
#[derive(Clone)]
pub struct Props<'a> {
    /// Position of node in global 3d coords
    /// If no initial position was specified in `traverse` method, it will be local
    pub position: &'a UVec3,
    /// If false, than position is always UVec3::ZERO (Makes algorithm a bit faster)
    pub positioned: bool,
    /// Idx of parent node for given node. If 0, than there is no parents (works only for root node)
    pub parent_idx: &'a usize,
    /// Actual Node data
    pub node: &'a Node,
    /// Level of node
    pub level: usize,
    /// Forwarded entry index
    pub entry: u32,
    /// By default each callback prop has `drop_tree = false`.
    /// If you want to drop traversing of current node and all its children, set to `true`
    /// Be aware, it wont drop traversal of entire graph
    pub drop_tree: bool,
}

impl RawPlat<'_> {
    /// Traverse through all voxels in world specified in arguments
    /// Algorithm goes from bottom to up, meaning that some voxels can overlap, in that case works recent-right rule.
    /// Return false in callback to drop traversing of subtree
    pub fn traverse<F>(&self, layer_opts: LayerOpts, entry_opts: EntryOpts, callback: &mut F)
    where
        F: FnMut(&mut Props),
    {
        // Iterate over all layers and nodes
        self.opts(
            None,
            layer_opts,
            entry_opts,
            true,
            &mut |plat, (layer, layer_id), entry| {
                layer.traverse(
                    entry,
                    // TODO: do something about this unsafe cringe
                    layer.entries[entry as usize],
                    UVec3::ZERO,
                    true,
                    self.depth,
                    callback,
                );
                return None as Option<()>;
            },
        );
    }

    pub fn traverse_unpositioned<F>(
        &self,
        layer_opts: LayerOpts,
        entry_opts: EntryOpts,
        callback: &mut F,
    ) where
        F: FnMut(&mut Props),
    {
        // Iterate over all layers and nodes
        self.opts(
            None,
            layer_opts,
            entry_opts,
            true,
            &mut |_plat, (layer, layer_id), entry| {
                layer.traverse(
                    entry,
                    entry as usize,
                    UVec3::ZERO,
                    false,
                    self.depth,
                    callback,
                );
                return None as Option<()>;
            },
        );
    }

    /// Traversing all nodes on all levels with voxel overlapping
    /// layers and voxels can overlap
    /// So if you specify a single layer, there are no overlaps
    /// Also region_position is just some value in global space within this region
    /// Dont traverse from level == depth, use normal `traverse`
    pub fn traverse_region<F>(
        &self,
        region_position: UVec3,
        region_level: usize,
        entry_opts: EntryOpts,
        layer_opts: LayerOpts,
        callback: &mut F,
    ) where
        F: FnMut(&mut Props),
    {
        let fork_level = 4;
        assert!(region_level > fork_level);

        for layer_idx in 0..4 {
            let layer = &self[layer_idx];

            let res = layer.get_node(region_position * l2s(region_level), region_level, None);

            if res.is_some() {
                layer.traverse(0, 2, UVec3::ZERO, true, region_level, callback)
            }
        }

        // self.opts(
        //     None,
        //     layer_opts,
        //     entry_opts,
        //     true,
        //     &mut |_plat, (layer, ..), entry| {
        //         // We need explicitly call it for all specified entries and layers. Otherwise it would find just one node with most priority.

        //         None as Option<()>
        //     },
        // );
    }
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
            /* index (progress of iterator in specific node) */
            usize,
        )> = EStack::new((from_node_idx, 0, from_node_position, from_level, 0));

        loop {
            // Read without pulling it
            let (node_idx, parent_idx, mut position, level, index) = stack.read();
            // Exit
            if *index > 7 && *level == from_level {
                break;
            }
            // Hit bottom and iterated over all children
            if *level == 0 && *index > 7 {
                stack.pop();
                continue;
            }
            // Iterated over all children
            if *index > 7 {
                stack.pop();
                continue;
            }
            // Some cache going on here
            let node = &self[*node_idx];

            if node.is_fork() {
                // Deadend
                if *index == 1 {
                    stack.pop();
                    continue;
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
                    }
                }

                let child_id = &node.children[*index + 1];

                *index += 2;

                if *child_id != 0 {
                    let (node_idx, level) = (node_idx.clone(), level.clone());

                    stack.push((*child_id as usize, node_idx, position.clone(), level - 1, 0));

                    continue;
                } else {
                    // Indicate that we should exit this traversal
                    *index = 1;
                }
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
                    entry: entry,
                    level: *level,
                    drop_tree: false,
                };

                // let ret = callback(props);
                callback(&mut props);

                // Drop subtree traversal
                if props.drop_tree {
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
                let (node_idx, level) = (node_idx.clone(), level.clone());

                stack.push((*child_id as usize, node_idx, position.clone(), level - 1, 0));
            }
        }
    }
}
#[cfg(feature = "bitcode_support")]
#[cfg(test)]
mod tests {
    extern crate alloc;
    extern crate std;
    use crate::*;
    use std::println;

    use alloc::{borrow::ToOwned, vec};
    use spirv_std::glam::{uvec3, UVec3};

    use crate::{
        plat::{
            chunk::chunk::Chunk,
            node::{Node, NodeAddr},
            op::{EntryOpts, LayerOpts},
            raw_plat::{LayerIndex, RawPlat},
        },
        utils::l2s,
    };

    #[test]
    fn traverse_region() {
        let mut base = ([Node::default(); 128], [0; 10]);
        let (mut tmp, mut schem, mut canvas) = (base.clone(), base.clone(), base.clone());
        let mut plat = RawPlat::new(
            6,
            5,
            5,
            (&mut base.0, &mut base.1),
            (&mut tmp.0, &mut tmp.1),
            (&mut schem.0, &mut schem.1),
            (&mut canvas.0, &mut canvas.1),
        );

        // Base
        plat[Base].set(uvec3(14, 14, 14), 1);
        plat[Base].set(uvec3(0, 0, 0), 2);
        plat[Base].set(uvec3(5, 15, 5), 3);
        plat[Base].set(uvec3(0, 10, 0), 1);

        // Canvas
        plat[Canvas].set(uvec3(15, 15, 15), 1);
        plat[Canvas].set(uvec3(0, 0, 0), 2);
        let mut seq = vec![];

        plat.traverse_region(
            UVec3::ZERO,
            5,
            super::EntryOpts::All,
            LayerOpts::All,
            &mut |props| {
                if props.level == 0 {
                    seq.push(props.position.clone());
                }
            },
        );
        assert_eq!(
            seq,
            [
                uvec3(0, 10, 0),
                uvec3(14, 14, 14),
                uvec3(0, 0, 0),
                uvec3(5, 15, 5),
                uvec3(15, 15, 15),
                uvec3(0, 0, 0)
            ]
        );
    }

    #[test]
    fn test_drop_tree() {
        todo!()
    }

    #[test]
    fn check_parent_nodes() {
        let mut base = ([Node::default(); 128], [0; 10]);
        let (mut tmp, mut schem, mut canvas) = (base.clone(), base.clone(), base.clone());
        let mut plat = RawPlat::new(
            5,
            5,
            5,
            (&mut base.0, &mut base.1),
            (&mut tmp.0, &mut tmp.1),
            (&mut schem.0, &mut schem.1),
            (&mut canvas.0, &mut canvas.1),
        );
        // Base
        plat[Base].set(uvec3(7, 20, 5), 1);

        let mut seq = vec![];

        plat[Base].traverse(
            1,
            plat[Base].entries[1],
            UVec3::ZERO,
            true,
            plat.depth,
            &mut |props| {
                seq.push(props.parent_idx.clone());
            },
        );

        // println!("{seq:?}");

        //let addr = NodeAddr::from_position(uvec3(7, 20, 5), 5);

        // // let mut right
        // for level in (0..=5).rev() {
        //     plat.base[addr.get_idx(level)];
        // }
        todo!()
        // assert_eq!(seq, vec![]);
    }

    #[test]
    fn traverse() {
        let mut base = ([Node::default(); 128], [0; 10]);
        let (mut tmp, mut schem, mut canvas) = (base.clone(), base.clone(), base.clone());
        let mut plat = RawPlat::new(
            5,
            5,
            5,
            (&mut base.0, &mut base.1),
            (&mut tmp.0, &mut tmp.1),
            (&mut schem.0, &mut schem.1),
            (&mut canvas.0, &mut canvas.1),
        );
        // Base
        plat[Base].set(uvec3(14, 14, 14), 1);
        plat[Base].set(uvec3(0, 0, 0), 2);
        plat[Base].set(uvec3(5, 15, 5), 3);
        plat[Base].set(uvec3(0, 10, 0), 1);

        // Canvas
        plat[Canvas].set(uvec3(15, 15, 15), 1);
        plat[Canvas].set(uvec3(0, 0, 0), 2);

        let mut seq = vec![];

        plat.traverse(LayerOpts::All, EntryOpts::All, &mut |props| {
            if props.level == 0 {
                seq.push(props.position.clone());
            }
        });

        // println!("{seq:?}");

        assert_eq!(
            seq,
            [
                uvec3(0, 10, 0),
                uvec3(14, 14, 14),
                uvec3(0, 0, 0),
                uvec3(5, 15, 5),
                uvec3(15, 15, 15),
                uvec3(0, 0, 0)
            ]
        );
    }

    #[test]
    fn traverse_layer_single() {
        let mut base = ([Node::default(); 128], [0; 10]);
        let (mut tmp, mut schem, mut canvas) = (base.clone(), base.clone(), base.clone());
        let mut plat = RawPlat::new(
            5,
            5,
            5,
            (&mut base.0, &mut base.1),
            (&mut tmp.0, &mut tmp.1),
            (&mut schem.0, &mut schem.1),
            (&mut canvas.0, &mut canvas.1),
        );
        // Base
        plat[Base].set(uvec3(7, 20, 5), 1);

        let mut seq = vec![];

        plat[Base].traverse(
            1,
            plat[Base].entries[1],
            UVec3::ZERO,
            true,
            plat.depth,
            &mut |props| {
                if props.level == 0 {
                    seq.push(props.position.clone());
                }
            },
        );

        // println!("{seq:?}");

        assert_eq!(seq, [uvec3(7, 20, 5)]);
    }

    #[test]
    fn traverse_layer() {
        let mut base = ([Node::default(); 128], [0; 10]);
        let (mut tmp, mut schem, mut canvas) = (base.clone(), base.clone(), base.clone());
        let mut plat = RawPlat::new(
            5,
            5,
            5,
            (&mut base.0, &mut base.1),
            (&mut tmp.0, &mut tmp.1),
            (&mut schem.0, &mut schem.1),
            (&mut canvas.0, &mut canvas.1),
        );
        // Base
        plat[Base].set(uvec3(14, 14, 14), 1);
        plat[Base].set(uvec3(0, 0, 0), 1);
        plat[Base].set(uvec3(5, 15, 5), 1);
        plat[Base].set(uvec3(0, 10, 0), 1);
        plat[Base].set(uvec3(15, 15, 15), 1);

        let mut seq = vec![];

        plat[Base].traverse(
            1,
            plat[Base].entries[1],
            UVec3::ZERO,
            true,
            plat.depth,
            &mut |props| {
                if props.level == 0 {
                    seq.push(props.position.clone());
                }
            },
        );

        // println!("{seq:?}");

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
