use std::borrow::Cow;

#[non_exhaustive]
pub enum ConvertError {
    Range,
    StringEncoding,
    StringDecoding,
    CodePage,
}

enum Codings {
    ERS(&'static encoding_rs::Encoding),
    OCC {
        encode: &'static oem_cp::ahash::AHashMap<char, u8>,
        decode: &'static oem_cp::code_table_type::TableType,
    },
}

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
        Ok(Coding(Codings::OCC { encode, decode }))
    }

    pub fn encode<'a, S>(&self, src: S) -> Result<Vec<u8>, ConvertError>
        where S: Into<Cow<'a, str>>
    {
        match self.0 {
            Codings::ERS(c) => {
                let src = src.into();
                let oe = c.output_encoding();
                let (out, _, ok) = oe.encode(src.as_ref());
                if ok {
                    Ok(out.to_owned().to_vec())
                } else {
                    Err(ConvertError::StringEncoding)
                }
            }
            Codings::OCC { encode: et, .. } => {
                match oem_cp::encode_string_checked(src, et) {
                    Some(out) => Ok(out),
                    None => Err(ConvertError::StringEncoding),
                }
            },
        }
    }

    pub fn decode<'a>(&self, src: &'a [u8]) -> Result<Cow<'a, str>, ConvertError> {
        match self.0 {
            Codings::ERS(c) => {
                let (out, _, ok) = c.decode(src.as_ref());
                if ok {
                    Ok(out)
                } else {
                    Err(ConvertError::StringDecoding)
                }
            }
            Codings::OCC { decode: dt, .. } => {
                match dt.decode_string_checked(src) {
                    Some(s) => Ok(Cow::from(s)),
                    None => Err(ConvertError::StringDecoding),
                }
            },
        }
    }

    pub fn decode_lossy<'a>(&self, src: &'a [u8]) -> Cow<'a, str> {
        match self.0 {
            Codings::ERS(c) => {
                let (out, _, _) = c.decode(src.as_ref());
                out
            }
            Codings::OCC { decode: dt, .. } => {
                Cow::from(dt.decode_string_lossy(src))
            }
        }
    }
}
