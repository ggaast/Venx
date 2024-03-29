use glam::UVec3;
use venx_core::{plat::op::get::GetNodeResult, utils::Grid};

use crate::plat::interfaces::layer::LayerInterface;

use super::gpu_plat::GpuPlat;

impl LayerInterface for GpuPlat {
    fn set_segment<const SIZE: usize>(
        &mut self,
        layer: usize,
        segment: Grid<SIZE>,
        position: glam::UVec3,
    ) {
        todo!()
    }

    fn set_voxel(&mut self, layer: usize, position: glam::UVec3, ty: usize) {
        todo!()
    }

    fn get_voxel(&self, position: glam::UVec3) -> Option<GetNodeResult> {
        todo!()
    }

    fn compress(
        &mut self,
        layer: usize,
        position: UVec3,
        level: u32,
        lookup_tables: &mut Vec<std::collections::HashMap<venx_core::plat::node::Node, usize>>,
    ) {
        todo!()
    }
}
