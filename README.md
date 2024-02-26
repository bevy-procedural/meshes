# Bevy Procedural: Meshes


[![Build Status](https://github.com/bevy-procedural/meshes/actions/workflows/rust.yml/badge.svg)](https://github.com/bevy-procedural/meshes/actions)
[![Documentation](https://docs.rs/bevy_procedural_meshes/badge.svg)](https://docs.rs/bevy_procedural_meshes)
[![Bevy Support](https://img.shields.io/badge/Bevy%20tracking-1.3-lightblue)](https://bevyengine.org/learn/book/plugin-development/#main-branch-tracking)
[![crates.io](https://img.shields.io/crates/v/bevy_procedural_meshes)](https://crates.io/crates/bevy_procedural_meshes)
[![Downloads](https://img.shields.io/crates/d/bevy_procedural_meshes)](https://crates.io/crates/bevy_procedural_meshes)
[![GitHub Repo stars](https://img.shields.io/github/stars/bevy-procedural/meshes)](https://github.com/Nilirad/bevy-procedural/meshes)

The objective of the [Bevy Procedural Project](https://bevy-procedural.org) is to provide a comprehensive suite of packages for the generation of procedural graphics, unified by a consistent API.

The [bevy_procedural_meshes](https://bevy-procedural.org/meshes)-crate is a procedural mesh builder for bevy. It can use [Lyon](https://github.com/nical/lyon) to generate 2D shapes and extrude them into 3D meshes. It also supports (very!) simple methods for off-screen rendering and Constructive Solid Geometry.

## WARNING

This crate is still in a _very_ early stage of development. Expect frequent API modifications, bugs, and missing features. Feel free to contribute by opening issues, pull requests or sharing your ideas in [Github Discussion](https://github.com/bevy-procedural/meshes/discussions) or the [Bevy Discord](https://discord.gg/bevy).


## Examples 

Try the live examples!
 * [2d](https://bevy-procedural.org/examples/meshes/2d)
 * [3d](https://bevy-procedural.org/examples/meshes/3d)

Or run the [examples]() like, e.g., `cargo run --example 2d --features="bevy/bevy_sprite bevy/bevy_winit"`.

For package development, we recommend using the `editor`-subcrate. This example has a little [egui](https://github.com/jakobhellermann/bevy-inspector-egui/)-editor. Run it using `cargo watch -w editor/src -w src -x "run -p editor --profile fast-dev"`. The `fast-dev` profile will enable optimizations for the dependencies, but not for the package itself. This will slow down the first build _significantly_, but incremental builds are slightly faster and bevy's performance improves a lot.


## Supported Bevy Versions

The following table shows the compatibility of `bevy_procedural_meshes` with certain versions of Bevy:

| bevy | bevy_procedural_meshes |
| ---- | ---------------------- |
| 0.13 | 0.1.*                  |


## License

The bevy-procedural packages are free, open source and permissively licensed. Except where noted (below and/or in individual files), all code in these repositories is dual-licensed, allowing you the flexibility to choose between:

 - The MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
 - The Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.