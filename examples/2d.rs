//! This example creates a star overlapping with a circle at the mouse cursor.

use bevy::{prelude::*, render::render_asset::RenderAssetUsages};
use bevy_procedural_meshes::*;

#[derive(Resource)]
struct MeshHandleRes(Option<Handle<Mesh>>);
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MeshHandleRes(None))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mesh_handle_res: ResMut<MeshHandleRes>,
) {
    let mesh = meshes.add(PMesh::<u16>::default().to_bevy(RenderAssetUsages::all()));

    mesh_handle_res.0 = Some(mesh.clone());

    commands.spawn(Camera2d::default());
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(Color::WHITE)),
    ));
}

fn update(
    mesh_handle_res: ResMut<MeshHandleRes>,
    mut assets: ResMut<Assets<Mesh>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let inner_radius = 100.0;
    let outer_radius = 200.0;
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
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        {
            builder.add_circle(
                Vec2::new(world_position.x, world_position.y),
                100.0,
                Winding::Positive,
            );
        }
    });
    mesh.bevy_set(
        assets
            .get_mut(mesh_handle_res.0.clone().unwrap().id())
            .unwrap(),
    );
}
