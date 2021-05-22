pub mod big_uint_helpers;
pub mod differ;
pub mod file_actions;
pub mod file_utils;
pub mod location;
#[macro_use]
pub mod logger;
pub mod command_helpers;
pub mod ldap;
pub mod mailer;
pub mod num_helpers;
pub mod session_store;
pub mod version;

use crate::{Result, STATUS};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{
    generic_array::typenum::{U12, U32},
    generic_array::GenericArray,
    Aead, NewAead,
};
use aes_gcm::Aes256Gcm;

pub fn resolve_os_str(s: &OsStr) -> Result<String> {
    Ok(s.to_os_string().into_string()?)
}

/// Resolves a directory path from the current application root.
/// Accepts an optional 'user_val' and a default. The resulting directory will be resolved from:
/// 1. If a user value is given, and its absolute, this is the final path.
/// 2. If a user value is given but its not absolute, then the final path is the user path relative to the application root.
/// 3. If no user value is given, the final path is the default path relative to the root.
/// Notes:
///   A default is required, but an empty default will point to the application root.
///   The default is assumed to be relative. Absolute defaults are not supported.
pub fn resolve_dir_from_app_root(user_val: Option<&String>, default: &str) -> PathBuf {
    let offset;
    if let Some(user_str) = user_val {
        if Path::new(&user_str).is_absolute() {
            return PathBuf::from(user_str);
        } else {
            offset = user_str.to_string();
        }
    } else {
        offset = default.to_string();
    }
    let mut dir = STATUS.origen_wksp_root.clone();
    dir.push(offset);
    dir
}

/// Checks the given values of a vector against an enumerated set of accepted values.
/// Optionally, check for duplicate items as well.
pub fn check_vec<T: std::cmp::Eq + std::hash::Hash + std::fmt::Display, V>(
    vut: &Vec<T>,
    valid_values_hashmap: &HashMap<T, V>,
    allow_duplicates: bool,
    obj_name: &str,
    container_name: &str,
) -> Result<()> {
    let mut indices = HashMap::new();
    for (i, d) in vut.iter().enumerate() {
        if valid_values_hashmap.contains_key(d) {
            if !allow_duplicates {
                if let Some(idx) = indices.get(d) {
                    // Duplicate dataset
                    return error!(
                        "{} '{}' can only appear once in the {} (first appearance at index {} - duplicate at index {})",
                        obj_name,
                        d,
                        container_name,
                        idx,
                        i
                    );
                }
            }
            indices.insert(d, i);
        } else {
            return error!(
                "'{}' is not a valid {} and cannot be used in the {}",
                d, obj_name, container_name
            );
        }
    }
    Ok(())
}

pub fn str_to_bool(s: &str) -> Result<bool> {
    match s {
        "true" | "True" => Ok(true),
        "false" | "False" => Ok(false),
        _ => error!("Could not convert string {} to boolean value", s),
    }
}

/// For convenience in converting to/from configs, allow bytes to be converted
/// to a string representation of byte values (NOT characters themselves).
/// This allows for invalid UTF-8 characters to still be encoded in a string.
pub fn bytes_from_str_of_bytes(s: &str) -> Result<Vec<u8>> {
    let mut bytes = vec![];
    for chrs in s.chars().collect::<Vec<char>>().chunks(2) {
        let s = format!("{}{}", chrs[0], chrs[1]);
        bytes.push(
            u8::from_str_radix(&s, 16)
                .expect(&format!("Could not interpret byte {} as a hex value", s)),
        );
    }
    Ok(bytes)
}

pub fn str_from_byte_array(bytes: &[u8]) -> Result<String> {
    let mut s = String::new();
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    Ok(s)
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
            &crate::ORIGEN_CONFIG.default_encryption_key,
        )?),
        *GenericArray::from_slice(&bytes_from_str_of_bytes(
            &crate::ORIGEN_CONFIG.default_encryption_nonce,
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
            &crate::ORIGEN_CONFIG.default_encryption_key,
        )?),
        *GenericArray::from_slice(&bytes_from_str_of_bytes(
            &crate::ORIGEN_CONFIG.default_encryption_nonce,
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

pub fn unsorted_dedup<T: Eq + std::hash::Hash + Clone>(v: &mut Vec<T>) {
    let mut uniques = std::collections::HashSet::new();
    v.retain(|e| uniques.insert(e.clone()));
}

#[cfg(test)]
mod tests {
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

        assert_eq!(bs1, crate::ORIGEN_CONFIG.default_encryption_key);
        assert_eq!(bs2, crate::ORIGEN_CONFIG.default_encryption_nonce);
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
