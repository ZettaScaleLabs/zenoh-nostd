use core::{borrow::Borrow, fmt, str::FromStr};

use heapless::{String, Vec};
use zenoh_result::{zerr, ZResult, ZE};

use crate::core::CowStr;

pub(super) const LIST_SEPARATOR: char = ';';
pub(super) const FIELD_SEPARATOR: char = '=';
pub(super) const VALUE_SEPARATOR: char = '|';

fn split_once(s: &str, c: char) -> (&str, &str) {
    match s.find(c) {
        Some(index) => {
            let (l, r) = s.split_at(index);
            (l, &r[1..])
        }
        None => (s, ""),
    }
}

/// Returns an iterator of key-value `(&str, &str)` pairs according to the parameters format.
pub fn iter(s: &str) -> impl DoubleEndedIterator<Item = (&str, &str)> + Clone {
    s.split(LIST_SEPARATOR)
        .filter(|p| !p.is_empty())
        .map(|p| split_once(p, FIELD_SEPARATOR))
}

/// Same as [`from_iter_into`] but keys are sorted in alphabetical order.
pub fn sort<'s, I, const N: usize>(iter: I) -> impl Iterator<Item = (&'s str, &'s str)>
where
    I: Iterator<Item = (&'s str, &'s str)>,
{
    let mut from = iter.collect::<Vec<(&str, &str), N>>();
    from.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));
    from.into_iter()
}

/// Joins two key-value `(&str, &str)` iterators removing from `current` any element whose key is present in `new`.
pub fn join<'s, C, N>(current: C, new: N) -> impl Iterator<Item = (&'s str, &'s str)> + Clone
where
    C: Iterator<Item = (&'s str, &'s str)> + Clone,
    N: Iterator<Item = (&'s str, &'s str)> + Clone + 's,
{
    let n = new.clone();
    let current = current
        .clone()
        .filter(move |(kc, _)| !n.clone().any(|(kn, _)| kn == *kc));
    current.chain(new)
}

/// Builds a string from an iterator preserving the order.
#[allow(clippy::should_implement_trait)]
pub fn from_iter<'s, I, const N: usize>(iter: I) -> ZResult<String<N>>
where
    I: Iterator<Item = (&'s str, &'s str)>,
{
    let mut into = String::new();
    from_iter_into(iter, &mut into)?;
    Ok(into)
}

/// Same as [`from_iter`] but it writes into a user-provided string instead of allocating a new one.
pub fn from_iter_into<'s, I, const N: usize>(iter: I, into: &mut String<N>) -> ZResult<()>
where
    I: Iterator<Item = (&'s str, &'s str)>,
{
    concat_into(iter, into)
}

/// Get the a `&str`-value for a `&str`-key according to the parameters format.
pub fn get<'s>(s: &'s str, k: &str) -> Option<&'s str> {
    iter(s).find(|(key, _)| *key == k).map(|(_, value)| value)
}

/// Get the a `&str`-value iterator for a `&str`-key according to the parameters format.
pub fn values<'s>(s: &'s str, k: &str) -> impl DoubleEndedIterator<Item = &'s str> {
    match get(s, k) {
        Some(v) => v.split(VALUE_SEPARATOR),
        None => {
            let mut i = "".split(VALUE_SEPARATOR);
            i.next();
            i
        }
    }
}

fn _insert<'s, I>(
    i: I,
    k: &'s str,
    v: &'s str,
) -> (impl Iterator<Item = (&'s str, &'s str)>, Option<&'s str>)
where
    I: Iterator<Item = (&'s str, &'s str)> + Clone,
{
    let mut iter = i.clone();
    let item = iter.find(|(key, _)| *key == k).map(|(_, v)| v);

    let current = i.filter(move |x| x.0 != k);
    let new = Some((k, v)).into_iter();
    (current.chain(new), item)
}

/// Insert a key-value `(&str, &str)` pair by appending it at the end of `s` preserving the insertion order.
pub fn insert<'s, const N: usize>(
    s: &'s str,
    k: &'s str,
    v: &'s str,
) -> (ZResult<String<N>>, Option<&'s str>) {
    let (iter, item) = _insert(iter(s), k, v);
    (from_iter(iter), item)
}

