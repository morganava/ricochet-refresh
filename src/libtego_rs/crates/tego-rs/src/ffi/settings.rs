// extern

// internal
use crate::error::translate_failures;
use crate::ffi::*;

/// Initialize a new tego_settings by attempting to read
/// from the default location.
///
/// @param out_settings : returned settings
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_settings_load_default(
    out_settings: *mut tego_settings,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

/// Initialize a new tego_settings by attempting to read
/// from a particular location
///
/// @param out_settings : returned settings
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_settings_load (
    out_settings: *mut tego_settings,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}
