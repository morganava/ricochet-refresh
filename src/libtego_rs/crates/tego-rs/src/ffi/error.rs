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

/// Invoke Rust's panic! macro with given message
///
/// @param message : utf8 encoded panic! message
/// @param message_length : length of message not including null-terminator
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_panic(message: *const c_char, message_length: usize) {
    if message.is_null() || message_length == 0usize {
        panic!();
    } else {
        let message = raw_to_str!(message, message_length).unwrap();
        panic!("{message}");
    }
}
