use super::{IndexType, PMesh};

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Iterates the faces.
    pub fn iter_faces(&self) -> impl Iterator<Item = [usize; 3]> + '_ {
        self.indices
            .chunks_exact(3)
            .map(|w| [w[0].index(), w[1].index(), w[2].index()])
    }
}
