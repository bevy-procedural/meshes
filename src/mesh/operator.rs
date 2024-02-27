use super::{IndexType, PMesh};

impl<T> std::ops::Add for &mut PMesh<T>
where
    T: IndexType,
{
    type Output = PMesh<T>;
    fn add(self, rhs: &mut PMesh<T>) -> PMesh<T> {
        let mut m = self.clone();
        m.extend(rhs);
        m
    }
}

impl<T> std::ops::Add for PMesh<T>
where
    T: IndexType,
{
    type Output = PMesh<T>;
    fn add(self, rhs: PMesh<T>) -> PMesh<T> {
        let mut m = self;
        m.extend(&rhs);
        m
    }
}
