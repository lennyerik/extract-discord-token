mod decrypt_token;
mod electron_extract;
mod error;
mod string_key;

use error::ExtractDiscordTokenError;
use std::env;
use std::path::Path;

fn main() -> Result<(), ExtractDiscordTokenError> {
    let appdata_path_str = env::var("APPDATA")?;
    let appdata_dir = Path::new(&appdata_path_str);

    let temp_path_str = env::var("TEMP")?;
    let temp_dir = Path::new(&temp_path_str);

    let mut token = electron_extract::extract_encrypted_token(appdata_dir, temp_dir)?;
    let decrypted_key =
        decrypt_token::win32_decrypt(&mut electron_extract::extract_encrypted_key(appdata_dir)?)?;
    let decrypted_token = decrypt_token::aead_decrypt_inplace(&mut token, &decrypted_key)?;

    println!("{}", decrypted_token);

    Ok(())
}
