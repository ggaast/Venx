#![no_std]
#![feature(generic_const_exprs)]

pub mod plat;
pub mod utils;
pub use spirv_std::glam;
pub mod mesh;

pub use plat::raw_plat::LayerIndex::*;
pub use utils::*;
