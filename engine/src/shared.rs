//! Copy-on-write sharing for state parts that are cloned far more often than they change.
//!
//! `Shared<T>` behaves like a `T` with value semantics: reading goes through `Deref`, and any mutable access
//! (`DerefMut`) first makes the value unique (`Arc::make_mut`), so a mutation of one state never shows in a clone of
//! it. Cloning is a reference-count increment instead of a deep copy. `Debug`, `PartialEq`, `Eq`, `Ord`, `Hash`
//! and serde all delegate to the inner value, so every observable output is the same as with a plain `T`.

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct Shared<T>(Arc<T>);

impl<T> Shared<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Shared(Arc::new(value))
    }
}

impl<T: Clone> Shared<T> {
    /// The value, without copying when this is the only reference.
    #[inline]
    pub fn into_inner(self) -> T {
        Arc::unwrap_or_clone(self.0)
    }
}

impl<T: Clone + IntoIterator> IntoIterator for Shared<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.0).into_iter()
    }
}

impl<T> Clone for Shared<T> {
    #[inline]
    fn clone(&self) -> Self {
        Shared(Arc::clone(&self.0))
    }
}

impl<T: Default> Default for Shared<T> {
    #[inline]
    fn default() -> Self {
        Shared(Arc::new(T::default()))
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> DerefMut for Shared<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.0)
    }
}

impl<T> From<T> for Shared<T> {
    #[inline]
    fn from(value: T) -> Self {
        Shared::new(value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.0, f)
    }
}

impl<T: PartialEq> PartialEq for Shared<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl<T: Eq> Eq for Shared<T> {}

impl<T: PartialOrd> PartialOrd for Shared<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (*self.0).partial_cmp(&*other.0)
    }
}

impl<T: Ord> Ord for Shared<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self.0).cmp(&*other.0)
    }
}

impl<T: std::hash::Hash> std::hash::Hash for Shared<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (*self.0).hash(state)
    }
}

impl<T: serde::Serialize> serde::Serialize for Shared<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (*self.0).serialize(s)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Shared<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        T::deserialize(d).map(Shared::new)
    }
}

impl<A, T: FromIterator<A>> FromIterator<A> for Shared<T> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        Shared::new(T::from_iter(iter))
    }
}

impl<'a, T> IntoIterator for &'a Shared<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&*self.0).into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut Shared<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Arc::make_mut(&mut self.0).into_iter()
    }
}

/// `Shared<T>` plus one memo slot for bytes derived from the value (the canonical key walk of a zone). Any mutable
/// access clears the memo, so the bytes always describe the current value; a clone starts with an empty memo.
pub struct Memo<T>(Arc<MemoInner<T>>);

struct MemoInner<T> {
    value: T,
    bytes: std::sync::OnceLock<Box<[u8]>>,
}

impl<T: Clone> Clone for MemoInner<T> {
    fn clone(&self) -> Self {
        MemoInner {
            value: self.value.clone(),
            bytes: std::sync::OnceLock::new(),
        }
    }
}

impl<T> Memo<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Memo(Arc::new(MemoInner {
            value,
            bytes: std::sync::OnceLock::new(),
        }))
    }
}

/// The bytes a zone contributes to `search_key`'s canonical walk. One implementation per zone type (in
/// `search_key.rs`), so the memo of a [`Memo`] can only ever hold that walk's bytes.
pub trait CanonicalBytes {
    fn canonical_bytes(&self) -> Vec<u8>;
}

impl<T: CanonicalBytes> Memo<T> {
    /// The canonical bytes of the current value, computed the first time they are asked for after a change.
    #[inline]
    pub fn canonical(&self) -> &[u8] {
        self.0
            .bytes
            .get_or_init(|| self.0.value.canonical_bytes().into_boxed_slice())
    }
}

impl<T: Clone> Memo<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        match Arc::try_unwrap(self.0) {
            Ok(inner) => inner.value,
            Err(shared) => shared.value.clone(),
        }
    }
}

impl<T: Clone + IntoIterator> IntoIterator for Memo<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<T> Clone for Memo<T> {
    #[inline]
    fn clone(&self) -> Self {
        Memo(Arc::clone(&self.0))
    }
}

impl<T: Default> Default for Memo<T> {
    #[inline]
    fn default() -> Self {
        Memo::new(T::default())
    }
}

impl<T> Deref for Memo<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.0.value
    }
}

impl<T: Clone> DerefMut for Memo<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        let inner = Arc::make_mut(&mut self.0);
        inner.bytes.take();
        &mut inner.value
    }
}

impl<T> From<T> for Memo<T> {
    #[inline]
    fn from(value: T) -> Self {
        Memo::new(value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0.value, f)
    }
}

impl<T: PartialEq> PartialEq for Memo<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}

impl<T: Eq> Eq for Memo<T> {}

impl<A, T: FromIterator<A>> FromIterator<A> for Memo<T> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        Memo::new(T::from_iter(iter))
    }
}

impl<'a, T> IntoIterator for &'a Memo<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.0.value).into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut Memo<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&mut **self).into_iter()
    }
}

impl<T: serde::Serialize> serde::Serialize for Memo<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.value.serialize(s)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Memo<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        T::deserialize(d).map(Memo::new)
    }
}
