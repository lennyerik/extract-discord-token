#[derive(Debug)]
pub enum ExtractDiscordTokenError {
    // Wrapped errors
    EnvVar(std::env::VarError),
    LevelDB(leveldb::error::Error),
    Base64Decode(base64::DecodeError),
    JsonDecode(serde_json::Error),
    IO(std::io::Error),

    // Custom errors
    Parsing(&'static str),
    ExpectedValue(&'static str),
    Decryption(&'static str),
}

// Error wrapper generation

macro_rules! impl_from {
    ($err_name:ident, $err:ty) => {
        impl From<$err> for ExtractDiscordTokenError {
            fn from(e: $err) -> Self {
                Self::$err_name(e)
            }
        }
    };
}

impl_from!(EnvVar, std::env::VarError);
impl_from!(LevelDB, leveldb::error::Error);
impl_from!(Base64Decode, base64::DecodeError);
impl_from!(JsonDecode, serde_json::Error);
impl_from!(IO, std::io::Error);
