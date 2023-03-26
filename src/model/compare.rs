use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Debug, Clone, Copy)]
pub struct Compare<T> {
    value: T,
    reverse: bool,
}

impl<T> Compare<T> {
    pub fn new(value: T, reverse: bool) -> Self {
        Self { value, reverse }
    }
}

impl<T: Display> Display for Compare<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq> PartialEq<Self> for Compare<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T: Eq> Eq for Compare<T> {}

impl<T: PartialOrd<T>> PartialOrd<Compare<T>> for Compare<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.reverse {
            other.value.partial_cmp(&self.value)
        } else {
            self.value.partial_cmp(&other.value)
        }
    }
}

impl<T: Ord> Ord for Compare<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.reverse {
            other.value.cmp(&self.value)
        } else {
            self.value.cmp(&other.value)
        }
    }
}

impl<T: Hash> Hash for Compare<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl<T: Default> Default for Compare<T> {
    fn default() -> Self {
        Compare::new(T::default(), false)
    }
}

impl<T> Deref for Compare<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
