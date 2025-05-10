# Bevy Procedural: Meshes

[![Documentation](https://docs.rs/bevy_procedural_meshes/badge.svg)](https://docs.rs/bevy_procedural_meshes)
[![Bevy](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)
[![crates.io](https://img.shields.io/crates/v/bevy_procedural_meshes)](https://crates.io/crates/bevy_procedural_meshes)
[![License](https://img.shields.io/crates/l/bevy_procedural_meshes)](https://bevyengine.org/learn/quick-start/plugin-development/#licensing)


The objective of the [Bevy Procedural Project](https://bevy-procedural.org) is to provide a comprehensive suite of packages for the generation of procedural graphics, unified by a consistent API.

The [bevy_procedural_meshes](https://bevy-procedural.org/meshes)-crate is a procedural mesh builder for bevy. It can use [Lyon](https://github.com/nical/lyon) to generate 2D shapes and extrude them into 3D meshes. Meshes can also be optimized using [Meshopt](https://github.com/gwihlidal/meshopt-rs) to improve their performance. Plans for future versions also include more advanced mesh operations like 3D boolean operations, parametric surfaces, and L-systems.


## WARNING

[![Build Status](https://github.com/bevy-procedural/meshes/actions/workflows/rust.yml/badge.svg)](https://github.com/bevy-procedural/meshes/actions)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.com/channels/691052431525675048/1034543904478998539)
[![Downloads](https://img.shields.io/crates/d/bevy_procedural_meshes)](https://crates.io/crates/bevy_procedural_meshes)
[![GitHub Repo stars](https://img.shields.io/github/stars/bevy-procedural/meshes)](https://github.com/bevy-procedural/meshes)

This crate is still in a _very_ early stage of development. Expect frequent API modifications, bugs, and missing features. Feel free to contribute by opening issues, pull requests or sharing your ideas in [Github Discussion](https://github.com/bevy-procedural/meshes/discussions) or the [Bevy Discord](https://discord.gg/bevy).


## Examples 

Try the live examples!
 * [2d](https://bevy-procedural.org/examples/meshes/2d)
 * [3d](https://bevy-procedural.org/examples/meshes/3d)

Or run the [examples](https://github.com/bevy-procedural/meshes/tree/main/examples) on your computer like, e.g., `cargo run --features="bevy/default" --example 2d`.

For package development, we recommend using the `editor`-subcrate. This example has a little [egui](https://github.com/jakobhellermann/bevy-inspector-egui/)-editor. Run it using `cargo watch -w editor/src -w src -x "run -p editor --profile fast-dev"`. The `fast-dev` profile will enable optimizations for the dependencies, but not for the package itself. This will slow down the first build _significantly_, but incremental builds are slightly faster and bevy's performance improves a lot.


## Usage

Install using `cargo add bevy_procedural_meshes`. Create meshes for bevy like:

```rs
use bevy_procedural_meshes::*;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = PMesh::<u32>::new();
    mesh.fill(0.01, |builder| {
        builder
            .begin_here()
            .quadratic_bezier_to(Vec2::new(3.0, 3.0), Vec2::new(1.5, 3.0))
            .quadratic_bezier_to(Vec2::new(0.0, 3.0), Vec2::new(0.0, 0.0))
            .close();
    });
    
    commands.spawn((
        Mesh3d(meshes.add(mesh.to_bevy(RenderAssetUsages::all()))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
    ));
}
```


## Features

The following features are available:

* `meshopt` -- Use [Meshopt](https://github.com/gwihlidal/meshopt-rs) to optimize the performance of generated meshes. 
* `lyon` -- Use [Lyon](https://github.com/nical/lyon) to tesselate 2D shapes like bezier curves and strokes.
* `inspector` -- Add [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui)-support to different structs.
* `dynamic` -- Compiles bevy as a dynamic library. Useful for development builds. 


## Supported Bevy Versions

The following table shows the compatibility of `bevy_procedural_meshes` with certain versions of Bevy:

| bevy | bevy_procedural_meshes |
| ---- | ---------------------- |
| 0.15 | 0.15.*, main           |
| 0.14 | 0.14.*                 |
| 0.13 | 0.1.*                  |


## License

The bevy-procedural packages are free, open source and permissively licensed. Except where noted (below and/or in individual files), all code in these repositories is dual-licensed, allowing you the flexibility to choose between:

 - The MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
 - The Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0).


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
