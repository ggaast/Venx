use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use glam::{uvec3, vec4};
use main::{plat::VenxPlat, Venx};
use venx_core::plat::Plat;
use venx_core::voxel::cpu::topology::graph::Graph;
use venx_core::voxel::cpu::voxel::Voxel;
use venx_core::voxel::segment::SegmentStatic;
use venx_core::voxel::vx_trait::*;

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

    let mut vx = Voxel::new(10, 4, 7);

    vx.topology.set(uvec3(0, 1, 0), true);
    vx.topology.set(uvec3(0, 2, 0), true);
    vx.topology.set(uvec3(1, 3, 0), true);
    // second chunk
    vx.topology.set(uvec3(0, 8, 0), true);

    let chunk = vx.load_chunk(UVec3::ZERO, 0);
    let vx_mesh = vx.to_mesh(&chunk);

    let mut bevy_mesh = vec![];
    let mut bevy_color = vec![];

    for (pos, color) in vx_mesh {
        bevy_mesh.push(pos);
        bevy_color.push(color);
    }
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
        material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
        ..default()
    });
}
