use db_key::Key;
use std::str;

pub struct StringKey(String);

impl Key for StringKey {
    fn from_u8(src: &[u8]) -> Self {
        Self(str::from_utf8(src).unwrap_or_default().to_string())
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(self.0.as_bytes())
    }
}

impl StringKey {
    pub fn new(src: &str) -> Self {
        Self(String::from(src))
    }
}
