#![doc(html_root_url = "https://docs.rs/codepage-strings/1.0.0")]

/*!
This Rust crate builds on the excellent work of the
[`encoding_rs`], [`codepage`], and [`oem-cp`] crates in an attempt
to provide idiomatic encoding and decoding of strings coded
according to
[Windows code pages](https://en.wikipedia.org/wiki/Windows_code_page).

Because Windows code pages are a legacy rathole, it is
difficult to transcode strings using them. Sadly, there are
still a lot of files out there that use these encodings.
This crate was specifically created for use with
[RIFF](https://www.aelius.com/njh/wavemetatools/doc/riffmci.pdf),
a file format that has code pages baked in for text
internationalization.

No effort has been made to deal with Windows code pages
beyond those supported by [`codepage`] and [`oem-cp`]. If the
single-byte codepage you need is missing, I suggest taking a
look at adding it to [`oem-cp`], which seems to be the main
Rust repository for unusual Windows code page tables. I
believe that most of the single-byte code pages supported by
`iconv` are dealt with here, but I haven't checked
carefully.

Other than UTF-16LE and UTF-16BE, multibyte Windows code
pages are not (for now) currently supported — in particular
various Asian languages. Code page 65001 (UTF-8) is
supported as an identity transformation.  UTF-32LE and
UTF32-BE are not supported. EBCDIC code pages and UTF-7 are
not supported and are low priority, because seriously?

No particular effort has been put into performance. The
interface allows [`std::borrow::Cow`] to some extent, but this
is limited by the minor impedance mismatches between
[`encoding_rs`] and [`oem-cp`].

# Examples

Do some string conversions on Windows code page 869
(alternate Greek).

```rust
# use codepage_strings::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let coding = Coding::new(869)?;
assert_eq!(
    coding.encode("αβ")?,
    vec![214, 215],
);
assert_eq!(
    coding.decode(&[214, 215])?,
    "αβ",
);
assert_eq!(
    coding.decode_lossy(&[214, 147]),
    "α\u{fffd}",
);
assert_eq!(
    coding.decode(&[214, 147]),
    Err(ConvertError::StringDecoding),
);
# Ok(())
# }
```

[`encoding_rs`]: http://crates.io/crates/encoding_rs
[`codepage`]: http://crates.io/crates/codepage
[`oem-cp`]: http://crates.io/crates/oem-cp
*/

use std::borrow::Cow;

/// Errors that can result from various conversions.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertError {
    /// Could not encode string as requested.
    StringEncoding,
    /// Could not decode string as requested.
    StringDecoding,
    /// Requested a Windows code page the library doesn't understand.
    UnknownCodepage,
    /// Requested a Windows code page the library can't do.
    UnsupportedCodepage,
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ConvertError::StringEncoding => "string codepage encoding error",
            ConvertError::StringDecoding => "string decoding error",
            ConvertError::UnknownCodepage => "invalid / unknown Windows code page",
            ConvertError::UnsupportedCodepage => "cannot transcode this Windows code page",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for ConvertError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    LE,
    BE,
}

#[derive(Debug, Clone)]
enum Codings {
    ERS(&'static encoding_rs::Encoding),
    OEMCP {
        encode: &'static oem_cp::OEMCPHashMap<char, u8>,
        decode: &'static oem_cp::code_table_type::TableType,
    },
    Identity,
    UTF16(Endian),
}

/// Coding information derived from a Windows code page.
#[derive(Debug, Clone)]
pub struct Coding(Codings);

impl Coding {
    /// Get an encoding for the given code page.
    ///
    /// # Errors
    ///
    /// Will fail with [`ConvertError::UnknownCodepage`] or
    /// [`ConvertError::UnsupportedCodepage`] if an encoding
    /// for the given page is unavailable.
    pub fn new(cp: u16) -> Result<Self, ConvertError> {
        if cp == 65001 {
            // UTF-8
            return Ok(Coding(Codings::Identity));
        }
        if cp == 1200 {
            // UTF-16LE
            return Ok(Coding(Codings::UTF16(Endian::LE)));
        }
        if cp == 1201 {
            // UTF-16BE
            return Ok(Coding(Codings::UTF16(Endian::BE)));
        }
        if [12000, 12001, 65000].contains(&cp) {
            // Weird UTF format (UTF-32LE, UTF-32BE, UTF-7).
            return Err(ConvertError::UnsupportedCodepage);
        }
        if let Some(c) = codepage::to_encoding(cp) {
            return Ok(Coding(Codings::ERS(c)));
        }
        let encode = match (*oem_cp::code_table::ENCODING_TABLE_CP_MAP).get(&cp) {
            Some(e) => e,
            None => return Err(ConvertError::UnknownCodepage),
        };
        let decode = match (*oem_cp::code_table::DECODING_TABLE_CP_MAP).get(&cp) {
            Some(e) => e,
            None => return Err(ConvertError::UnknownCodepage),
        };
        Ok(Coding(Codings::OEMCP { encode, decode }))
    }

