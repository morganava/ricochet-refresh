// std crates
use std::str::FromStr;

// external crates
use anyhow::Context;
use rico_profile::v4::profile::{Profile, User, UserType};
use tor_interface::tor_crypto::{Ed25519PrivateKey, Ed25519PublicKey};

// internal crates
use crate::ffi::*;
use crate::macros::*;

/// Create a new profile from an ed25519 private key
///
/// @param out_profile : returned ricochet profile
/// @param destination_path : location of the profile to save new profile
/// @param host_private_key : ed25519 private identity key for host user
/// @param nickname : nickname to use for host user
/// @param password: password used to unlock the profile
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_profile_store_generate_new(
    out_profile: *mut *mut tego_profile,
    destination_path: *const tego_string,
    host_private_key: *const tego_ed25519_private_key,
    nickname: *const tego_string,
    password: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_profile);
        bail_if_null!(destination_path);
        bail_if_null!(host_private_key);
        bail_if_null!(nickname);
        bail_if_null!(password);

        let destination_path = std::path::PathBuf::from_str(
            tego_string_map()
                .get(&Handle::try_from(destination_path)?)?
                .to_str()?,
        )?;
        let destination_filename = destination_path
            .file_name()
            .context("destination path invalid")?
            .to_str()
            .context("destination filename not uf8")?;

        let mut creation_path = std::env::temp_dir();
        creation_path.push(format!(".{destination_filename}.temp"));
        let _ = std::fs::remove_file(&creation_path);

        let host_private_key = Handle::try_from(host_private_key)?;
        let host_private_key = tego_ed25519_private_key_map()
            .get(&host_private_key)?
            .clone();

        let nickname = Handle::try_from(nickname)?;
        let nickname = tego_string_map().get(&nickname)?.to_str()?.to_string();

        let profile = Profile::new_from_private_key(
            host_private_key,
            nickname,
            creation_path.as_path(),
            tego_string_map()
                .get(&Handle::try_from(password)?)?
                .to_str()?,
        )?;

        std::fs::copy(&creation_path, &destination_path)?;
        std::fs::remove_file(&creation_path)?;

        let profile = tego_profile_map().insert(profile);
        unsafe {
            *out_profile = profile.into();
        }

        Ok(())
    })
}

/// Create a new profile from a legacy profile
///
/// @param out_profile : returned ricochet profile
/// @param destination_path : location of the profile to save new profile
/// @param source_path : location of the legacy profile
/// @param nickname : nickname to use for host user
/// @param password: password used to unlock the profile
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_profile_import_legacy(
    out_profile: *mut *mut tego_profile,
    destination_path: *const tego_string,
    source_path: *const tego_string,
    nickname: *const tego_string,
    password: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_profile);
        bail_if_null!(destination_path);
        bail_if_null!(source_path);
        bail_if_null!(nickname);
        bail_if_null!(password);

        let destination_path = std::path::PathBuf::from_str(
            tego_string_map()
                .get(&Handle::try_from(destination_path)?)?
                .to_str()?,
        )?;
        let destination_filename = destination_path
            .file_name()
            .context("destination path invalid")?
            .to_str()
            .context("destination filename not uf8")?;

        let mut creation_path = std::env::temp_dir();
        creation_path.push(format!(".{destination_filename}.temp"));
        let _ = std::fs::remove_file(&creation_path);

        let source_path = Handle::try_from(source_path)?;
        let source_path =
            std::path::PathBuf::from_str(tego_string_map().get(&source_path)?.to_str()?)?;
        let v3_json = std::fs::read_to_string(source_path)?;
        let v3_profile = rico_profile::v3::profile::Profile::from_str(&v3_json)?;

        let string_map = tego_string_map();
        let profile = Profile::new_from_v3_profile(
            v3_profile,
            string_map
                .get(&Handle::try_from(nickname)?)?
                .to_str()?
                .to_string(),
            creation_path.as_path(),
            string_map.get(&Handle::try_from(password)?)?.to_str()?,
        )?;

        std::fs::copy(&creation_path, &destination_path)?;
        std::fs::remove_file(&creation_path)?;

        let profile = tego_profile_map().insert(profile);
        unsafe {
            *out_profile = profile.into();
        }

        Ok(())
    })
}

/// Try to open an existing profile
///
/// @param out_profile : returned ricochet profile
/// @param profile_path : location of the profile
/// @param password: password used to unlock the profile
/// @param error : filled on error
/// @return : TEGO_TRUE on success and TEGO_FALSE when provided password
///  is not correct
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_profile_try_open_existing(
    out_profile: *mut *mut tego_profile,
    profile_path: *const tego_string,
    password: *const tego_string,
    error: *mut *mut tego_error,
) -> tego_bool {
    translate_failures(TEGO_FALSE, error, || -> Result<tego_bool> {
        bail_if_null!(out_profile);
        bail_if_null!(profile_path);
        bail_if_null!(password);

        let profile_path = std::path::PathBuf::from_str(
            tego_string_map()
                .get(&Handle::try_from(profile_path)?)?
                .to_str()?,
        )?;

        match Profile::open(
            profile_path.as_path(),
            tego_string_map()
                .get(&Handle::try_from(password)?)?
                .to_str()?,
        ) {
            Ok(profile) => {
                let profile = tego_profile_map().insert(profile);
                unsafe {
                    *out_profile = profile.into();
                }
                return Ok(TEGO_TRUE);
            }
            Err(rico_profile::v4::error::Error::InvalidPassword) => {
                unsafe {
                    *out_profile = std::ptr::null_mut();
                }
                return Ok(TEGO_FALSE);
            }
            Err(err) => return Err(err.into()),
        }
    })
}
