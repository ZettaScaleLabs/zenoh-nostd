use super::{DELIMITER, keyexpr};

trait MayHaveVerbatim {
    fn has_verbatim(&self) -> bool;
    fn has_direct_verbatim(&self) -> bool;
    unsafe fn has_direct_verbatim_non_empty(&self) -> bool {
        self.has_direct_verbatim()
    }
}

impl MayHaveVerbatim for [u8] {
    fn has_direct_verbatim(&self) -> bool {
        matches!(self, [b'@', ..])
    }
    fn has_verbatim(&self) -> bool {
        self.split(|c| *c == DELIMITER)
            .any(MayHaveVerbatim::has_direct_verbatim)
    }
    unsafe fn has_direct_verbatim_non_empty(&self) -> bool {
        unsafe { *self.get_unchecked(0) == b'@' }
    }
}

#[repr(u8)]
enum MatchComplexity {
    NoWilds = 0,
    ChunkWildsOnly = 1,
    Dsl = 2,
}

impl keyexpr {
    fn match_complexity(&self) -> MatchComplexity {
        let mut has_wilds = false;
        for &c in self.as_bytes() {
            match c {
                b'*' => has_wilds = true,
                b'$' => return MatchComplexity::Dsl,
                _ => {}
            }
        }
        if has_wilds {
            MatchComplexity::ChunkWildsOnly
        } else {
            MatchComplexity::NoWilds
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let left = self.as_bytes();
        let right = other.as_bytes();

        if left == right {
            return true;
        }

        match self.match_complexity() as u8 | other.match_complexity() as u8 {
            0 => false,
            1 => it_intersect::<false>(left, right),
            _ => it_intersect::<true>(left, right),
        }
    }
}

#[cold]
fn star_dsl_intersect(mut it1: &[u8], mut it2: &[u8]) -> bool {
    fn next(s: &[u8]) -> (u8, &[u8]) {
        (s[0], &s[1..])
    }
    while !it1.is_empty() && !it2.is_empty() {
        let (current1, advanced1) = next(it1);
        let (current2, advanced2) = next(it2);
        match (current1, current2) {
            (b'$', b'$') => {
                if advanced1.len() == 1 || advanced2.len() == 1 {
                    return true;
                }
                if star_dsl_intersect(&advanced1[1..], it2) {
                    return true;
                } else {
                    return star_dsl_intersect(it1, &advanced2[1..]);
                };
            }
            (b'$', _) => {
                if advanced1.len() == 1 {
                    return true;
                }
                if star_dsl_intersect(&advanced1[1..], it2) {
                    return true;
                }
                it2 = advanced2;
            }
            (_, b'$') => {
                if advanced2.len() == 1 {
                    return true;
                }
                if star_dsl_intersect(it1, &advanced2[1..]) {
                    return true;
                }
                it1 = advanced1;
            }
            (sub1, sub2) if sub1 == sub2 => {
                it1 = advanced1;
                it2 = advanced2;
            }
            (_, _) => return false,
        }
    }
    it1.is_empty() && it2.is_empty() || it1 == b"$*" || it2 == b"$*"
}

fn chunk_it_intersect<const STAR_DSL: bool>(it1: &[u8], it2: &[u8]) -> bool {
    it1 == b"*" || it2 == b"*" || (STAR_DSL && star_dsl_intersect(it1, it2))
}

fn chunk_intersect<const STAR_DSL: bool>(c1: &[u8], c2: &[u8]) -> bool {
    if c1 == c2 {
        return true;
    }
    if c1.has_direct_verbatim() || c2.has_direct_verbatim() {
        return false;
    }
    chunk_it_intersect::<STAR_DSL>(c1, c2)
}

fn next(s: &[u8]) -> (&[u8], &[u8]) {
    match s.iter().position(|c| *c == b'/') {
        Some(i) => (&s[..i], &s[(i + 1)..]),
        None => (s, b""),
    }
}

fn it_intersect<const STAR_DSL: bool>(mut it1: &[u8], mut it2: &[u8]) -> bool {
    while !it1.is_empty() && !it2.is_empty() {
        let (current1, advanced1) = next(it1);
        let (current2, advanced2) = next(it2);
        match (current1, current2) {
            (b"**", _) => {
                if advanced1.is_empty() {
                    return !it2.has_verbatim();
                }
                return (!unsafe { current2.has_direct_verbatim_non_empty() }
                    && it_intersect::<STAR_DSL>(it1, advanced2))
                    || it_intersect::<STAR_DSL>(advanced1, it2);
            }
            (_, b"**") => {
                if advanced2.is_empty() {
                    return !it1.has_verbatim();
                }
                return (!unsafe { current1.has_direct_verbatim_non_empty() }
                    && it_intersect::<STAR_DSL>(advanced1, it2))
                    || it_intersect::<STAR_DSL>(it1, advanced2);
            }
            (sub1, sub2) if chunk_intersect::<STAR_DSL>(sub1, sub2) => {
                it1 = advanced1;
                it2 = advanced2;
            }
            (_, _) => return false,
        }
    }
    (it1.is_empty() || it1 == b"**") && (it2.is_empty() || it2 == b"**")
}
