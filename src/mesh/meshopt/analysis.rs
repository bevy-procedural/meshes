use super::super::PMesh;
use super::util::get_adapter;
use crate::IndexType;
use bevy::{ecs::system::Resource, prelude::*};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use meshopt::analyze::analyze_vertex_cache;
use meshopt::{analyze_overdraw, analyze_vertex_fetch};

/// Results when analyzing the mesh efficiency using meshopt
#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct MeshoptAnalysis {
    /// The number of vertices in the mesh
    pub vertex_count: usize,

    /// The number of indices in the mesh
    pub index_count: usize,

    // overdraw
    /// The number of pixels covered by the mesh
    pub pixels_covered: u32,
    /// The number of pixels shaded by the mesh
    pub pixels_shaded: u32,
    /// The portion of pixels that are overdrawn
    pub overdraw: f32,

    // vertex cache
    /// The number of vertices transformed
    pub vertices_transformed: u32,
    /// The number of warps executed
    pub warps_executed: u32,
    /// The average cache miss ratio
    pub acmr: f32,
    /// The average transformed vertex ratio
    pub atvr: f32,

    // vertex fetch
    /// The number of bytes fetched from the vertex buffer
    pub bytes_fetched: u32,
    /// The portion of bytes that are overfetched
    pub overfetch: f32,
}

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Analyzes the mesh using meshopt and stores the results in a `MeshoptAnalysis` resource
    pub fn meshopt_analyse(&self) -> MeshoptAnalysis {
        let mesh = self.to_meshopt_data();

        // estimate based on a RTX 4060
        let warp_size = 32; // NVIDIA's warp size has been consistently 32 threads across several generations of GPUs.
        let prim_group_size = 1024; // can vary by architecture but is often 1024 threads per block for recent architectures.
        let cache_size = 1024 * 32; // not sure, I'm guessing around
        let vertex_cache = analyze_vertex_cache(
            &mesh.indices,
            mesh.vertices.len(),
            cache_size,
            warp_size,
            prim_group_size,
        );

        // I'm not sure overdraw is correctly analyzed... The values don't seem plausible.
        let overdraw = analyze_overdraw(&mesh.indices, &get_adapter(&mesh.vertices));

        let vertex_size = std::mem::size_of::<f32>() * (3 + 3 + 2 + 4); // position, normal, uv, tangent
        let fetch = analyze_vertex_fetch(&mesh.indices, mesh.vertices.len(), vertex_size);

        return MeshoptAnalysis {
            vertex_count: mesh.vertices.len(),
            index_count: mesh.indices.len(),
            pixels_covered: overdraw.pixels_covered,
            pixels_shaded: overdraw.pixels_shaded,
            overdraw: overdraw.overdraw,
            vertices_transformed: vertex_cache.vertices_transformed,
            warps_executed: vertex_cache.warps_executed,
            acmr: vertex_cache.acmr,
            atvr: vertex_cache.atvr,
            bytes_fetched: fetch.bytes_fetched,
            overfetch: fetch.overfetch,
        };
    }
}
