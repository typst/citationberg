use std::num::NonZeroUsize;

use serde::Deserialize;

pub fn deserialize_bool<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<bool, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        Bool(bool),
        String(String),
    }

    let deser = StringOrBool::deserialize(deserializer)?;
    Ok(match deser {
        StringOrBool::Bool(b) => b,
        StringOrBool::String(s) => s.to_ascii_lowercase() == "true",
    })
}

pub fn deserialize_bool_option<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<bool>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        Bool(bool),
        String(String),
    }

    let res = Option::<StringOrBool>::deserialize(deserializer)?;
    Ok(res.map(|s| match s {
        StringOrBool::Bool(b) => b,
        StringOrBool::String(s) => s.to_ascii_lowercase() == "true",
    }))
}

pub fn deserialize_u32<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUnsigned {
        Unsigned(u32),
        String(String),
    }

    let res = StringOrUnsigned::deserialize(deserializer)?;
    Ok(match res {
        StringOrUnsigned::Unsigned(u) => u,
        StringOrUnsigned::String(s) => {
            s.trim().parse().map_err(serde::de::Error::custom)?
        }
    })
}

pub fn deserialize_u32_option<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUnsigned {
        Unsigned(u32),
        String(String),
    }

    let res = Option::<StringOrUnsigned>::deserialize(deserializer)?;
    res.map(|s| match s {
        StringOrUnsigned::Unsigned(u) => Ok(u),
        StringOrUnsigned::String(s) => s.trim().parse().map_err(serde::de::Error::custom),
    })
    .transpose()
}

/// Split `s` into maximal chunks such that two successive chars satisfy `pred`.
///
/// Returns an iterator over these chunks.
pub(crate) fn group_by<'a, F>(s: &'a str, pred: F) -> GroupBy<'a, F>
where
    F: FnMut(char, char) -> bool,
{
    GroupBy::new(s, pred)
}

/// An iterator over string slice in (non-overlapping) chunks separated by a predicate.
///
/// Adapted from the nightly std.
pub(crate) struct GroupBy<'a, P> {
    string: &'a str,
    predicate: P,
}

impl<'a, P> GroupBy<'a, P> {
    pub(crate) fn new(string: &'a str, predicate: P) -> Self {
        GroupBy { string, predicate }
    }
}

impl<'a, P> Iterator for GroupBy<'a, P>
where
    P: FnMut(char, char) -> bool,
{
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.string.is_empty() {
            None
        } else {
            let mut len = 1;
            let mut iter = windows(self.string, 2);
            while let Some(w) = iter.next() {
                let chars: Vec<_> = w.chars().collect();
                let (c, d) = (chars[0], chars[1]);
                if (self.predicate)(c, d) {
                    len += 1
                } else {
                    break;
                }
            }
            let (head, tail) = self.string.split_at(len);
            self.string = tail;
            Some(head)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.string.chars().size_hint()
    }
}

/// Return an iterator of sliding windows of size `size` over `string`.
///
/// # Panic
///
/// Panics if `size` is zero.
pub(crate) fn windows(string: &str, size: usize) -> Windows<'_> {
    assert!(size > 0);
    Windows::new(string, NonZeroUsize::new(size).unwrap())
}

/// An iterator of sliding windows of size `size` over `string`.
///
/// Each call of `next` advanced the window by one.
pub(crate) struct Windows<'a> {
    string: &'a str,
    size: NonZeroUsize,
}

impl<'a> Windows<'a> {
    pub(crate) fn new(string: &'a str, size: NonZeroUsize) -> Self {
        Self { string, size }
    }
}

impl<'a> Iterator for Windows<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size.get() > self.string.len() {
            None
        } else {
            let ret = Some(&self.string[..self.size.get()]);
            self.string = &self.string[1..];
            ret
        }
    }
}
