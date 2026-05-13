// standard
use std::ffi::{c_char, c_void};

// extern
use anyhow::{bail, Result};
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
) -> tego_bool {
    translate_failures(TEGO_FALSE, error, || -> Result<tego_bool> {
        bail_if_null!(service_id_string);

        let service_id_string = raw_to_str!(service_id_string, service_id_string_length)?;

        if V3OnionServiceId::is_valid(service_id_string) {
            Ok(TEGO_TRUE)
        } else {
            Ok(TEGO_FALSE)
        }
    })
}

/// Construct a service id object from string. Validates
/// the checksum and version byte per spec:
/// https://gitweb.torproject.org/torspec.git/tree/rend-spec-v3.txt
///
/// @param out_service_id : returned v3 onion service id
/// @param service_id_string : a string beginning with a v3 service id
/// @param service_id_string_length : length of the service id string not
///  counting the null terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_v3_onion_service_id_from_string(
    out_service_id: *mut *mut tego_v3_onion_service_id,
    service_id_string: *const c_char,
    service_id_string_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_service_id);
        bail_if_null!(service_id_string);

        let service_id_string = raw_to_str!(service_id_string, service_id_string_length)?;

        let service_id = V3OnionServiceId::from_string(service_id_string)?;

        let object = TegoObject::V3OnionServiceId(service_id);
        let key = get_object_map().insert(object);
        unsafe {
            *out_service_id = key as *mut tego_v3_onion_service_id;
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
    out_service_id_string: *mut c_char,
    service_id_string_size: usize,
    error: *mut *mut tego_error,
) -> usize {
    translate_failures(0usize, error, || -> Result<usize> {
        bail_if_null!(service_id);
        bail_if_null!(out_service_id_string);
        bail_if!(service_id_string_size < TEGO_V3_ONION_SERVICE_ID_SIZE);

        let key = service_id as TegoKey;
        match get_object_map().get(&key) {
            Some(TegoObject::V3OnionServiceId(service_id)) => {
                let service_id = service_id.to_string();
                let service_id = service_id.as_str();
                assert!(service_id.len() == TEGO_V3_ONION_SERVICE_ID_LENGTH);

                unsafe {
                    let out_service_id_string = std::slice::from_raw_parts_mut(
                        out_service_id_string as *mut u8,
                        service_id_string_size,
                    );
                    std::ptr::copy(
                        service_id.as_ptr(),
                        out_service_id_string.as_mut_ptr(),
                        service_id.len(),
                    );
                    out_service_id_string[TEGO_V3_ONION_SERVICE_ID_LENGTH] = 0u8;
                }
                Ok(TEGO_V3_ONION_SERVICE_ID_SIZE)
            }
            Some(_) => bail!(
                "not a tego_v3_onion_service_id pointer: {:?}",
                key as *const c_void
            ),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }
    })
}
