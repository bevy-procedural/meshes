//! This example creates a star overlapping with a circle at the mouse cursor.

use std::f32::consts::PI;

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::render_asset::RenderAssetUsages,
};
use bevy_procedural_meshes::*;

#[derive(Resource)]
struct MeshHandleRes(Option<Handle<Mesh>>);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WireframePlugin::default()))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE,
        })
        .insert_resource(MeshHandleRes(None))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_handle_res: ResMut<MeshHandleRes>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
    ));

    let mesh = meshes.add(PMesh::<u16>::default().to_bevy(RenderAssetUsages::all()));
    mesh_handle_res.0 = Some(mesh.clone());

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        })),
    ));
}

fn update(
    mesh_handle_res: ResMut<MeshHandleRes>,
    mut assets: ResMut<Assets<Mesh>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let inner_radius = 1.0;
    let outer_radius = 2.0;
    let points = 5;
    let angle = std::f32::consts::PI / points as f32;

    let mut mesh = PMesh::<u32>::new();
    mesh.fill(0.1, |builder| {
        builder.push().begin(Vec2::new(inner_radius, 0.0));
        for _ in 0..points {
            builder
                .rotate(angle)
                .line_to(Vec2::new(outer_radius, 0.0))
                .rotate(angle)
                .line_to(Vec2::new(inner_radius, 0.0));
        }
        builder.close_pop();

        let window = windows.single().unwrap();
        let (camera, camera_transform) = camera_q.single().unwrap();
        if let Some(ray) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        {
            let distance = ray
                .intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
                .unwrap_or(0.0);
            let world_position = ray.get_point(distance);
            builder.add_circle(
                Vec2::new(world_position.x, world_position.z),
                1.0,
                Winding::Positive,
            );
        }
    });

    // TODO: Extrude the whole shape! However, sorting the vertices is not sufficient for complex shapes.
    // fill.build::<u16>(false).get_vertices_mut().sort_clockwise().extrude(Vec3::Z);

    mesh.flip_yz().bevy_set(
        assets
            .get_mut(mesh_handle_res.0.clone().unwrap().id())
            .unwrap(),
    );
}
