![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)
[![CI](https://github.com/BartMassey/codepage-strings/actions/workflows/main.yml/badge.svg)](https://github.com/BartMassey/codepage-strings/actions)
[![crates-io](https://img.shields.io/crates/v/codepage-strings.svg)](https://crates.io/crates/codepage-strings)
[![api-docs](https://docs.rs/codepage-strings/badge.svg)](https://docs.rs/codepage-strings)
[![dependency status](https://deps.rs/repo/github/BartMassey/codepage-strings/status.svg)](https://deps.rs/repo/github/BartMassey/codepage-strings)

# codepage-strings: encode / decode strings for Windows code pages
Bart Massey 2021 (version 1.0.1)

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

## Examples

Do some string conversions on Windows code page 869
(alternate Greek).

```rust
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
```

[`encoding_rs`]: http://crates.io/crates/encoding_rs
[`codepage`]: http://crates.io/crates/codepage
[`oem-cp`]: http://crates.io/crates/oem-cp
[`std::borrow::Cow`]: https://doc.rust-lang.org/nightly/alloc/borrow/enum.Cow.html

This crate is made available under the "MIT
license". Please see the file `LICENSE` in this distribution
for license terms.

Thanks to the `cargo-readme` crate for generation of this `README`.
