use super::super::PMesh;
use super::util::{get_adapter, MeshoptMesh};
use crate::IndexType;
use bevy::prelude::*;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};

/// Settings for the meshopt Optimization
#[derive(Reflect, Resource)]
#[reflect(Resource)]
#[cfg_attr(feature = "inspector", derive(InspectorOptions))]
#[cfg_attr(feature = "inspector", reflect(InspectorOptions))]
pub struct MeshoptSettings {
    /// Whether the mesh should be simplified
    pub simplify: bool,

    /// The maximum error allowed when simplifying the mesh
    pub simplify_target_error: f32,

    /// The target percentage of the original vertex count when simplifying the mesh
    pub simplify_target_percentage: f32,

    /// Whether to reorder indices to reduce the number of GPU vertex shader invocations
    pub optimize_vertex_cache: bool,

    /// Whether to reorder indices to reduce overdraw, balancing overdraw and vertex cache efficiency
    pub optimize_overdraw: bool,

    /// When using optimize_overdraw: Acceptance for worse ACMR to get more reordering opportunities for overdraw.
    /// For example, 1.05 means that the algorithm will accept a 5% worse ACMR if it can get a better overdraw.
    pub accept_worse_acmr: f32,

    /// Whether to reorder vertices and change indices to reduce the amount of GPU memory fetches during vertex processing.
    pub optimize_vertex_fetch: bool,
}

impl Default for MeshoptSettings {
    fn default() -> Self {
        MeshoptSettings {
            simplify: true,
            simplify_target_error: 0.001,
            simplify_target_percentage: 0.0,
            optimize_vertex_cache: true,
            optimize_overdraw: true,
            accept_worse_acmr: 1.05,
            optimize_vertex_fetch: true,
        }
    }
}

fn mesh_opt_complete(mesh: &mut MeshoptMesh, settings: &MeshoptSettings) {
    let vertex_adapter = get_adapter(&mesh.vertices);
    //let initial_size = mesh.vertices.len();
    //let initial_indices = mesh.indices.len();

    /*
    // Generates a vertex remap table from the vertex buffer and an optional index buffer and returns number of unique vertices.
    let (vertex_count, remap) = meshopt::generate_vertex_remap(vertices, Some(indices));

    // Apply the remap table to the vertex buffer and the index buffer.
    let new_indices = meshopt::remap_index_buffer( Some(indices.as_ref()) as Option<&[u32]>, vertex_count, &remap);
    indices.resize(new_indices.len(), 0);
    indices.copy_from_slice(&new_indices);
    */

    let mut result_error = 0.0f32;
    if settings.simplify {
        let target_count =
            (mesh.indices.len() as f32 * settings.simplify_target_percentage) as usize / 3 * 3;
        let new_indices = meshopt::simplify(
            &mesh.indices,
            &vertex_adapter,
            target_count,
            settings.simplify_target_error,
            meshopt::SimplifyOptions::None,
            Some(&mut result_error),
        );
        mesh.indices.resize(new_indices.len(), 0);
        mesh.indices.copy_from_slice(&new_indices);
    }

    if settings.optimize_vertex_cache {
        meshopt::optimize_vertex_cache_in_place(&mut mesh.indices, vertex_adapter.vertex_count);
    }

    if settings.optimize_overdraw {
        meshopt::optimize_overdraw_in_place(
            &mut mesh.indices,
            &vertex_adapter,
            settings.accept_worse_acmr,
        );
    }

    if settings.optimize_vertex_fetch {
        let final_size =
            meshopt::optimize_vertex_fetch_in_place(&mut mesh.indices, &mut mesh.vertices);
        mesh.vertices.resize(final_size, Default::default());
    }

    /*
    println!("Final size:    {}\t<- {}", final_size, initial_size);
    println!("Final indices: {}\t<- {}", indices.len(), initial_indices);
    println!("result_error:  {}", result_error);
    */
}

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Optimize the mesh using meshopt
    pub fn mesh_opt(&mut self, settings: &MeshoptSettings) -> &mut Self {
        let mut mesh = self.to_meshopt_data();
        mesh_opt_complete(&mut mesh, settings);
        self.import_meshopt_data(&mesh);
        self
    }
}
