// standard
use std::ffi::c_char;

// internal
use crate::ffi::*;

/// Get error message form tego_error
///
/// @param error : the error object to get the message from
/// @return : null terminated string with error message whose
///  lifetime is tied to the source tego_error_t; null pointer on failure
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_error_get_message(error: *const tego_error) -> *const c_char {
    if error.is_null() {
        std::ptr::null()
    } else {
        if let Ok(handle) = Handle::try_from(error) {
            if let Ok(error) = tego_error_map().get(&handle) {
                return error.message().as_ptr();
            }
        }
        std::ptr::null()
    }
}
