use std::intrinsics::size_of;

use bytemuck::{cast, cast_ref, cast_slice};
use easy_compute::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupBuilder, BufferDescriptor, BufferRW, BufferUsages, ComputePassDescriptor,
    PipelineBuilder,
};
use glam::uvec3;
use log::info;
use pollster::block_on;
use venx_core::{
    plat::chunk::chunk::{Chunk, ChunkLoadRequest},
    utils::Grid,
};

use crate::plat::{
    interfaces::load::LoadInterface,
    normal::mesh::{Mesh, CHUNK_BUCKET, MESH_SIZE},
};

use super::gpu_plat::GpuPlat;

impl LoadInterface for GpuPlat {
    fn load_chunk(&self, position: glam::UVec3, lod_level: usize) -> Box<Chunk> {
        block_on(async {
            // TODO: Make use of push constants for position and lod_level
            let chunk = Chunk::new(position.to_array(), lod_level, 5);
            // let (flatten, chunk_meta) = .to_send();
            let chunk_buffer = self.cs.new_buffer(bytemuck::cast_slice(&[chunk]));
            // let chunk_flatten_buffer = self.cs.new_buffer(bytemuck::cast_slice(&flatten));

            let chunk_bg = BindGroupBuilder::new()
                .insert(0, false, chunk_buffer.as_entire_binding())
                .build(&self.cs);

            let output_buffer = self.cs.new_staging_buffer(chunk_buffer.size(), true);

            // Load pipelines
            let load_chunk_pl = PipelineBuilder::new(&self.module, "load_chunk_2")
                .for_bindgroup(&self.base_bg)
                .for_bindgroup(&self.tmp_bg)
                .for_bindgroup(&self.schem_bg)
                .for_bindgroup(&self.canvas_bg)
                .for_bindgroup(&self.raw_plat_bg)
                .for_bindgroup(&chunk_bg)
                .build(&self.cs);

            self.cs
                .eval(|encoder| {
                    {
                        let mut cpass =
                            encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
                        cpass.set_pipeline(&load_chunk_pl);

                        cpass.set_bind_group(0, &self.base_bg.bindgroup, &[]);
                        cpass.set_bind_group(1, &self.tmp_bg.bindgroup, &[]);
                        cpass.set_bind_group(2, &self.schem_bg.bindgroup, &[]);
                        cpass.set_bind_group(3, &self.canvas_bg.bindgroup, &[]);
                        cpass.set_bind_group(4, &self.raw_plat_bg.bindgroup, &[]);
                        cpass.set_bind_group(5, &chunk_bg.bindgroup, &[]);

                        cpass.dispatch_workgroups(1, 1, 1);
                    }
                    //
                    encoder.copy_buffer_to_buffer(
                        &chunk_buffer,
                        0,
                        &output_buffer,
                        0,
                        output_buffer.size(),
                    );
                })
                .await;
            let output: Vec<Chunk> = output_buffer.read_manual().await;

            output_buffer.unmap();

            Box::new(output[0])
        })
    }

    fn overlay_chunk(&self) {
        todo!()
    }

    fn overlay_chunks(&self) {
        todo!()
    }

    fn compute_mesh_from_chunk<'a>(&self, _chunk: &Chunk) -> crate::plat::normal::mesh::Mesh {
        block_on(async {
            info!("Prepering buffers and pipeline");

            let output_buffer = self.cs.new_staging_buffer(self.mesh.size(), true);

            self.cs
                .eval(|encoder| {
                    {
                        let mut cpass =
                            encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
                        cpass.set_pipeline(&self.to_mesh_greedy_pl);

                        cpass.set_bind_group(0, &self.base_bg.bindgroup, &[]);
                        cpass.set_bind_group(1, &self.tmp_bg.bindgroup, &[]);
                        cpass.set_bind_group(2, &self.schem_bg.bindgroup, &[]);
                        cpass.set_bind_group(3, &self.canvas_bg.bindgroup, &[]);
                        cpass.set_bind_group(4, &self.raw_plat_bg.bindgroup, &[]);
                        cpass.set_bind_group(5, &self.chunk_bg.bindgroup, &[]);
                        cpass.set_bind_group(6, &self.mesh_helpers_bg.bindgroup, &[]);
                        // Hardcoded!
                        cpass.dispatch_workgroups(CHUNK_BUCKET as u32, 1, 1);
                    }
                    //
                    encoder.copy_buffer_to_buffer(
                        &self.mesh,
                        0,
                        &output_buffer,
                        0,
                        output_buffer.size(),
                    );
                })
                .await;
            info!("Queue submited");
            let output: Vec<[f32; 10]> = output_buffer.read_manual().await;

            output_buffer.unmap();
            info!("Chunks are copied");
            Box::new(output)
            // Box::new(vec![])
        })
    }

    fn load_chunks(&self, blank_chunks: Box<Vec<venx_core::plat::chunk::chunk::ChunkLoadRequest>>) {
        block_on(async {
            // info!("Prepering buffers and pipeline");
            // let (flatten, chunk_meta) = .to_send();

            //let new_buffer = self.cs.new_buffer(bytemuck::cast_slice(&blank_chunks));

            // let st_buffer = self
            //     .cs
            //     .new_staging_buffer(self.chunks_requests_buffer.size(), false);

            let st_buffer = self.cs.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                // con: self.chunks_requests_buffer.size(), //size: 32768 * 4 * locs.len() as u64,
                usage: BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC,
                contents: bytemuck::cast_slice(&blank_chunks),
            });

            let buf = self.cs.device.create_buffer(&BufferDescriptor {
                label: None,
                // con: self.chunks_requests_buffer.size(), //size: 32768 * 4 * locs.len() as u64,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                size: st_buffer.size(),
                mapped_at_creation: false,
            });

            self.cs
                .eval(|encoder| {
                    encoder.copy_buffer_to_buffer(
                        &st_buffer,
                        0,
                        &self.chunks_requests_buffer,
                        0,
                        self.chunks_requests_buffer.size(),
                    );
                    {
                        let mut cpass =
                            encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
                        cpass.set_pipeline(&self.load_chunk_pl);

                        cpass.set_bind_group(0, &self.base_bg.bindgroup, &[]);
                        cpass.set_bind_group(1, &self.tmp_bg.bindgroup, &[]);
                        cpass.set_bind_group(2, &self.schem_bg.bindgroup, &[]);
                        cpass.set_bind_group(3, &self.canvas_bg.bindgroup, &[]);
                        cpass.set_bind_group(4, &self.raw_plat_bg.bindgroup, &[]);
                        cpass.set_bind_group(5, &self.chunk_bg.bindgroup, &[]);
                        cpass.dispatch_workgroups(blank_chunks.len() as u32, 1, 1);
                    }

                    encoder.copy_buffer_to_buffer(
                        &self.chunks_requests_buffer,
                        0,
                        &buf,
                        0,
                        self.chunks_requests_buffer.size(),
                    );
                })
                .await;
            buf.read(|_: Vec<ChunkLoadRequest>| {}).await;
            buf.destroy();
            st_buffer.destroy();

            // let _ = &self
            //     .chunks_requests_staging_buffer
            //     .write(|d: &mut [ChunkLoadRequest]| {
            //         dbg!(&d[0..10]);
            //     })
            //     .await;

            // panic!();
        });
    }
}
