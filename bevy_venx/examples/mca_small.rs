use std::f32::consts::PI;

use bevy::{
    math::vec3,
    pbr::{
        CascadeShadowConfigBuilder, DirectionalLightShadowMap, NotShadowCaster,
        ScreenSpaceAmbientOcclusionBundle,
    },
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use bevy_panorbit_camera::PanOrbitCamera;
use venx::plat::VenxPlat;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_panorbit_camera::PanOrbitCameraPlugin))
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(Color::rgb(0.52, 0.80, 0.92)))
        .insert_resource(DirectionalLightShadowMap { size: 512 })
        .run();
}
fn setup(
    mut cmd: Commands,
    mut bevy_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Its small-sized plat, its slow to convert it from mca each run, it will be saved
    let plat = VenxPlat::load("mca_small").unwrap_or_else(|e| {
        warn!("Plat wasnt found on device, creating new and saving ({e})");
        // Convert from minecraft map
        let plat = VenxPlat::load_mca("./assets/mca/1/", (0..1, 0..1)).unwrap();
        plat.save("mca_small").unwrap();
        plat
    });
    for mesh in plat.static_mesh(0..16, 3..6, 0..16, Some(1)) {
        let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.0.clone());
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh.1.clone());
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.2.clone());

        cmd.spawn(PbrBundle {
            mesh: bevy_meshes.add(bevy_mesh),
            material: materials.add(StandardMaterial {
                reflectance: 0.1,
                base_color: Color::rgb(1., 1., 1.),
                // alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            ..default()
        })
        //.insert(Wireframe)
        ;
    }

    // ambient light
    cmd.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.15,
    });
    // // light
    // cmd.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 5000000.0,
    //         shadows_enabled: true,
    //         range: 4000.,
    //         radius: 2000.,
    //         color: Color::YELLOW,
    //         ..Default::default()
    //     },
    //     transform: Transform {
    //         translation: Vec3::new(-10.0, 500.0, -10.0),
    //         rotation: Quat::from_rotation_x(-PI / 3.87),
    //         ..default()
    //     },
    //     ..default()
    // });

    // // // light
    // cmd.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 5000000.0,
    //         shadows_enabled: true,
    //         range: 4000.,
    //         radius: 2000.,
    //         color: Color::WHITE,
    //         ..default()
    //     },
    //     transform: Transform {
    //         translation: Vec3::new(300.0, 500.0, -10.0),
    //         rotation: Quat::from_rotation_x(-PI / 3.87),
    //         ..default()
    //     },
    //     ..default()
    // });
    // directional 'sun' light
    // cmd.spawn(DirectionalLightBundle {
    //     directional_light: DirectionalLight {
    //         shadows_enabled: true,
    //         illuminance: 35_000.,
    //         ..Default::default()
    //     },
    //     transform: Transform {
    //         translation: Vec3::new(1000.0, -1000.0, 1000.0),
    //         // rotation: Quat::from_rotation_x(-PI / 3.87),
    //         ..default()
    //     },
    //     // The default cascade config is designed to handle large scenes.
    //     // As this example has a much smaller world, we can tighten the shadow
    //     // bounds for better visual quality.
    //     cascade_shadow_config: CascadeShadowConfigBuilder {
    //         first_cascade_far_bound: 200.0,
    //         maximum_distance: 5000.0,
    //         ..default()
    //     }
    //     .into(),
    //     ..default()
    // });
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Sun
    cmd.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(300.0, 300.0, 300.0)
            .looking_at(Vec3::new(-0.15, -0.05, -0.25), Vec3::Y),
        cascade_shadow_config,
        ..default()
    });
    // Sky
    cmd.spawn((
        PbrBundle {
            mesh: bevy_meshes.add(Mesh::from(shape::Box::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("888888").unwrap(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(1900.0)),
            ..default()
        },
        NotShadowCaster,
    ));

    // camera
    cmd.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(28.0, 200., 28.0)
                .looking_at(Vec3::new(-0.15, -0.05, -0.25), Vec3::Y),
            ..default()
        },
        // ScreenSpaceAmbientOcclusionBundle::default(),
        // TemporalAntiAliasBundle::default(),
        // FogSettings {
        //     color: Color::rgb(0.52, 0.80, 0.92),
        //     // falloff: FogFalloff::Atmospheric { extinction: (), inscattering: () } {
        //     //     start: 200.0,
        //     //     end: 5000.0,
        //     // },
        //     falloff: FogFalloff::from_visibility(3050.0),
        //     ..Default::default()
        // },
        FogSettings {
            color: Color::rgba(0.35, 0.48, 0.66, 1.0),
            directional_light_color: Color::rgba(1.0, 0.95, 0.85, 0.5),
            directional_light_exponent: 30.0,
            falloff: FogFalloff::from_visibility_colors(
                300.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
            ),
        },
        PanOrbitCamera {
            // Set focal point (what the camera should look at)
            focus: Vec3::new(280.0, 228., 280.0),
            ..Default::default()
        },
    ));
}
