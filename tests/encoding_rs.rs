// Do some tests on Windows code page 1257 (Baltic),
// which is encoded by `encoding_rs`.

use codepage_strings::*;

#[test]
fn test_encoding_rs() {
    let coding = Coding::new(1257).unwrap();
    assert_eq!(
        coding.encode("ąž").unwrap(),
        vec![224, 254],
    );
    assert_eq!(
        coding.decode(&[224, 254]).unwrap(),
        "ąž",
    );
    assert_eq!(
        coding.decode_lossy(&[224, 161]),
        "ą\u{fffd}",
    );
    assert_eq!(
        coding.decode(&[224, 161]),
        Err(ConvertError::StringDecoding),
    );
}