    /// Encode a UTF-8 string into a byte vector according
    /// to this encoding.
    ///
    /// # Errors
    ///
    /// Returns [`ConvertError::StringEncoding`] if any
    /// character cannot be encoded.
    pub fn encode<'a, S>(&self, src: S) -> Result<Vec<u8>, ConvertError>
    where
        S: Into<Cow<'a, str>>,
    {
        match self.0 {
            Codings::ERS(c) => {
                let src = src.into();
                let oe = c.output_encoding();
                let (out, _, fail) = oe.encode(src.as_ref());
                if fail {
                    Err(ConvertError::StringEncoding)
                } else {
                    Ok(out.to_owned().to_vec())
                }
            }
            Codings::OEMCP { encode: et, .. } => match oem_cp::encode_string_checked(src, et) {
                Some(out) => Ok(out),
                None => Err(ConvertError::StringEncoding),
            },
            Codings::Identity => Ok(src.into().as_ref().as_bytes().to_vec()),
            Codings::UTF16(e) => {
                let encoded = src
                    .into()
                    .as_ref()
                    .encode_utf16()
                    .flat_map(|w| {
                        let lo = (w & 0xff) as u8;
                        let hi = (w >> 8) as u8;
                        let bs: Vec<u8> = match e {
                            Endian::LE => vec![lo, hi],
                            Endian::BE => vec![hi, lo],
                        };
                        bs.into_iter()
                    })
                    .collect();
                Ok(encoded)
            }
        }
    }

    /// Decode a byte vector into UTF-8 [`Cow`]`<`[`str`]`>` according
    /// to this encoding.
    ///
    /// # Errors
    ///
    /// Returns [`ConvertError::StringDecoding`] if any
    /// character cannot be decoded.
    pub fn decode<'a>(&self, src: &'a [u8]) -> Result<Cow<'a, str>, ConvertError> {
        match self.0 {
            Codings::ERS(c) => {
                let (out, _, fail) = c.decode(src.as_ref());
                if fail {
                    Err(ConvertError::StringDecoding)
                } else {
                    Ok(out)
                }
            }
            Codings::OEMCP { decode: dt, .. } => match dt.decode_string_checked(src) {
                Some(s) => Ok(Cow::from(s)),
                None => Err(ConvertError::StringDecoding),
            },
            Codings::Identity => match std::str::from_utf8(src) {
                Ok(s) => Ok(Cow::from(s)),
                Err(_) => Err(ConvertError::StringDecoding),
            },
            Codings::UTF16(e) => {
                let ws = src
                    .chunks(2)
                    .map(|bs| {
                        if bs.len() < 2 {
                            return Err(ConvertError::StringDecoding);
                        }
                        let (hi, lo) = (bs[0] as u16, bs[1] as u16);
                        match e {
                            Endian::LE => Ok((lo << 8) | hi),
                            Endian::BE => Ok((hi << 8) | lo),
                        }
                    })
                    .collect::<Result<Vec<u16>, ConvertError>>()?;
                match String::from_utf16(&ws) {
                    Ok(s) => Ok(Cow::from(s)),
                    Err(_) => Err(ConvertError::StringDecoding),
                }
            }
        }
    }

    /// Decode a byte vector into UTF-8 [`Cow`]`<`[`str`]`>` according
    /// to this encoding. Replace any bytes that cannot be
    /// encoded with the Unicode
    /// "[replacement character](https://en.wikipedia.org/wiki/Specials_%28Unicode_block%29#Replacement_character)"
    /// (`\u{fffd}`).
    pub fn decode_lossy<'a>(&self, src: &'a [u8]) -> Cow<'a, str> {
        match self.0 {
            Codings::ERS(c) => {
                let (out, _, _) = c.decode(src.as_ref());
                out
            }
            Codings::OEMCP { decode: dt, .. } => Cow::from(dt.decode_string_lossy(src)),
            Codings::Identity => match std::str::from_utf8(src) {
                Ok(s) => Cow::from(s),
                Err(_) => String::from_utf8_lossy(src),
            },
            Codings::UTF16(e) => {
                let ws: Vec<u16> = src
                    .chunks(2)
                    .map(|bs| {
                        let (hi, lo) = if bs.len() == 1 {
                            // Unicode replacement character.
                            (0xff, 0xfd)
                        } else {
                            // Big-endian by default.
                            (bs[0] as u16, bs[1] as u16)
                        };
                        match e {
                            Endian::LE => (lo << 8) | hi,
                            Endian::BE => (hi << 8) | lo,
                        }
                    })
                    .collect();
                Cow::from(String::from_utf16_lossy(&ws))
            }
        }
    }
}
