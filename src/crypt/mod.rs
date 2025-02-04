use cipher::generic_array::{
    typenum::{U32, U64},
    GenericArray,
};

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

use crate::error::CryptographyError;

pub(crate) mod ciphers;
pub(crate) mod kdf;

pub(crate) fn calculate_hmac(
    elements: &[&[u8]],
    key: &[u8],
) -> Result<GenericArray<u8, U32>, CryptographyError> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key)?;

    for element in elements {
        mac.update(element);
    }

    let result = mac.finalize();
    Ok(result.into_bytes())
}

pub(crate) fn calculate_sha256(
    elements: &[&[u8]],
) -> Result<GenericArray<u8, U32>, CryptographyError> {
    let mut digest = Sha256::new();

    for element in elements {
        digest.update(element);
    }

    Ok(digest.finalize())
}

pub(crate) fn calculate_sha512(
    elements: &[&[u8]],
) -> Result<GenericArray<u8, U64>, CryptographyError> {
    let mut digest = Sha512::new();

    for element in elements {
        digest.update(element);
    }

    Ok(digest.finalize())
}
