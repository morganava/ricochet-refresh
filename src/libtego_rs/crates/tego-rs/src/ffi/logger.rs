// standard
use std::ffi::c_char;

// internal
use crate::macros::*;

//
// Logging functions
//

/// @param message : utf8 encoded 'Error' message
/// @param message_length : length of message not including null-terminator
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
#[cfg(feature = "logging")]
pub unsafe extern "C" fn tego_log_error(message: *const c_char, message_length: usize) {
    if message.is_null() && message_length > 0 {
        panic!("message pointer is null but has non-zero length");
    } else {
        let message = raw_to_str!(message, message_length).expect("message not utf8 encoded");

        crate::logger::Logger::log(crate::logger::LogLevel::Error, message.to_string());
    }
}

/// @param message : utf8 encoded 'Info' message
/// @param message_length : length of message not including null-terminator
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
#[cfg(feature = "logging")]
pub unsafe extern "C" fn tego_log_info(message: *const c_char, message_length: usize) {
    if message.is_null() && message_length > 0 {
        panic!("message pointer is null but has non-zero length");
    } else {
        let message = raw_to_str!(message, message_length).expect("message not utf8 encoded");

        crate::logger::Logger::log(crate::logger::LogLevel::Info, message.to_string());
    }
}

/// @param message : utf8 encoded 'Info' message
/// @param message_length : length of message not including null-terminator
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
#[cfg(feature = "logging")]
pub unsafe extern "C" fn tego_log_trace(
    message: *const c_char,
    message_length: usize,
    function_name: *const c_char,
    function_name_length: usize,
    source_path: *const c_char,
    source_path_length: usize,
    line_number: usize,
) {
    if message.is_null() && message_length > 0 {
        panic!("message pointer is null but has non-zero length");
    } else if function_name.is_null() {
        panic!("function_name pointer must not be null");
    } else if function_name_length == 0usize {
        panic!("function_name_length must be greater than 0");
    } else if source_path.is_null() {
        panic!("source_path pointer must not be null");
    }
    if source_path_length == 0usize {
        panic!("source_path_length must be greater than 0");
    } else {
        let message = raw_to_str!(message, message_length).expect("message not utf8 encoded");
        let function_name = raw_to_str!(function_name, function_name_length)
            .expect("function_name not utf8 encoded");
        let source_path =
            raw_to_str!(source_path, source_path_length).expect("source_path not utf8 encoded");

        crate::logger::Logger::log(
            crate::logger::LogLevel::Trace,
            format!("{function_name} in {source_path}:{line_number} {message}"),
        );
    }
}
