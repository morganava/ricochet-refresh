// standard
use std::ffi::{c_char, c_void};

// extern
use anyhow::{bail, Result};
use tor_interface::tor_crypto::Ed25519PrivateKey;

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

/// Securely generate a new ed25519 private key
///
/// @param out_private_key : returned ed25519 private key
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_ed25519_private_key_generate(
    out_private_key: *mut *mut tego_ed25519_private_key,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_private_key);

        let private_key = Ed25519PrivateKey::generate();
        let object = TegoObject::Ed25519PrivateKey(private_key);
        let key = get_object_map().insert(object);
        unsafe {
            *out_private_key = key as *mut tego_ed25519_private_key;
        }
        Ok(())
    })
}

/// Conversion method for converting the keyblob string returned by
/// ADD_ONION command into an ed25519_private_key_t
///
/// @param out_private_key : returned ed25519 private key
/// @param keyblob : an ED25519 keyblob string in the form
///  "ED25519-V3:abcd1234..."
/// @param keyblob_length : number of characters in keyblob not
///  counting the null terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_ed25519_private_key_from_ed25519_keyblob(
    out_private_key: *mut *mut tego_ed25519_private_key,
    keyblob: *const c_char,
    keyblob_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_private_key);
        bail_if_null!(keyblob);

        let keyblob = raw_to_str!(keyblob, keyblob_length)?;

        let private_key = if let Ok(private_key) = Ed25519PrivateKey::from_key_blob(keyblob) {
            private_key
        } else {
            // try to fall back to legacy validation if failed
            Ed25519PrivateKey::from_key_blob_legacy(keyblob)?
        };

        let object = TegoObject::Ed25519PrivateKey(private_key);
        let key = get_object_map().insert(object);
        unsafe {
            *out_private_key = key as *mut tego_ed25519_private_key;
        }
        Ok(())
    })
}

/// Conversion method for converting an ed25519 private key
/// to a null-terminated keyblob string for use with ADD_ONION
/// command
///
/// @param out_keyblob : buffer to be filled with ed25519 keyblob in
///  the form "ED25519-V3:abcd1234...\0"
/// @param keyblob_size : size of out_keyblob buffer in bytes, must be at
///  least 100 characters (99 for string + 1 for null terminator)
/// @param private_key : the private key to encode
/// @param error : filled on error
/// @return : the number of characters written (including null terminator)
///  to out_keyblob
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_ed25519_keyblob_from_ed25519_private_key(
    out_keyblob: *mut c_char,
    keyblob_size: usize,
    private_key: *const tego_ed25519_private_key,
    error: *mut *mut tego_error,
) -> usize {
    translate_failures(0usize, error, || -> Result<usize> {
        bail_if_null!(out_keyblob);
        bail_if!(keyblob_size < TEGO_ED25519_KEYBLOB_SIZE);
        bail_if_null!(private_key);

        let key = private_key as TegoKey;
        match get_object_map().get(&key) {
            Some(TegoObject::Ed25519PrivateKey(private_key)) => {
                let keyblob = private_key.to_key_blob();
                let keyblob = keyblob.as_str();
                assert!(keyblob.len() == TEGO_ED25519_KEYBLOB_LENGTH);

                unsafe {
                    let out_keyblob =
                        std::slice::from_raw_parts_mut(out_keyblob as *mut u8, keyblob_size);
                    std::ptr::copy(keyblob.as_ptr(), out_keyblob.as_mut_ptr(), keyblob.len());
                    out_keyblob[TEGO_ED25519_KEYBLOB_LENGTH] = 0u8;
                }
                Ok(TEGO_ED25519_KEYBLOB_SIZE)
            }
            Some(_) => bail!(
                "not a tego_ed25519_private_key pointer: {:?}",
                key as *const c_void
            ),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }
    })
}
