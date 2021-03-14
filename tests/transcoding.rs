use codepage_strings::*;

// Do some tests on Windows code page 869 (alternate Greek),
// which is encoded by `oem_cp`. XXX This test is duplicated
// as an example in the crate doctests.

#[test]
fn test_oem_cp() {
    let coding = Coding::new(869).unwrap();
    assert_eq!(coding.encode("αβ").unwrap(), vec![214, 215]);
    assert_eq!(coding.decode(&[214, 215]).unwrap(), "αβ");
    assert_eq!(coding.decode_lossy(&[214, 147]), "α\u{fffd}");
    assert_eq!(
        coding.decode(&[214, 147]),
        Err(ConvertError::StringDecoding),
    );
}

// Do some tests on Windows code page 1257 (Baltic),
// which is encoded by `encoding_rs`.

#[test]
fn test_encoding_rs() {
    let coding = Coding::new(1257).unwrap();
    assert_eq!(coding.encode("ąž").unwrap(), vec![224, 254]);
    assert_eq!(coding.decode(&[224, 254]).unwrap(), "ąž");
    assert_eq!(coding.decode_lossy(&[224, 161]), "ą\u{fffd}");
    assert_eq!(
        coding.decode(&[224, 161]),
        Err(ConvertError::StringDecoding),
    );
}

// Do some tests on UTF-8, which is identity-encoded.

#[test]
fn test_identity() {
    let coding = Coding::new(65001).unwrap();
    let baltic = coding.encode("ąž").unwrap();
    assert_eq!(baltic, "ąž".as_bytes());
    assert_eq!(coding.decode(&baltic).unwrap(), "ąž");
    assert_eq!(coding.decode_lossy(&[65, 255]), "A\u{fffd}");
    assert_eq!(coding.decode(&[65, 255]), Err(ConvertError::StringDecoding));
}

// Do some tests on UTF-16LE, which is handled internally.

#[test]
fn test_utf16le() {
    let coding = Coding::new(1200).unwrap();
    let baltic = coding.encode("ąž").unwrap();
    assert_eq!(baltic, &[0x05, 0x01, 0x7e, 0x01]);
    assert_eq!(coding.decode(&baltic).unwrap(), "ąž");
    assert_eq!(coding.decode_lossy(&baltic), "ąž");
}
