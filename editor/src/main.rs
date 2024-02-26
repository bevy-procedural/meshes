use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::{
        wireframe::{WireframeConfig, WireframePlugin},
        CascadeShadowConfigBuilder,
    },
    prelude::*,
    render::render_asset::RenderAssetUsages,
    window::WindowResolution,
};
use bevy_inspector_egui::{
    inspector_options::ReflectInspectorOptions,
    quick::{FilterQueryInspectorPlugin, ResourceInspectorPlugin, WorldInspectorPlugin},
    InspectorOptions,
};
use bevy_panorbit_camera::*;
use bevy_procedural_meshes::mesh::{
    lyon::Winding,
    meshopt::{MeshoptAnalysis, MeshoptSettings},
    PMesh,
};
use std::{env, f32::consts::PI};

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct GlobalSettings {
    #[inspector(min = 0.1, max = 10.0)]
    extrusion: f32,

    #[inspector(min = -20.0, max = 10.0)]
    tol: f32,
    #[inspector(min = 0.0, max = 3.0)]
    inner_radius: f32,
    #[inspector(min = 0.0, max = 3.0)]
    outer_radius: f32,
    #[inspector(min = 0.0, max = 3.0)]
    circle_radius: f32,
    #[inspector(min = 2, max = 50)]
    points: u32,

    meshopt: bool,
    settings: MeshoptSettings,
    analysis: MeshoptAnalysis,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        GlobalSettings {
            extrusion: 5.0,
            tol: -4.0,
            inner_radius: 1.0,
            outer_radius: 2.0,
            circle_radius: 1.0,
            points: 5,

            meshopt: false,
            settings: MeshoptSettings::default(),
            analysis: MeshoptAnalysis::default(),
        }
    }
}

#[derive(Reflect, Component, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct MeshSettings {
    #[inspector(min = 0.1, max = 10.0)]
    extrude: f32,
    #[inspector(min = -20.0, max = 10.0)]
    tol: f32,
    vertices: u32,
}

impl Default for MeshSettings {
    fn default() -> Self {
        MeshSettings {
            extrude: 5.0,
            tol: -10.0,
            vertices: 0,
        }
    }
}

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
        .add_plugins(WireframePlugin)
        .insert_resource(WireframeConfig {
            global: true,
            default_color: Color::WHITE,
        })
        .register_type::<GlobalSettings>()
        .insert_resource(GlobalSettings::default())
        .register_type::<MeshSettings>()
        .add_plugins((
            FilterQueryInspectorPlugin::<With<MeshSettings>>::default(),
            ResourceInspectorPlugin::<GlobalSettings>::default(),
            WorldInspectorPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            PanOrbitCameraPlugin,
        ))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup_meshes)
        .add_systems(Update, update_meshes)
        .run();
}

fn update_meshes(
    query: Query<&Handle<Mesh>>,
    mut assets: ResMut<Assets<Mesh>>,
    mut settings: ResMut<GlobalSettings>,
) {
    let angle = std::f32::consts::PI / settings.points as f32;

    let mut mesh = PMesh::<u16>::hexagon(1.0);
    mesh.fill(0.01, |builder| {
        builder.push().begin(Vec2::new(settings.inner_radius, 0.0));
        for _ in 0..settings.points {
            builder
                .rotate(angle)
                .line_to(Vec2::new(settings.outer_radius, 0.0))
                .rotate(angle)
                .line_to(Vec2::new(settings.inner_radius, 0.0));
        }
        builder.close_pop();

        builder.add_circle(
            Vec2::new(0.5, 0.5),
            settings.circle_radius,
            Winding::Positive,
        );
    });

    if settings.meshopt {
        mesh.mesh_opt(&settings.settings);
    }

    settings.analysis = mesh.meshopt_analyse();

    mesh.flip_yz().bevy_set(assets.get_mut(query.single().id()).unwrap());
}

fn setup_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(PMesh::<u16>::new().to_bevy(RenderAssetUsages::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.5, 0.5),
                double_sided: false,
                cull_mode: None,
                ..default()
            }),
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

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 0.1).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}
