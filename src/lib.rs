/*!
This Rust crate builds on the excellent work of the
`encoding_rs`, `codepage`, and `oem-cp` crates in an attempt
to provide idiomatic encoding and decoding of strings coded
according to
[Windows code pages](https://en.wikipedia.org/wiki/Windows_code_page).

Because Windows code pages are a legacy rathole, it is
difficult to transcode strings using them. Sadly, there are
still a lot of files out there that use these encodings.
This crate was specifically created for use with [RIFF](), a
file format that has code pages baked in for text
internationalization.

No effort has been made to deal with Windows code pages
beyond those supported by `codepage` and `oem-cp`. If the
codepage you need is missing, I suggest taking a look at
adding it to `oem-cp`, which seems to be the main Rust
repository for unusual Windows code page tables. I believe
that most of the code pages supported by `iconv` are dealt
with here, but I haven't checked carefully.

No particular effort has been put into performance. The
interface allows `std::borrow::Cow` to some extent, but this
is limited by the minor impedance mismatches between
`encoding_rs` and `oem-cp`.

# Examples

Do some string conversions on Windows code page 869
(alternate Greek).

```rust
# use codepage_strings::*;

let coding = Coding::new(869).unwrap();
assert_eq!(
    coding.encode("αβ").unwrap(),
    vec![214, 215],
);
assert_eq!(
    coding.decode(&[214, 215]).unwrap(),
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
```
*/

use std::borrow::Cow;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertError {
    StringEncoding,
    StringDecoding,
    CodePage,
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ConvertError::StringEncoding => "string codepage encoding error",
            ConvertError::StringDecoding => "string decoding error",
            ConvertError::CodePage => "invalid / unknown Windows code page",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for ConvertError {}

#[derive(Debug, Clone)]
enum Codings {
    ERS(&'static encoding_rs::Encoding),
    OEMCP {
        encode: &'static oem_cp::ahash::AHashMap<char, u8>,
        decode: &'static oem_cp::code_table_type::TableType,
    },
}

#[derive(Debug, Clone)]
pub struct Coding(Codings);

impl Coding {
    pub fn new(cp: u16) -> Result<Self, ConvertError> {
        if let Some(c) = codepage::to_encoding(cp) {
            return Ok(Coding(Codings::ERS(c)));
        }
        let encode = match (*oem_cp::code_table::ENCODING_TABLE_CP_MAP).get(&cp) {
            Some(e) => e,
            None => return Err(ConvertError::CodePage),
        };
        let decode = match (*oem_cp::code_table::DECODING_TABLE_CP_MAP).get(&cp) {
            Some(e) => e,
            None => return Err(ConvertError::CodePage),
        };
        Ok(Coding(Codings::OEMCP { encode, decode }))
    }

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
        }
    }

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
        }
    }

    pub fn decode_lossy<'a>(&self, src: &'a [u8]) -> Cow<'a, str> {
        match self.0 {
            Codings::ERS(c) => {
                let (out, _, _) = c.decode(src.as_ref());
                out
            }
            Codings::OEMCP { decode: dt, .. } => Cow::from(dt.decode_string_lossy(src)),
        }
    }
}
