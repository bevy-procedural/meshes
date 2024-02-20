# Bevy Procedural: Meshes

<p align="center">
  <a href="https://crates.io/crates/procedural-meshes">
      <img src="https://img.shields.io/crates/v/procedural-meshes.svg" alt="crates.io">
  </a>
  <a href="https://github.com/bevy-procedural/meshes/actions">
      <img src="https://github.com/bevy-procedural/meshes/actions/workflows/rust.yml/badge.svg" alt="Build Status">
  </a>
  <a href="https://docs.rs/procedural-meshes">
      <img src="https://docs.rs/procedural-meshes/badge.svg" alt="documentation">
  </a>
   <a href="https://bevyengine.org/learn/book/plugin-development/#main-branch-tracking">
      <img src="https://img.shields.io/badge/Bevy%20tracking-1.3-lightblue" alt="crates.io">
  </a>
</p>

The objective of the [Bevy Procedural Project](https://bevy-procedural.org) is to furnish a comprehensive suite of packages for the generation of procedural graphics, unified by a consistent API.

[Procedural Meshes](https://bevy-procedural.org/meshes) is a procedural mesh builder for bevy. It can use [Lyon](https://github.com/nical/lyon) to generate 2D shapes and extrude them into 3D meshes. It also supports (very!) simple methods for off-screen rendering and Constructive Solid Geometry.

Run the editor example using `cargo watch -w editor/src -w src -x "run -p editor"`.

## WARNING: This repository is practically empty

The Bevy Procedural ecosystem is presently in its nascent phase. Expect frequent API modifications as it is under active development, with many features yet to be implemented. We highly encourage contributions to enrich the project!

## License

The bevy-procedural packages are free, open source and permissively licensed. Except where noted (below and/or in individual files), all code in these repositories is dual-licensed, allowing you the flexibility to choose between:

 - The MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
 - The Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0).