use glam::{uvec3, UVec3};

use crate::{chunk::chunk::Chunk, voxel::cpu::utils};

use super::voxel::Voxel;

impl Voxel {
    pub fn load_chunk(&self, position: UVec3, lod_level: u8) -> Option<Chunk> {
        let chunk_level = self.chunk_level;
        let mtx_size = 1 << (chunk_level - lod_level);
        let mut chunk = Chunk {
            mtx: vec![vec![vec![false; mtx_size]; mtx_size]; mtx_size],
            position,
            lod_level,
            chunk_level,
        };

        let chunk_size = utils::lvl_to_size::lvl_to_size(chunk_level);
        if let Some(chunk_idx) = self.topology.get(chunk_level, position * chunk_size) {
            self.traverse_from(chunk_idx, uvec3(0, 0, 0), |branch, idx, pos| {
                if branch.level() > lod_level {
                } else if branch.level() == lod_level {
                    chunk.set(pos, true);
                } else {
                    return false;
                }
                true
            });
            return Some(chunk);
        }
        None
    }
}

#[test]
fn load_chunk_test() {
    let mut vx = Voxel::new(5, 3, 5);

    vx.topology.set(uvec3(0, 0, 0), true);
    vx.topology.set(uvec3(0, 5, 0), true);
    vx.topology.set(uvec3(1, 1, 0), true);
    vx.topology.set(uvec3(1, 7, 0), true);
    // second chunk
    vx.topology.set(uvec3(0, 8, 0), true);

    let chunk = vx.load_chunk(UVec3::ZERO, 0).unwrap();
    assert!(chunk.mtx[0][0][0]);
    assert!(chunk.mtx[0][5][0]);
    assert!(chunk.mtx[1][1][0]);
    assert!(chunk.mtx[1][7][0]);

    // Should not be loaded
    assert!(!chunk.mtx[5][7][0]);
    assert!(!chunk.mtx[3][1][0]);
    assert!(!chunk.mtx[5][2][0]);
    assert!(!chunk.mtx[6][7][5]);

    // second chunk should not be loaded
    // assert!(!chunk.mtx[0][8][0]); panicing
}
