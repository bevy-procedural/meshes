use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    pbr::CascadeShadowConfigBuilder,
    prelude::*, window::WindowResolution,
};
use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use procedural_meshes::fill::MyFill;
use std::{env, f32::consts::PI};

pub fn main() {
    env::set_var("RUST_BACKTRACE", "1"); // or "full"

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1920.0, 1080.0),
                position: WindowPosition::Centered(MonitorSelection::Index(1)),
                decorations: false,
                ..default()
            }),
            ..default()
        }))
        .register_type::<MeshSettings>()
        .add_plugins((
            EditorPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            SystemInformationDiagnosticsPlugin::default(),
        ))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup_meshes)
        .add_systems(Update, update_meshes)
        .run();
}

#[derive(Reflect, Component, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct MeshSettings {
    #[inspector(min = 0.1, max = 10.0)]
    extrude: f32,
    #[inspector(min = -20.0, max = 1.0)]
    tol: f32,
    vertices: u32,
}

impl Default for MeshSettings {
    fn default() -> Self {
        MeshSettings {
            extrude: 5.0,
            tol: -5.0,
            vertices: 0,
        }
    }
}

pub fn update_meshes(
    mut query: Query<(&Handle<Mesh>, &mut MeshSettings), Changed<MeshSettings>>,
    mut assets: ResMut<Assets<Mesh>>,
) {
    for (handle, mut settings) in query.iter_mut() {
        if let Some(mesh) = assets.get_mut(handle.id()) {
            let mut fill = MyFill::new(2.0f32.powf(settings.tol));
            fill.draw(|builder| {
                builder.begin(Vec2::new(3.0, 0.0));
                builder.quadratic_bezier_to(Vec2::new(3.0, 3.0), Vec2::new(1.5, 3.0));
                builder.quadratic_bezier_to(Vec2::new(0.0, 3.0), Vec2::new(0.0, 0.0));
                builder.end(true);
            });
            let my_mesh = fill
                .build(false)
                .vertices
                .sort_clockwise()
                .extrude(Vec3::new(0.0, 0.0, -settings.extrude));
            settings.vertices = my_mesh.vertices.len() as u32;
            my_mesh.bevy_set(mesh);
        }
    }
}

pub fn setup_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.5, 0.5),
                // base_color_texture: Some(tex),
                double_sided: true,
                cull_mode: None,
                alpha_mode: AlphaMode::Mask(0.5),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.1, 0.0).with_scale(Vec3::new(0.3, 0.3, 0.3)),
            ..default()
        },
        MeshSettings::default(),
        Name::new("Generated Shape"),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..Default::default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2.0, 3.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
