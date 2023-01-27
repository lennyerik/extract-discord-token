use crate::ExtractDiscordTokenError;
use aes_gcm::aead::generic_array::typenum::U12;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aes::cipher::Unsigned;
use aes_gcm::aes::Aes128;
use aes_gcm::AeadInPlace;
use aes_gcm::{aes::Aes256, AesGcm, KeyInit, Nonce};
use std::slice;
use windows::Win32::{
    Foundation::TRUE,
    Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB},
    System::Memory::LocalFree,
};

type NonceSize = U12;

pub fn win32_decrypt(input: &mut [u8]) -> Result<Vec<u8>, ExtractDiscordTokenError> {
    let blob_in = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let mut blob_out = CRYPT_INTEGER_BLOB::default();

    unsafe {
        let result = CryptUnprotectData(&blob_in, None, None, None, None, 0, &mut blob_out);

        match result {
            TRUE => {
                LocalFree(blob_out.cbData as isize);
                Ok(slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize).to_vec())
            }
            _ => Err(ExtractDiscordTokenError::Decryption(
                "CryptUnprotectData failed",
            )),
        }
    }
}

pub fn aead_decrypt_inplace<'a>(
    buf: &'a mut Vec<u8>,
    key: &[u8],
) -> Result<&'a str, ExtractDiscordTokenError> {
    // Extract the nonce
    let nonce_buf = buf.drain(0..NonceSize::USIZE).as_slice().to_owned();
    if nonce_buf.len() != NonceSize::USIZE {
        return Err(ExtractDiscordTokenError::Decryption(
            "Decrypted token is not long enough for extracting nonce",
        ));
    }
    let nonce: &GenericArray<u8, NonceSize> = Nonce::from_slice(&nonce_buf);

    match key.len() {
        16 => {
            let aes = AesGcm::<Aes128, NonceSize>::new_from_slice(key)
                .expect("AesGcm did not take a 128 Bit key as a valid key for AES128");
            aes.decrypt_in_place(nonce, &[], buf)
                .or(Err(ExtractDiscordTokenError::Decryption(
                    "AEAD decryption of the token failed",
                )))?;

            Ok(
                (std::str::from_utf8(buf)).or(Err(ExtractDiscordTokenError::Decryption(
                    "Decrypted token is not UTF-8",
                )))?,
            )
        }
        32 => {
            let aes = AesGcm::<Aes256, NonceSize>::new_from_slice(key)
                .expect("AesGcm did not take a 256 Bit key as a valid key for AES256");
            aes.decrypt_in_place(nonce, &[], buf)
                .or(Err(ExtractDiscordTokenError::Decryption(
                    "AEAD decryption of the token failed",
                )))?;

            Ok(
                (std::str::from_utf8(buf)).or(Err(ExtractDiscordTokenError::Decryption(
                    "Decrypted token is not UTF-8",
                )))?,
            )
        }
        _ => Err(ExtractDiscordTokenError::Decryption(
            "Extracted key has invalid size",
        )),
    }
}
