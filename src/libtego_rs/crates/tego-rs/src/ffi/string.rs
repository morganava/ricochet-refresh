// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

/// Initialize a new tego_string from buffer
///
/// @param out_string: destination for string
/// @param data: utf8-encoded string containing no null (0x00) bytes
/// @param data_length: the number of bytes in str not counting any null-terminator
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_string_new(
    out_string: *mut *mut tego_string,
    data: *const c_char,
    data_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_string);
        bail_if_null!(data);

        let bytes = std::slice::from_raw_parts(data as *const u8, data_length);
        let string = std::str::from_utf8(bytes)?;
        let string = CString::new(string)?;

        let handle = tego_string_map().insert(string);
        *out_string = handle.into();

        Ok(())
    });
}

/// Copy contents of given tego_string including null-terminator to external buffer
///
/// @param string: source string
/// @param out_buffer: destination for string
/// @param buffer_size: the total size of out_buffer; must be large enough to contain source string
///  including the null-terminator
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NUL
#[no_mangle]
pub unsafe extern "C" fn tego_string_get_data(
    string: *const tego_string,
    out_buffer: *mut c_char,
    buffer_size: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(string);
        bail_if_null!(out_buffer);

        let handle = Handle::try_from(string)?;
        match tego_string_map().get(&handle) {
            Ok(cstring) => {
                let bytes_with_nul = cstring.as_bytes_with_nul();
                bail_if!(bytes_with_nul.len() > buffer_size);

                let out_buffer = std::slice::from_raw_parts_mut(out_buffer as *mut u8, buffer_size);
                out_buffer.copy_from_slice(bytes_with_nul);

                Ok(())
            }
            Err(err) => Err(err),
        }
    });
}

/// Get the number of bytes in underlying tego_string including null-terminator
///
/// @param string: source string
/// @param out_size: the number of bytes required to store this string
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NUL
#[no_mangle]
pub unsafe extern "C" fn tego_string_get_size(
    string: *const tego_string,
    out_size: *mut usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(string);
        bail_if_null!(out_size);

        let handle = Handle::try_from(string)?;
        match tego_string_map().get(&handle) {
            Ok(cstring) => {
                // +1 for the null terminator
                *out_size = cstring.count_bytes() + 1usize;
                Ok(())
            }
            Err(err) => Err(err),
        }
    });
}
