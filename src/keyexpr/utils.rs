#[derive(Debug)]
pub(crate) struct Splitter<'a, S: ?Sized, D: ?Sized> {
    s: Option<&'a S>,
    d: &'a D,
}
impl<S: ?Sized, D: ?Sized> Clone for Splitter<'_, S, D> {
    fn clone(&self) -> Self {
        Self {
            s: self.s,
            d: self.d,
        }
    }
}

impl<'a, S: Split<D> + ?Sized, D: ?Sized> Iterator for Splitter<'a, S, D> {
    type Item = &'a S;
    fn next(&mut self) -> Option<Self::Item> {
        match self.s {
            Some(s) => {
                let (ret, s) = s.try_split_once(self.d);
                self.s = s;
                Some(ret)
            }
            None => None,
        }
    }
}

impl<S: Split<D> + ?Sized, D: ?Sized> DoubleEndedIterator for Splitter<'_, S, D> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.s {
            Some(s) => {
                let (s, ret) = s.try_rsplit_once(self.d);
                self.s = s;
                Some(ret)
            }
            None => None,
        }
    }
}
pub(crate) trait Split<Delimiter: ?Sized> {
    fn split_once<'a>(&'a self, delimiter: &Delimiter) -> (&'a Self, &'a Self);
    fn try_split_once<'a>(&'a self, delimiter: &Delimiter) -> (&'a Self, Option<&'a Self>);
    fn try_rsplit_once<'a>(&'a self, delimiter: &Delimiter) -> (Option<&'a Self>, &'a Self);
    fn splitter<'a>(&'a self, delimiter: &'a Delimiter) -> Splitter<'a, Self, Delimiter> {
        Splitter {
            s: Some(self),
            d: delimiter,
        }
    }
}
impl Split<u8> for [u8] {
    fn split_once<'a>(&'a self, delimiter: &u8) -> (&'a Self, &'a Self) {
        match self.iter().position(|c| c == delimiter) {
            Some(i) => (&self[..i], &self[(i + 1)..]),
            None => (self, &[]),
        }
    }

    fn try_split_once<'a>(&'a self, delimiter: &u8) -> (&'a Self, Option<&'a Self>) {
        match self.iter().position(|c| c == delimiter) {
            Some(i) => (&self[..i], Some(&self[(i + 1)..])),
            None => (self, None),
        }
    }

    fn try_rsplit_once<'a>(&'a self, delimiter: &u8) -> (Option<&'a Self>, &'a Self) {
        match self.iter().rposition(|c| c == delimiter) {
            Some(i) => (Some(&self[..i]), &self[(i + 1)..]),
            None => (None, self),
        }
    }
}
impl Split<[u8]> for [u8] {
    fn split_once<'a>(&'a self, delimiter: &[u8]) -> (&'a Self, &'a Self) {
        for i in 0..self.len() {
            if self[i..].starts_with(delimiter) {
                return (&self[..i], &self[(i + delimiter.len())..]);
            }
        }
        (self, &[])
    }

    fn try_split_once<'a>(&'a self, delimiter: &[u8]) -> (&'a Self, Option<&'a Self>) {
        for i in 0..self.len() {
            if self[i..].starts_with(delimiter) {
                return (&self[..i], Some(&self[(i + delimiter.len())..]));
            }
        }
        (self, None)
    }

    fn try_rsplit_once<'a>(&'a self, delimiter: &[u8]) -> (Option<&'a Self>, &'a Self) {
        for i in (delimiter.len()..(self.len() + 1)).rev() {
            if self[..i].ends_with(delimiter) {
                return (Some(&self[..(i - delimiter.len())]), &self[i..]);
            }
        }
        (None, self)
    }
}
impl<const N: usize> Split<[u8; N]> for [u8] {
    fn split_once<'a>(&'a self, delimiter: &[u8; N]) -> (&'a Self, &'a Self) {
        for i in 0..self.len() {
            if self[i..].starts_with(delimiter) {
                return (&self[..i], &self[(i + delimiter.len())..]);
            }
        }
        (self, &[])
    }

    fn try_split_once<'a>(&'a self, delimiter: &[u8; N]) -> (&'a Self, Option<&'a Self>) {
        for i in 0..self.len() {
            if self[i..].starts_with(delimiter) {
                return (&self[..i], Some(&self[(i + delimiter.len())..]));
            }
        }
        (self, None)
    }

    fn try_rsplit_once<'a>(&'a self, delimiter: &[u8; N]) -> (Option<&'a Self>, &'a Self) {
        for i in (delimiter.len()..(self.len() + 1)).rev() {
            if self[..i].ends_with(delimiter) {
                return (Some(&self[..(i - delimiter.len())]), &self[i..]);
            }
        }
        (None, self)
    }
}

#[allow(dead_code)]
pub(crate) trait Utf {
    fn utf(&self) -> &str;
}

#[allow(dead_code)]
impl Utf for [u8] {
    fn utf(&self) -> &str {
        unsafe { ::core::str::from_utf8_unchecked(self) }
    }
}

#[allow(unused_macros)]
macro_rules! utfdbg {
    ($x: expr) => {{
        let x = $x;
        println!(
            "[{}:{}] {} = {}",
            file!(),
            line!(),
            stringify!($x),
            $crate::key_expr::utils::Utf::utf(x)
        );
        x
    }};
}
