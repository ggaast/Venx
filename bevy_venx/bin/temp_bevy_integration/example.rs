use std::time::{Duration, Instant};

use bevy::render::render_resource::PrimitiveTopology;
use bevy::{log, prelude::*};
use glam::{uvec3, vec4};
use main::{plat::VenxPlat, Venx};
use venx::plat::Plat;
use venx::voxel::cpu::topology::graph::Graph;
use venx::voxel::cpu::traverse::TrProps;
use venx::voxel::cpu::voxel::Voxel;
use venx::voxel::interfaces::layer::LayerInterface;
use venx::voxel::interfaces::load::LoadInterface;
use venx::voxel::interfaces::voxel::VoxelInterface;
use venx::voxel::segment::{Segment, SegmentStatic};

mod main;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Venx))
        .add_systems(Startup, setup)
        .run();
}
fn setup(
    mut cmd: Commands,
    mut q: Query<&mut VenxPlat>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // let mut vx = Voxel::new(10, 4, 7);

    // vx.topology.set(uvec3(0, 1, 0), true);
    // vx.topology.set(uvec3(0, 2, 0), true);
    // vx.topology.set(uvec3(1, 3, 0), true);
    // // second chunk
    info!("Starting the program");
    // vx.topology.set(uvec3(0, 8, 0), true);
    info!("Converting minecraft mca map into plat");

    let mut plat = Plat::load_mca("./assets/mca/9/", (0..3, 0..3)).unwrap();

    //let mut plat = Plat::new(6, 3, 3);
    // // let mut plat = Plat::new(3, 2, 2);

    // for _ in 0..100 {
    //     let start = Instant::now();
    //     for x in 0..200 {
    //         for z in 0..200 {
    //             v.get_voxel((x, 33, z).into());
    //         }
    //     }
    //     dbg!(start.elapsed());
    // }

    // plat.controller.get_voxel().

    // panic!();
    // plat.controller
    //     .get_voxel_mut()
    //     .set_voxel(0, (1, 1, 1).into(), 1);

    // plat.controller
    //     .get_voxel_mut()
    //     .set_voxel(0, (1, 2, 1).into(), 2);

    // let chunk = plat.controller.get_voxel().load_chunk((0, 0, 0).into());

    // let

    let voxel = plat.controller.get_voxel_mut();
    use downcast_rs::Downcast;

    let v: &mut Voxel = voxel.downcast_mut().unwrap();
    // for layer in &v.layers {
    //     for (key, slice) in &layer.slices {
    //         for level in &slice.graph.levels {
    //             dbg!(level.nodes.len());
    //         }
    //     }

    // }

    // v.set_voxel(0, (9, 2, 1).into(), 4);
    // v.set_voxel(0, (9, 3, 1).into(), 3);
    // v.set_voxel(0, (9, 4, 1).into(), 2);
    // v.set_voxel(0, (10, 4, 1).into(), 5);
    // v.set_voxel(0, (11, 4, 1).into(), 6);
    // v.set_voxel(0, (8, 4, 1).into(), 6);

    // // ^^^ Same 1 ^^^

    // v.set_voxel(0, (1, 2, 1).into(), 4);
    // v.set_voxel(0, (1, 3, 1).into(), 3);
    // v.set_voxel(0, (1, 4, 1).into(), 2);
    // v.set_voxel(0, (2, 4, 1).into(), 5);
    // v.set_voxel(0, (3, 4, 1).into(), 6);
    // v.set_voxel(0, (0, 4, 1).into(), 6);

    // // ^^^ Same 2 ^^^

    // v.set_voxel(0, (0, 6, 1).into(), 6);
    // v.set_voxel(0, (0, 7, 2).into(), 6);

    // v.set_voxel(0, (2, 6, 1).into(), 6);
    // v.set_voxel(0, (2, 7, 2).into(), 6);

    // v.set_voxel(0, (1, 1, 1).into(), 1);

    let mut counter = 0;

    // Graph::traverse_from_unpositioned(&v.layers[0].graph.levels, )

    let graph = &mut v.layers[0].graph;

    // panic!();

    // for entry in 1..(graph.entries()) {
    //     Graph::traverse_from_unpositioned(&graph.levels, entry, graph.depth(), |p| {
    //         if let TrProps::Branch {
    //             children, level, ..
    //         } = p
    //         {
    //             if level == 1 {
    //                 dbg!("Hey", entry);
    //                 return false;
    //             } else {
    //                 return true;
    //             }
    //         }
    //         true
    //     });
    // }

    let mut node_counter = 0;
    let mut empty_counter = 0;

    for node in &v.layers[0].graph.levels[2].nodes {
        if node.ident == -1 {
            empty_counter += 1;
        } else {
            node_counter += 1;
        }
    }
    dbg!(node_counter, empty_counter);
    //panic!();
    let start = Instant::now();
    // for node in &mut v.layers[0].graph.levels[1].nodes {
    //     counter += node.children[0];
    // }
    // dbg!(start.elapsed());
    // for node in &mut v.layers[0].graph.levels[1].nodes {
    //     for child in &mut node.children {
    //         if *child != 0 {
    //             counter += 1;
    //         }
    //     }
    // }
    // dbg!(start.elapsed());

    // dbg!(counter);

    //
    // let start = Instant::now();

    // dbg!(start.elapsed());

    // v.layers[0].merge();
    // dbg!(&v);

    // panic!();

    // dbg!(chunk.get((1, 1, 1)));

    // panic!();

    // let mut segment = Segment::new(5);
    // let red = 3;
    // let white = 1;
    // let green = 2;
    // let blue = 4;

    // segment.set((1, 0, 1), blue);
    // segment.set((1, 2, 1), white);
    // segment.set((1, 1, 1), red);
    // segment.set((0, 1, 0), green);
    // segment.set((0, 3, 0), blue);
    // segment.set((0, 0, 0), blue);
    // segment.set((0, 0, 1), blue);
    // segment.set((0, 0, 2), green);

    // plat.insert_segment(segment, (0, 0, 0).into());

    // dbg!(&plat.controller.get_voxel());

    let mut bevy_mesh: Vec<Vec3> = vec![];
    let mut bevy_color: Vec<Vec4> = vec![];

    // let mut final_chunk = None;
    log::info!("Loading chunks and computing mesh");

    for x in 0..(32 * 9) {
        for z in 0..(32 * 9) {
            for y in (7..15).rev() {
                let chunk = v.load_chunk(uvec3(x, y, z));
                let vx_mesh = v.compute_mesh_from_chunk(&chunk);
                // dbg!("Check");
                // chunk.iter(|p, t| {
                //     if t != 0 {
                //         // dbg!(p, t);
                //     }
                // });
                // panic!();
                for (pos, color) in vx_mesh {
                    let new_pos: bevy::prelude::Vec3 =
                        bevy::prelude::Vec3::from_array(pos.to_array());
                    let new_color: bevy::prelude::Vec4 =
                        bevy::prelude::Vec4::from_array(color.to_array());
                    bevy_mesh.push(new_pos);
                    bevy_color.push(new_color);
                }
                // continue;
            }
        }
    }

    log::info!("finish loading and computing mesh");

    let len = bevy_mesh.len();

    // Positions of the vertices
    // See https://bevy-cheatbook.github.io/features/coords.html
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, bevy_mesh);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, bevy_color);

    // In this example, normals and UVs don't matter,
    // so we just use the same value for all of them
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; len]);
    //mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; len]);
    // let plat = todo!();
    // cmd.spawn((VenxPlat(todo!())));
    // for plat in &mut q {
    //     let mut mtx = [[[0; 64]; 64]; 64];

    //     mtx[0][0][0] = 1;
    //     mtx[0][1][0] = 1;
    //     mtx[0][2][0] = 1;
    //     mtx[0][3][0] = 1;

    //     // let segment = SegmentStatic { mtx };

    //     // plat.0.insert_segment(segment, uvec3(0, 0, 0));

    //     // let chunk = plat.0.load_chunk(uvec3(0, 0, 0), 0);

    //     // let mesh = plat.0.compute_mesh();
    // }

    cmd.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(1., 1., 1.),
            // alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        ..default()
    });
}
