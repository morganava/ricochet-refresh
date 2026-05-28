// extern
use anyhow::Result;

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

/// Convert a v3 onion service id to a user id
///
/// @param out_user_id : returned user id
/// @param service_id : input v3 onion service id
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_user_id_from_v3_onion_service_id(
    out_user_id: *mut *mut tego_user_id,
    service_id: *const tego_v3_onion_service_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_user_id);
        bail_if_null!(service_id);

        let service_id = Handle::try_from(service_id)?;
        let service_id = tego_v3_onion_service_id_map().get(&service_id)?.clone();

        let handle = tego_user_id_map().insert(service_id);
        unsafe { *out_user_id = handle.into() };

        Ok(())
    })
}

/// Get the v3 onion service id from the user id
///
/// @param user_id : input user id
/// @param out_service_id : returned v3 onion service id
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_user_id_get_v3_onion_service_id(
    user_id: *const tego_user_id,
    out_service_id: *mut *mut tego_v3_onion_service_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(user_id);
        bail_if_null!(out_service_id);

        let user_id = Handle::try_from(user_id)?;
        let service_id = tego_user_id_map().get(&user_id)?.clone();
        let service_id = tego_v3_onion_service_id_map().insert(service_id);

        unsafe { *out_service_id = service_id.into() };

        Ok(())
    })
}
