use glam::{UVec3, Vec3, Vec4};
use venx_core::{plat::chunk::chunk::Chunk, utils::Grid};

use crate::plat::normal::mesh::Mesh;

pub trait LoadInterface {
    /// Position in chunk grid
    /// TODO: Make async
    fn load_chunk(&self, position: UVec3, lod_level: usize) -> Box<Chunk>;
    fn load_chunks(&self, blank_chunks: Box<Vec<Chunk>>) -> Box<Vec<Chunk>>;
    fn overlay_chunk(&self);
    fn overlay_chunks(&self);

    // Mesh creation

    fn compute_mesh_from_chunk<'a>(&self, chunk: &Chunk) -> Mesh;
}
