// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! [`Utf32String`] and [`MysteryString`].

use alloc::borrow::Borrow;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::{
    fmt::{Debug, Display, Formatter, Write},
    num::NonZeroUsize,
};

#[cfg(feature = "std")]
use std::error::Error;

/// A string encoded as UTF-32.
///
/// Strings of this type can be serialized into a story file (via the
/// [`Item::Utf32String`](`crate::Item::Utf32String`) constructor) formatted
/// compatibly with the `streamstr` instruction. Constructors ensure that the
/// string is valid Unicode with no embedded nulls. Internally it's a [`Bytes`],
/// so cloning it is cheap.
///
/// This is not at all a full-featured alternative to [`std::String`](`String`).
/// `Utf32String`s are immutable once constructed and not intended for anything
/// other than being serialized into a story file.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct Utf32String(Bytes);

/// A string whose encoding is defined by the IO system, but *probably* treated
/// as Latin-1.
///
/// Strings of this type can be serialized into a story file (via the
/// [`Item::MysteryString`](`crate::Item::MysteryString`)) constructor formatted
/// compatibly with the `streamstr` instruction. Constructors ensure that it
/// will not contain any embedded nulls. Internally it's a [`Bytes`], so cloning
/// it is cheap.
///
/// This corresponds to a Glulx `E0` string, of which the spec says "the
/// encoding scheme is the business of the I/O system; in Glk, it will be the
/// Latin-1 character set". It is in any case required to be a single-byte
/// encoding which uses a zero byte as a terminator.
///
/// When building a `MysteryString` from a `char` iterator or using its
/// `Display` impl, Latin1 is assumed. However, you can also build it from a
/// `u8` iterator in which case no assumption is made about the encoding.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct MysteryString(Bytes);

/// Error returned when constructing a [`Utf32String`] or [`MysteryString`] from
/// malformed input.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringConversionError<T> {
    /// The number of errors which were encountered when encoding the string.
    pub num_errors: NonZeroUsize,
    /// The index at which the first error was encountered.
    pub first_error: usize,
    /// A lossy representation of the string.
    pub lossy: T,
}

impl<T> Display for StringConversionError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if usize::from(self.num_errors) > 1 {
            write!(
                f,
                "string conversion encountered {} unrepresentable characters, the first one at index {}.", 
                self.num_errors,
                self.first_error
            )
        } else {
            write!(
                f,
                "string conversion encountered an unrepresentable character at index {}.",
                self.first_error
            )
        }
    }
}

#[cfg(feature = "std")]
impl<T> Error for StringConversionError<T> where T: Debug {}

impl Utf32String {
    /// Construct a `Utf32String` from an iterator over `char`s (or over any type
    /// that lets you borrow a `char`).
    ///
    /// If the string contains embedded nulls, an error is returned, but a lossy
    /// version can be extracted from the error struct. The lossy string
    /// replaces nulls with `U+2400 SYMBOL FOR NULL` (␀), which belongs to the
    /// Control Pictures block, which is a really neat block that I bet you
    /// didn't know existed.
    pub fn from_chars<I, C>(chars: I) -> Result<Self, StringConversionError<Self>>
    where
        I: IntoIterator<Item = C>,
        C: Borrow<char>,
    {
        let mut num_errors: usize = 0;
        let mut first_error: usize = usize::MAX;

        let iter = chars.into_iter();
        let mut bm = BytesMut::with_capacity(4 * iter.size_hint().0);

        for (i, cref) in iter.enumerate() {
            let c = *cref.borrow();
            if c == '\0' {
                bm.put_u32('\u{2400}'.into());
                num_errors += 1;
                first_error = first_error.min(i);
            } else {
                bm.put_u32(c.into())
            }
        }

        if let Some(num_errors) = NonZeroUsize::new(num_errors) {
            Err(StringConversionError {
                num_errors,
                first_error,
                lossy: Self(bm.freeze()),
            })
        } else {
            Ok(Self(bm.freeze()))
        }
    }

    /// Like [`from_chars`](`Self::from_chars`), but in case of error will
    /// silently unwrap the error and return the lossy version.
    pub fn from_chars_lossy<I, C>(chars: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Borrow<char>,
    {
        match Self::from_chars(chars) {
            Ok(s) => s,
            Err(e) => e.lossy,
        }
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the length of the string in characters.
    pub fn char_len(&self) -> usize {
        self.0.len() / 4
    }

    /// Returns the length of the string in bytes, excluding prefix and null
    /// terminator.
    pub fn byte_len(&self) -> usize {
        self.0.len()
    }

    /// Returns the length of the string in bytes, including prefix and null
    /// terminator.
    pub fn byte_len_with_prefix_and_nul(&self) -> usize {
        self.0.len() + 8
    }

    /// Returns a clone of the underlying [`Bytes`].
    pub fn to_bytes(&self) -> Bytes {
        self.clone().into_bytes()
    }

    /// Unwraps the string into its underlying [`Bytes`].
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

impl Display for Utf32String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = self.0.clone();

        while buf.has_remaining() {
            let c: char = buf
                .get_u32()
                .try_into()
                .expect("Utf32String should always contain valid characters");
            f.write_char(c)?
        }

        Ok(())
    }
}

impl Debug for Utf32String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self.to_string();
        f.debug_tuple("Utf32String").field(&s).finish()
    }
}

impl TryFrom<String> for Utf32String {
    type Error = StringConversionError<Utf32String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Utf32String::from_chars(value.chars())
    }
}

impl TryFrom<&String> for Utf32String {
    type Error = StringConversionError<Utf32String>;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Utf32String::from_chars(value.chars())
    }
}

