// standard
use std::ffi::c_char;

// extern
use anyhow::Result;
use tor_interface::tor_crypto::V3OnionServiceId;

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

/// Checks if a service id string is valid per tor rend spec:
/// https://gitweb.torproject.org/torspec.git/tree/rend-spec-v3.txt
///
/// @param service_id_string : string containing the v3 service id to be validated
/// @param service_id_string_length : length of service_id_string not counting the
///  null terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_v3_onion_service_id_string_is_valid(
    service_id_string: *const c_char,
    service_id_string_length: usize,
    error: *mut *mut tego_error,
) -> bool {
    translate_failures(false, error, || -> Result<bool> {
        bail_if_null!(service_id_string);

        let service_id_string = raw_to_str!(service_id_string, service_id_string_length)?;

        Ok(V3OnionServiceId::is_valid(service_id_string))
    })
}

/// Construct a service id object from string. Validates
/// the checksum and version byte per spec:
/// https://gitweb.torproject.org/torspec.git/tree/rend-spec-v3.txt
///
/// @param out_service_id : returned v3 onion service id
/// @param service_id_string : a string containing a v3 service id
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_v3_onion_service_id_from_string(
    out_service_id: *mut *mut tego_v3_onion_service_id,
    service_id_string: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_service_id);
        bail_if_null!(service_id_string);

        let service_id_string = Handle::try_from(service_id_string)?;
        let service_id_string = tego_string_map()
            .get(&service_id_string)?
            .clone()
            .into_string()?;
        let service_id = V3OnionServiceId::from_string(&service_id_string)?;
        let service_id = tego_v3_onion_service_id_map().insert(service_id);

        unsafe {
            *out_service_id = service_id.into();
        }
        Ok(())
    })
}

/// Construct a service id object fromm private key.
///
/// @param out_service_id : returned v3 onion service id
/// @param private_key: the ed25519 private key to drive service id from
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_v3_onion_service_id_from_ed25519_private_key(
    out_service_id: *mut *mut tego_v3_onion_service_id,
    private_key: *const tego_ed25519_private_key,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_service_id);
        bail_if_null!(private_key);

        let private_key = Handle::try_from(private_key)?;
        let service_id =
            V3OnionServiceId::from_private_key(tego_ed25519_private_key_map().get(&private_key)?);
        let service_id = tego_v3_onion_service_id_map().insert(service_id);

        unsafe {
            *out_service_id = service_id.into();
        }
        Ok(())
    })
}

/// Serializes out a service id object as a null-terminated utf8 string
/// to provided character buffer.
///
/// @param service_id : v3 onion service id object to serialize
/// @param out_service_id_string : destination buffer for string
/// @param service_id_string_size : size of out_service_id_string buffer in
///  bytes, must be at least 57 bytes (56 bytes for string + null
///  terminator)
/// @param error : filled on error
/// @return : number of bytes written including null terminator;
///  TEGO_V3_ONION_SERVICE_ID_SIZE (57) on success, 0 on failure
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_v3_onion_service_id_to_string(
    service_id: *const tego_v3_onion_service_id,
    out_service_id_string: *mut *mut tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(service_id);
        bail_if_null!(out_service_id_string);

        let service_id = Handle::try_from(service_id)?;
        let service_id_string = tego_v3_onion_service_id_map().get(&service_id)?.to_string();
        let service_id_string = CString::new(service_id_string)?;
        let service_id_string = tego_string_map().insert(service_id_string);
        unsafe {
            *out_service_id_string = service_id_string.into();
        }
        Ok(())
    })
}
