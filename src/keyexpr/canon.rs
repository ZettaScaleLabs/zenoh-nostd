use heapless::String;

pub(crate) trait Canonize {
    fn canonize(&mut self);
}

fn canonize(bytes: &mut [u8]) -> usize {
    let mut index = 0;
    let mut written = 0;
    let mut double_wild = false;
    loop {
        match &bytes[index..] {
            [b'*', b'*'] => {
                bytes[written..written + 2].copy_from_slice(b"**");
                written += 2;
                return written;
            }
            [b'*', b'*', b'/', ..] => {
                double_wild = true;
                index += 3;
            }
            [b'*', r @ ..] | [b'$', b'*', r @ ..] if r.is_empty() || r.starts_with(b"/") => {
                let (end, len) = (!r.starts_with(b"/"), r.len());
                bytes[written] = b'*';
                written += 1;
                if end {
                    if double_wild {
                        bytes[written..written + 3].copy_from_slice(b"/**");
                        written += 3;
                    }
                    return written;
                }
                bytes[written] = b'/';
                written += 1;
                index = bytes.len() - len + 1;
            }

            [b'$', b'*', b'$', b'*', ..] => {
                index += 2;
            }
            _ => {
                if double_wild && &bytes[index..] != b"**" {
                    bytes[written..written + 3].copy_from_slice(b"**/");
                    written += 3;
                    double_wild = false;
                }
                let mut write_start = index;
                loop {
                    match bytes.get(index) {
                        Some(b'/') => {
                            index += 1;
                            bytes.copy_within(write_start..index, written);
                            written += index - write_start;
                            break;
                        }
                        Some(b'$') if matches!(bytes.get(index + 1..index + 4), Some(b"*$*")) => {
                            index += 2;
                            bytes.copy_within(write_start..index, written);
                            written += index - write_start;
                            let skip = bytes[index + 4..]
                                .windows(2)
                                .take_while(|s| s == b"$*")
                                .count();
                            index += (1 + skip) * 2;
                            write_start = index;
                        }
                        Some(_) => index += 1,
                        None => {
                            bytes.copy_within(write_start..index, written);
                            written += index - write_start;
                            return written;
                        }
                    }
                }
            }
        }
    }
}

impl Canonize for &mut str {
    fn canonize(&mut self) {
        let bytes = unsafe { self.as_bytes_mut() };
        let length = canonize(bytes);
        bytes[length..].fill(b'\0');
        *self = &mut core::mem::take(self)[..length];
    }
}

impl<const N: usize> Canonize for String<N> {
    fn canonize(&mut self) {
        let bytes = unsafe { self.as_mut_vec() };
        let length = canonize(bytes);
        bytes.truncate(length);
    }
}
