use crate::Result;
use crate::_utility::{bytes_from_str_of_bytes};
use aes_gcm::aead::{
    generic_array::typenum::{U12, U32},
    generic_array::GenericArray,
    Aead, NewAead,
};
use aes_gcm::Aes256Gcm;
use std::sync::{Mutex, MutexGuard};

lazy_static! {
    /// Default keys generated from crate::utility::mod::tests::check_default_key_values
    /// default_encryption_key: !<<<---Origen StandardKey--->>>!
    pub static ref DEFAULT_ENCRYPTION_KEY__BYTE_STR: Mutex<String> = Mutex::new("213c3c3c2d2d2d4f726967656e205374616e646172644b65792d2d2d3e3e3e21".to_string());
    /// default_encryption_nonce: ORIGEN NONCE
    pub static ref DEFAULT_ENCRYPTION_NONCE__BYTE_STR: Mutex<String> = Mutex::new("4f524947454e204e4f4e4345".to_string());
}

#[macro_export]
macro_rules! into_aes_gcm_generic_array {
    ($byte_str: expr) => {
        *aes_gcm::aead::generic_array::GenericArray::from_slice(&crate::_utility::bytes_from_str_of_bytes($byte_str)?)
    }
}

#[allow(non_snake_case)]
pub fn default_encryption_key__byte_str<'a>() -> MutexGuard<'a, String> {
    DEFAULT_ENCRYPTION_KEY__BYTE_STR.lock().unwrap()
}

#[allow(non_snake_case)]
pub fn default_encryption_nonce__byte_str<'a>() -> MutexGuard<'a, String> {
    DEFAULT_ENCRYPTION_NONCE__BYTE_STR.lock().unwrap()
}

pub fn keygen() -> GenericArray<u8, U32> {
    let key: [u8; 32] = rand::random();
    *GenericArray::from_slice(&key)
}

pub fn noncegen() -> GenericArray<u8, U12> {
    let nonce: [u8; 12] = rand::random();
    *GenericArray::from_slice(&nonce)
}

pub fn encrypt(data: &str) -> Result<Vec<u8>> {
    encrypt_with(
        data,
        *GenericArray::from_slice(&bytes_from_str_of_bytes(
            default_encryption_key__byte_str().as_str(),
        )?),
        *GenericArray::from_slice(&bytes_from_str_of_bytes(
            default_encryption_nonce__byte_str().as_str(),
        )?),
    )
}

pub fn encrypt_with(
    data: &str,
    key: GenericArray<u8, U32>,
    nonce: GenericArray<u8, U12>,
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(&key);
    Ok(cipher.encrypt(&nonce, data.as_bytes())?)
}

pub fn decrypt(data: &[u8]) -> Result<String> {
    decrypt_with(
        data,
        *GenericArray::from_slice(&bytes_from_str_of_bytes(
            default_encryption_key__byte_str().as_str(),
        )?),
        *GenericArray::from_slice(&bytes_from_str_of_bytes(
            default_encryption_nonce__byte_str().as_str(),
        )?),
    )
}

pub fn decrypt_with(
    data: &[u8],
    key: GenericArray<u8, U32>,
    nonce: GenericArray<u8, U12>,
) -> Result<String> {
    let cipher = Aes256Gcm::new(&key);
    let plaintext = cipher.decrypt(&nonce, data)?;
    Ok(String::from_utf8(plaintext)?)
}


#[cfg(test)]
mod tests {
    use crate::_utility::{bytes_from_str_of_bytes, str_from_byte_array};
    use super::*;

    #[test]
    fn test_encrypted_roundtrip() {
        let s = "Test data!";
        let encrypted = encrypt(&s).unwrap();
        let d = decrypt(&encrypted).unwrap();
        assert_eq!(d, s);
    }

    #[test]
    fn test_str_byte_array_roundtrip() {
        let s1 = "TEST";
        let s2 = "another test";

        // Get the actual byte values of the characters in the strings
        let bytes1 = s1.as_bytes();
        let bytes2 = s2.as_bytes();

        // Convert the byte values to a string of 2-char hex values of the bytes
        // e.g.: [1, 8, 16, 32] => "01081020"
        let bs1 = str_from_byte_array(bytes1).unwrap();
        let bs2 = str_from_byte_array(bytes2).unwrap();
        assert_eq!(bs1, "54455354");
        assert_eq!(bs2, "616e6f746865722074657374");

        // Convert bytes string back to array of bytes
        // e.g.: "01081020" => [1, 8, 16, 32]
        let b1 = bytes_from_str_of_bytes(&bs1).unwrap();
        let b2 = bytes_from_str_of_bytes(&bs2).unwrap();
        assert_eq!(b1, [0x54, 0x45, 0x53, 0x54]);
        assert_eq!(
            b2,
            [0x61, 0x6e, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x20, 0x74, 0x65, 0x73, 0x74]
        );

        // Convert the byte values back to a str to check the roundtrip.
        // Note: invalid utf-8 characters may arise when this is actually used for key generation,
        // so this next step could fail.
        let r1 = std::str::from_utf8(&b1).unwrap();
        let r2 = std::str::from_utf8(&b2).unwrap();

        assert_eq!(r1, s1);
        assert_eq!(r2, s2);
    }

    #[test]
    fn check_default_key_values() {
        let s1 = "!<<<---Origen StandardKey--->>>!";
        let s2 = "ORIGEN NONCE";

        // Get the actual byte values of the characters in the strings
        let bytes1 = s1.as_bytes();
        let bytes2 = s2.as_bytes();

        // Convert the byte values to a string of 2-char hex values of the bytes
        // e.g.: [1, 8, 16, 32] => "01081020"
        let bs1 = str_from_byte_array(bytes1).unwrap();
        let bs2 = str_from_byte_array(bytes2).unwrap();

        // Run 'cargo test -- --nocapture --color always' to see these printed out
        println!("");
        println!("  Default Key: {}", bs1);
        println!("  Default nonce: {}", bs2);

        assert_eq!(bs1, default_encryption_key__byte_str().as_str());
        assert_eq!(bs2, default_encryption_nonce__byte_str().as_str());
    }

    #[test]
    fn test_keygen() {
        let k1 = keygen();
        let n1 = noncegen();
        let k2 = keygen();
        let n2 = noncegen();
        assert_ne!(k1, k2);
        assert_ne!(n1, n2);

        let s = "Key/Nonce Gen Test";
        let e1 = encrypt_with(&s, k1, n1).unwrap();
        let e2 = encrypt_with(&s, k2, n2).unwrap();
        assert_ne!(e1, e2);

        let d1 = decrypt_with(&e1, k1, n1).unwrap();
        let d2 = decrypt_with(&e2, k2, n2).unwrap();
        assert_eq!(d1, s);
        assert_eq!(d2, s);
    }
}
