# codepage-strings: encode / decode strings for Windows code pages
Bart Massey 2021

This Rust crate builds on the excellent work of the
`encoding_rs`, `codepage`, and `oem-cp` crates in an attempt
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
beyond those supported by `codepage` and `oem-cp`. If the
single-byte codepage you need is missing, I suggest taking a
look at adding it to `oem-cp`, which seems to be the main
Rust repository for unusual Windows code page tables. I
believe that most of the single-byte code pages supported by
`iconv` are dealt with here, but I haven't checked
carefully.

Multibyte code pages are not (for now) currently supported —
in particular code page 1200 (UTF-16LE) and code page 1201
(UTF-16BE), but also various Asian languages. Code page
65001 (UTF-8) is supported as an identity transformation.
EBCDIC code pages are not supported and are low priority,
because seriously?

No particular effort has been put into performance. The
interface allows `std::borrow::Cow` to some extent, but this
is limited by the minor impedance mismatches between
`encoding_rs` and `oem-cp`.

## Examples

Do some string conversions on Windows code page 869
(alternate Greek).

```rust

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

This crate is made available under the "MIT"
license. Please see the file `LICENSE` in this distribution
for license terms.

Thanks to the `cargo-readme` crate for generation of this `README`.