/// Same as [`insert`] but keys are sorted in alphabetical order.
pub fn insert_sort<'s, const N: usize>(
    s: &'s str,
    k: &'s str,
    v: &'s str,
) -> (ZResult<String<N>>, Option<&'s str>) {
    let (iter, item) = _insert(iter(s), k, v);
    (from_iter(sort::<'s, _, N>(iter)), item)
}

/// Remove a key-value `(&str, &str)` pair from `s` preserving the insertion order.
pub fn remove<'s, const N: usize>(s: &'s str, k: &str) -> (ZResult<String<N>>, Option<&'s str>) {
    let mut iter = iter(s);
    let item = iter.find(|(key, _)| *key == k).map(|(_, v)| v);
    let iter = iter.filter(|x| x.0 != k);
    (concat(iter), item)
}

/// Returns `true` if all keys are sorted in alphabetical order
pub fn is_ordered(s: &str) -> bool {
    let mut prev = None;
    for (k, _) in iter(s) {
        match prev.take() {
            Some(p) if k < p => return false,
            _ => prev = Some(k),
        }
    }
    true
}

fn concat<'s, I, const N: usize>(iter: I) -> ZResult<String<N>>
where
    I: Iterator<Item = (&'s str, &'s str)>,
{
    let mut into = String::new();
    concat_into(iter, &mut into)?;
    Ok(into)
}

fn concat_into<'s, I, const N: usize>(iter: I, into: &mut String<N>) -> ZResult<()>
where
    I: Iterator<Item = (&'s str, &'s str)>,
{
    let mut first = true;
    for (k, v) in iter.filter(|(k, _)| !k.is_empty()) {
        if !first {
            into.push(LIST_SEPARATOR)
                .map_err(|_| zerr!(ZE::CapacityExceeded))?;
        }
        into.push_str(k).map_err(|_| zerr!(ZE::CapacityExceeded))?;
        if !v.is_empty() {
            into.push(FIELD_SEPARATOR)
                .map_err(|_| zerr!(ZE::CapacityExceeded))?;
            into.push_str(v).map_err(|_| zerr!(ZE::CapacityExceeded))?;
        }
        first = false;
    }

    Ok(())
}

/// A map of key/value (String,String) parameters.
/// It can be parsed from a String, using `;` or `<newline>` as separator between each parameters
/// and `=` as separator between a key and its value. Keys and values are trimmed.
///
/// Example:
/// ```
/// use zenoh_protocol::core::Parameters;
///
/// let a = "a=1;b=2;c=3|4|5;d=6";
/// let p = Parameters::<32>::from(a);
///
/// // Retrieve values
/// assert!(!p.is_empty());
/// assert_eq!(p.get("a").unwrap(), "1");
/// assert_eq!(p.get("b").unwrap(), "2");
/// assert_eq!(p.get("c").unwrap(), "3|4|5");
/// assert_eq!(p.get("d").unwrap(), "6");
/// assert_eq!(p.values("c").collect::<Vec<&str>>(), vec!["3", "4", "5"]);
///
/// // Iterate over parameters
/// let mut iter = p.iter();
/// assert_eq!(iter.next().unwrap(), ("a", "1"));
/// assert_eq!(iter.next().unwrap(), ("b", "2"));
/// assert_eq!(iter.next().unwrap(), ("c", "3|4|5"));
/// assert_eq!(iter.next().unwrap(), ("d", "6"));
/// assert!(iter.next().is_none());
///
/// // Create parameters from iterators
/// let pi = Parameters::from_iter(vec![("a", "1"), ("b", "2"), ("c", "3|4|5"), ("d", "6")]);
/// assert_eq!(p, pi);
/// ```
#[derive(Clone, Hash, Default)]
pub struct Parameters<'s, const N: usize>(CowStr<'s, N>);

impl<'s, const N: usize> Parameters<'s, N> {
    /// Create empty parameters.
    pub const fn empty() -> Self {
        Self(CowStr::Borrowed(""))
    }

