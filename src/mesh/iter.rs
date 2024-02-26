use bevy::render::mesh::PrimitiveTopology;

use super::{IndexType, PMesh};

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Iterates the faces assuming a triangle list.
    pub fn iter_faces_list(&self) -> impl Iterator<Item = [usize; 3]> + '_ {
        assert!(self.topology == PrimitiveTopology::TriangleList);
        self.indices
            .chunks_exact(3)
            .map(|w| [w[0].index(), w[1].index(), w[2].index()])
    }

    /// Iterates the faces  assuming a triangle strip.
    pub fn iter_faces_strip(&self) -> impl Iterator<Item = [usize; 3]> + '_ {
        assert!(self.topology == PrimitiveTopology::TriangleStrip);
        self.indices
            .windows(3)
            .map(|w| [w[0].index(), w[1].index(), w[2].index()])
    }
}
