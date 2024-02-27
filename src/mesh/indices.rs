use crate::IndexType;
use std::ops::Index;

/// A list of indices of type T.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PIndices<T>
where
    T: IndexType,
{
    indices: Vec<T>,
}

impl<T> Index<usize> for PIndices<T>
where
    T: IndexType,
{
    type Output = T;

    #[inline(always)]
    fn index<'a>(&'a self, i: usize) -> &'a T {
        &self.indices[i]
    }
}

impl<T> PIndices<T>
where
    T: IndexType,
{
    /// Creates a new PIndices with an empty list of indices.
    pub fn new() -> Self {
        PIndices {
            indices: Vec::new(),
        }
    }

    /// Returns the number of indices in the PIndices.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Builds a new PIndices with the given vector of indices consuming the vector.
    pub fn build(indices: Vec<T>) -> PIndices<T> {
        PIndices { indices }
    }

    /// Push a triangle to the list of indices.
    pub fn push(&mut self, a: T, b: T, c: T) -> &mut Self {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
        self
    }

    /// Overwrites the indices at the given index with the given values.
    pub fn overwrite(&mut self, i: usize, a: T, b: T, c: T) -> &mut Self {
        self.indices[3 * i + 0] = a;
        self.indices[3 * i + 1] = b;
        self.indices[3 * i + 2] = c;
        self
    }

    /// Returns the triangle at the given index.
    pub fn get_triangle(&self, i: usize, rotate: usize) -> (T, T, T) {
        (
            self.indices[3 * i + (rotate + 0) % 3],
            self.indices[3 * i + (rotate + 1) % 3],
            self.indices[3 * i + (rotate + 2) % 3],
        )
    }

    /// Returns a reference to the vector of indices.
    pub fn get_indices(&self) -> &Vec<T> {
        &self.indices
    }

    /// Returns a mutable reference to the vector of indices.
    pub fn get_indices_mut(&mut self) -> &mut Vec<T> {
        &mut self.indices
    }

    /// Returns an iterator over the indices.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.indices.iter()
    }

    /// Returns an mutable iterator over the indices.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.indices.iter_mut()
    }

    /// Iterates the indices as usize.
    pub fn iter_usize<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.indices.iter().map(|x: &T| x.index())
    }

    /// Returns an iterator over chunk_size elements of the slice at a time, starting at the beginning of the slice.
    pub fn chunks_exact(&self, size: usize) -> impl Iterator<Item = &[T]> {
        self.indices.chunks_exact(size)
    }

    /// Returns an iterator over all contiguous windows of length size. The windows overlap. If the slice is shorter than size, the iterator returns no values.
    pub fn windows(&self, size: usize) -> impl Iterator<Item = &[T]> {
        self.indices.windows(size)
    }

    /// Appends indices from the other PIndices to this one.
    pub fn extend(&mut self, other: &PIndices<T>) -> &mut Self {
        self.indices.extend(other.indices.iter().cloned());
        self
    }

    /// Modifies the indices in place using the given function.
    pub fn map(&mut self, f: impl Fn(T) -> T) -> &mut Self {
        for i in &mut self.indices {
            *i = f(*i);
        }
        self
    }

    /// Returns a clone of the PIndices with the indices reversed.
    pub fn reversed(&self) -> PIndices<T> {
        let mut indices = self.indices.clone();
        indices.reverse();
        PIndices { indices }
    }

    /// Adds a reversed copy of the indices to the end of the list.
    pub fn add_backfaces(&mut self) -> &mut PIndices<T> {
        self.extend(&self.reversed())
    }

    /// Resets the indices to the given range.
    pub fn reset_to_interval(&mut self, start: T, end: T) -> &mut PIndices<T> {
        self.indices = (start.index()..end.index()).map(|i| T::new(i)).collect();
        self
    }

    /// Convert the indices to a bevy mesh index type.
    /// The appropriate width is chosen based on the size of T.
    pub fn get_bevy(&self) -> bevy::render::mesh::Indices {
        if std::mem::size_of::<T>() == std::mem::size_of::<u32>() {
            bevy::render::mesh::Indices::U32(
                self.indices
                    .clone()
                    .into_iter()
                    .map(|x| x.index() as u32)
                    .collect(),
            )
        } else if std::mem::size_of::<T>() == std::mem::size_of::<u16>()
            || std::mem::size_of::<T>() == std::mem::size_of::<u8>()
        {
            bevy::render::mesh::Indices::U16(
                self.indices
                    .clone()
                    .into_iter()
                    .map(|x| x.index() as u16)
                    .collect(),
            )
        } else {
            panic!("Unsupported index type");
        }
    }

    /// Converts the indices to a triangle strip.
    /// Inserts two degenerate triangles between each triangle in the list.
    /// TODO: This is not the most efficient way to do this...
    pub fn triangle_list_to_triangle_strip(&self) -> PIndices<T> {
        // TODO: use meshopt for this

        let mut indices = Vec::new();
        for face in self.chunks_exact(3) {
            indices.push(face[0]);
            indices.push(face[0]);
            indices.push(face[1]);
            indices.push(face[2]);
            indices.push(face[2]);
        }
        PIndices { indices }
    }

    /// Converts the indices to a triangle list.
    pub fn triangle_strip_to_triangle_list(&self) -> PIndices<T> {
        let mut indices = Vec::new();
        for face in self.windows(3) {
            indices.push(face[0]);
            indices.push(face[1]);
            indices.push(face[2]);
        }
        PIndices { indices }
    }
}
