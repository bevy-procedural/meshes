# Bevy Procedural: Meshes

<p align="center">
  <a href="https://crates.io/crates/bevy_procedural_meshes">
      <img src="https://img.shields.io/crates/v/bevy_procedural_meshes.svg" alt="crates.io">
  </a>
  <a href="https://github.com/bevy-procedural/meshes/actions">
      <img src="https://github.com/bevy-procedural/meshes/actions/workflows/rust.yml/badge.svg" alt="Build Status">
  </a>
  <a href="https://docs.rs/bevy_procedural_meshes">
      <img src="https://docs.rs/bevy_procedural_meshes/badge.svg" alt="documentation">
  </a>
   <a href="https://bevyengine.org/learn/book/plugin-development/#main-branch-tracking">
      <img src="https://img.shields.io/badge/Bevy%20tracking-1.3-lightblue" alt="crates.io">
  </a>
</p>

The objective of the [Bevy Procedural Project](https://bevy-procedural.org) is to provide a comprehensive suite of packages for the generation of procedural graphics, unified by a consistent API.

The [bevy_procedural_meshes](https://bevy-procedural.org/meshes)-crate is a procedural mesh builder for bevy. It can use [Lyon](https://github.com/nical/lyon) to generate 2D shapes and extrude them into 3D meshes. It also supports (very!) simple methods for off-screen rendering and Constructive Solid Geometry.

Run the editor example using `cargo watch -w editor/src -w src -x "run -p editor --profile fast-dev"`.

## WARNING

This crate is still in a _very_ early stage of development. Expect frequent API modifications, bugs, and missing features. Feel free to contribute by opening issues, pull requests or sharing your ideas in [Github Discussion](https://github.com/bevy-procedural/meshes/discussions) or the [Bevy Discord](https://discord.gg/bevy).

## License

The bevy-procedural packages are free, open source and permissively licensed. Except where noted (below and/or in individual files), all code in these repositories is dual-licensed, allowing you the flexibility to choose between:

 - The MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
 - The Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0).