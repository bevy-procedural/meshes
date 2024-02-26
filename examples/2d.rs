//! This example creates a star overlapping with a circle at the mouse cursor.

use bevy::{
    prelude::*,
    render::render_asset::RenderAssetUsages,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_procedural_meshes::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(PMesh::<u16>::default().to_bevy(RenderAssetUsages::all()))
            .into(),
        material: materials.add(Color::PURPLE),
        ..default()
    });
}

fn update(
    query: Query<&Mesh2dHandle>,
    mut assets: ResMut<Assets<Mesh>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let inner_radius = 100.0;
    let outer_radius = 200.0;
    let points = 5;
    let angle = std::f32::consts::PI / points as f32;

    let mut fill = PFill::new(0.1);
    fill.draw(|builder| {
        builder.push().begin(Vec2::new(inner_radius, 0.0));
        for _ in 0..points {
            builder
                .rotate(angle)
                .line_to(Vec2::new(outer_radius, 0.0))
                .rotate(angle)
                .line_to(Vec2::new(inner_radius, 0.0));
        }
        builder.close_pop();

        let window = windows.single();
        let (camera, camera_transform) = camera_q.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        {
            builder.add_circle(
                Vec2::new(-world_position.x, world_position.y),
                100.0,
                Winding::Positive,
            );
        }
    });
    fill.build::<u16>(false)
        .bevy_set(assets.get_mut(query.single().0.id()).unwrap());
}