    /// Returns `true` if parameters does not contain anything.
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().is_empty()
    }

    /// Returns parameters as [`str`].
    pub fn as_str(&'s self) -> &'s str {
        self.0.as_ref()
    }

    /// Returns `true` if parameters contains the specified key.
    pub fn contains_key<K>(&self, k: K) -> bool
    where
        K: Borrow<str>,
    {
        super::parameters::get(self.as_str(), k.borrow()).is_some()
    }

    /// Returns a reference to the `&str`-value corresponding to the key.
    pub fn get<K>(&'s self, k: K) -> Option<&'s str>
    where
        K: Borrow<str>,
    {
        super::parameters::get(self.as_str(), k.borrow())
    }

    /// Returns an iterator to the `&str`-values corresponding to the key.
    pub fn values<K>(&'s self, k: K) -> impl DoubleEndedIterator<Item = &'s str>
    where
        K: Borrow<str>,
    {
        super::parameters::values(self.as_str(), k.borrow())
    }

    /// Returns an iterator on the key-value pairs as `(&str, &str)`.
    pub fn iter(&'s self) -> impl DoubleEndedIterator<Item = (&'s str, &'s str)> + Clone {
        super::parameters::iter(self.as_str())
    }

    /// Inserts a key-value pair into the map.
    /// If the map did not have this key present, [`None`]` is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    pub fn insert<K, V, const L: usize>(&mut self, k: K, v: V) -> ZResult<Option<String<L>>>
    where
        K: Borrow<str>,
        V: Borrow<str>,
    {
        let (inner, item) = super::parameters::insert(self.as_str(), k.borrow(), v.borrow());
        let item = item.map(|i| String::from_str(i).unwrap_or_default());
        self.0 = CowStr::Owned(inner?);
        Ok(item)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the parameters.
    pub fn remove<K, const L: usize>(&mut self, k: K) -> ZResult<Option<String<L>>>
    where
        K: Borrow<str>,
    {
        let (inner, item) = super::parameters::remove(self.as_str(), k.borrow());
        let item = item.map(|i| String::from_str(i).unwrap_or_default());
        self.0 = CowStr::Owned(inner?);
        Ok(item)
    }

    /// Extend these parameters with other parameters.
    pub fn extend<const L: usize>(&mut self, other: &Parameters<L>) -> ZResult<()> {
        self.extend_from_iter(other.iter())
    }

    /// Extend these parameters from an iterator.
    pub fn extend_from_iter<'e, I, K, V>(&mut self, iter: I) -> ZResult<()>
    where
        I: Iterator<Item = (&'e K, &'e V)> + Clone,
        K: Borrow<str> + 'e + ?Sized,
        V: Borrow<str> + 'e + ?Sized,
    {
        let inner = super::parameters::from_iter(super::parameters::join(
            self.iter(),
            iter.map(|(k, v)| (k.borrow(), v.borrow())),
        ));
        self.0 = CowStr::Owned(inner?);

        Ok(())
    }

    /// Convert these parameters into owned parameters.
    pub fn into_owned(self) -> ZResult<Parameters<'static, N>> {
        Ok(Parameters(CowStr::Owned(self.0.into_owned()?)))
    }

    /// Returns `true`` if all keys are sorted in alphabetical order.
    pub fn is_ordered(&self) -> bool {
        super::parameters::is_ordered(self.as_str())
    }
}

impl<const N: usize> PartialEq for Parameters<'_, N> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl<const N: usize> Eq for Parameters<'_, N> {}

impl<'s, const N: usize> From<&'s str> for Parameters<'s, N> {
    fn from(mut value: &'s str) -> Self {
        value = value.trim_end_matches(|c| {
            c == LIST_SEPARATOR || c == FIELD_SEPARATOR || c == VALUE_SEPARATOR
        });
        Self(CowStr::Borrowed(value))
    }
}

impl<const N: usize> From<String<N>> for Parameters<'_, N> {
    fn from(mut value: String<N>) -> Self {
        let s = value.trim_end_matches(|c| {
            c == LIST_SEPARATOR || c == FIELD_SEPARATOR || c == VALUE_SEPARATOR
        });
        value.truncate(s.len());
        Self(CowStr::Owned(value))
    }
}

impl<'s, const N: usize> From<CowStr<'s, N>> for Parameters<'s, N> {
    fn from(value: CowStr<'s, N>) -> Self {
        match value {
            CowStr::Borrowed(s) => Parameters::from(s),
            CowStr::Owned(s) => Parameters::from(s),
        }
    }
}

impl<'s, K, V, const N: usize> FromIterator<(&'s K, &'s V)> for Parameters<'_, N>
where
    K: Borrow<str> + 's + ?Sized,
    V: Borrow<str> + 's + ?Sized,
{
    fn from_iter<T: IntoIterator<Item = (&'s K, &'s V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let inner = super::parameters::from_iter(iter.map(|(k, v)| (k.borrow(), v.borrow())));
        Self(CowStr::Owned(inner.expect("Parameters::from_iter failed")))
    }
}

impl<'s, K, V, const N: usize> FromIterator<&'s (K, V)> for Parameters<'_, N>
where
    K: Borrow<str> + 's,
    V: Borrow<str> + 's,
{
    fn from_iter<T: IntoIterator<Item = &'s (K, V)>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(|(k, v)| (k.borrow(), v.borrow())))
    }
}

impl<'s, K, V, const N: usize> From<&'s [(K, V)]> for Parameters<'_, N>
where
    K: Borrow<str> + 's,
    V: Borrow<str> + 's,
{
    fn from(value: &'s [(K, V)]) -> Self {
        Self::from_iter(value.iter())
    }
}

impl<const N: usize> fmt::Display for Parameters<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const N: usize> fmt::Debug for Parameters<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parameters() {
        type Parameters = super::Parameters<'static, 256>;

        assert!(Parameters::from("").0.as_ref().is_empty());

        assert_eq!(Parameters::from("p1"), Parameters::from(&[("p1", "")][..]));

        assert_eq!(
            Parameters::from("p1=v1"),
            Parameters::from(&[("p1", "v1")][..])
        );

        assert_eq!(
            Parameters::from("p1=v1;p2=v2;"),
            Parameters::from(&[("p1", "v1"), ("p2", "v2")][..])
        );

        assert_eq!(
            Parameters::from("p1=v1;p2=v2;|="),
            Parameters::from(&[("p1", "v1"), ("p2", "v2")][..])
        );

        assert_eq!(
            Parameters::from("p1=v1;p2;p3=v3"),
            Parameters::from(&[("p1", "v1"), ("p2", ""), ("p3", "v3")][..])
        );

        assert_eq!(
            Parameters::from("p1=v 1;p 2=v2"),
            Parameters::from(&[("p1", "v 1"), ("p 2", "v2")][..])
        );

        assert_eq!(
            Parameters::from("p1=x=y;p2=a==b"),
            Parameters::from(&[("p1", "x=y"), ("p2", "a==b")][..])
        );
    }
}