impl TryFrom<&str> for Utf32String {
    type Error = StringConversionError<Utf32String>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Utf32String::from_chars(value.chars())
    }
}

impl TryFrom<&[char]> for Utf32String {
    type Error = StringConversionError<Utf32String>;

    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        Utf32String::from_chars(value)
    }
}

impl MysteryString {
    /// Constructs a `MysteryString` from an iterator over `char`s (or over any
    /// type that lets you borrow a `char`).
    ///
    /// If the string contains embedded nulls or any character which cannot be
    /// represented in Latin-1, an error is returned, but a lossy version can be
    /// extracted from the error struct. The lossy string replaces nulls and
    /// unrepresentable characters with `b'?'`.
    pub fn from_chars<I, C>(chars: I) -> Result<Self, StringConversionError<Self>>
    where
        I: IntoIterator<Item = C>,
        C: Borrow<char>,
    {
        let mut num_errors: usize = 0;
        let mut first_error: usize = usize::MAX;

        let iter = chars.into_iter();
        let mut bm = BytesMut::with_capacity(4 * iter.size_hint().0);

        for (i, cref) in iter.enumerate() {
            match u8::try_from(*cref.borrow()) {
                Ok(b) if b != 0 => {
                    bm.put_u8(b);
                }
                _ => {
                    bm.put_u8(b'?');
                    num_errors += 1;
                    first_error = first_error.min(i);
                }
            }
        }

        if let Some(num_errors) = NonZeroUsize::new(num_errors) {
            Err(StringConversionError {
                num_errors,
                first_error,
                lossy: Self(bm.freeze()),
            })
        } else {
            Ok(Self(bm.freeze()))
        }
    }

    /// Constructs a `MysteryString` from an iterator over `u8`s (or over any
    /// type that lets you borrow a `u8`).
    ///
    /// If the string contains embedded nulls, an error is returned, but a lossy
    /// version can be extracted from the error struct. The lossy string is
    /// truncated at the first occurence of a null. (Unlike `from_chars`, this
    /// constructor doesn't that the string is Latin-1 or even any ASCII
    /// superset, so therefore it can't know what would be a reasonable
    /// replacement character to substitute.)
    pub fn from_bytes<I, C>(chars: I) -> Result<Self, StringConversionError<Self>>
    where
        I: IntoIterator<Item = C>,
        C: Borrow<u8>,
    {
        let mut failed = false;
        let mut num_errors: usize = 0;
        let mut first_error: usize = usize::MAX;

        let iter = chars.into_iter();
        let mut bm = BytesMut::with_capacity(4 * iter.size_hint().0);

        for (i, bref) in iter.enumerate() {
            let b = *bref.borrow();

            if b != 0 {
                // We use this separate boolean rather than testing on
                // num_errors == 0 so the optimizer can prove that it's
                // monotonic. The num_errors increment could wrap if we're
                // handed an infinite iterator.
                if !failed {
                    bm.put_u8(b);
                }
            } else {
                failed = true;
                num_errors += 1;
                first_error = first_error.min(i);
            }
        }

        if let Some(num_errors) = NonZeroUsize::new(num_errors) {
            Err(StringConversionError {
                num_errors,
                first_error,
                lossy: Self(bm.freeze()),
            })
        } else {
            Ok(Self(bm.freeze()))
        }
    }

    /// Like [`from_chars`](`Self::from_chars`), but in case of error will
    /// silently unwrap the error and return the lossy version.
    pub fn from_chars_lossy<I, C>(chars: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Borrow<char>,
    {
        match Self::from_chars(chars) {
            Ok(s) => s,
            Err(e) => e.lossy,
        }
    }

    /// Like [`from_bytes`](`Self::from_bytes`), but in case of error will
    /// silently unwrap the error and return the lossy version.
    pub fn from_bytes_lossy<I, C>(chars: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Borrow<u8>,
    {
        match Self::from_bytes(chars) {
            Ok(s) => s,
            Err(e) => e.lossy,
        }
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the length of the string, excluding prefix and null terminator.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the length of the string, including prefix and null terminator.
    pub fn len_with_prefix_and_nul(&self) -> usize {
        self.len() + 2
    }

    /// Returns a clone of the string's underlying `Bytes`.
    pub fn to_bytes(&self) -> Bytes {
        self.clone().into_bytes()
    }

    /// Unwraps the string into its underlying `Bytes`.
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

impl Display for MysteryString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = self.0.clone();
        while buf.has_remaining() {
            let byte = buf.get_u8();
            let c = char::from_u32(byte.into()).unwrap();
            f.write_char(c)?
        }

        Ok(())
    }
}

impl Debug for MysteryString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self.to_string();
        f.debug_tuple("MysteryString").field(&s).finish()
    }
}

impl TryFrom<String> for MysteryString {
    type Error = StringConversionError<MysteryString>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        MysteryString::from_chars(value.chars())
    }
}

impl TryFrom<&String> for MysteryString {
    type Error = StringConversionError<MysteryString>;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        MysteryString::from_chars(value.chars())
    }
}

impl TryFrom<&str> for MysteryString {
    type Error = StringConversionError<MysteryString>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        MysteryString::from_chars(value.chars())
    }
}

impl TryFrom<Vec<u8>> for MysteryString {
    type Error = StringConversionError<MysteryString>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        MysteryString::from_bytes(value)
    }
}

impl TryFrom<&Vec<u8>> for MysteryString {
    type Error = StringConversionError<MysteryString>;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        MysteryString::from_bytes(value)
    }
}

impl TryFrom<&[u8]> for MysteryString {
    type Error = StringConversionError<MysteryString>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        MysteryString::from_bytes(value)
    }
}
