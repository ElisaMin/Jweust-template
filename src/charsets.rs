use std::io::{BufReader, Read};
use encoding_rs::{Encoding};
use once_cell::sync::Lazy;
use crate::var::CHARSET_STDOUT;

static CHARSET_STDOUT_:Lazy<Option<&Encoding>> = Lazy::new(|| {
    CHARSET_STDOUT.map(label_to_encoding)
});
pub trait CharsetConverter {
    fn encode_from_std(&mut self) -> String;
}
impl CharsetConverter for &[u8] {
    fn encode_from_std(&mut self) -> String {
        let charset = CHARSET_STDOUT_.as_ref();
        if let Some(&encoding) = charset {
            let (r, _,err) = encoding.decode(self);
            if !err {
                return r.to_string();
            }
        }
        String::from_utf8_lossy(self).to_string()
    }
}

impl CharsetConverter for Vec<u8>  {
    #[inline]
    fn encode_from_std(&mut self) -> String {
        self.as_slice().encode_from_std()
    }
}

impl CharsetConverter for dyn Read {
    #[inline]
    fn encode_from_std(&mut self) -> String {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf).unwrap();
        buf.as_slice().encode_from_std()
    }
}

impl <R:Read> CharsetConverter for BufReader<R> {
    #[inline]
    fn encode_from_std(&mut self) -> String {
        self.buffer().encode_from_std()
    }
}

fn label_to_encoding(charset: &str) -> &Encoding {
    Encoding::for_label(charset.as_bytes()).unwrap()
}