// extern

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;
use crate::settings::SettingsFile;

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
    out_settings: *mut *mut tego_settings,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_settings);

        let settings = SettingsFile::load_default()?;
        let handle = tego_settings_map().insert(settings);
        unsafe {
            *out_settings = handle.into();
        }
        Ok(())
    });
}

/// Initialize a new tego_settings by attempting to read
/// from a particular location
///
/// @param out_settings : returned settings
/// @param settings_file_path: utf8 path to settings file
/// @param settings_file_path_length: length of settings_file_path not
///  including null-terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_settings_load(
    out_settings: *mut *mut tego_settings,
    settings_file_path: *const c_char,
    settings_file_path_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_settings);
        bail_if_null!(settings_file_path);
        bail_if_equal!(settings_file_path_length, 0usize);

        let settings_file_path = raw_to_str!(settings_file_path, settings_file_path_length)?;
        let settings = SettingsFile::load_custom(settings_file_path.into())?;

        let handle = tego_settings_map().insert(settings);
        unsafe {
            *out_settings = handle.into();
        }
        Ok(())
    });
}

/// Write current settings to disk
///
/// @param settings : the settings object to write to disk
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_settings_flush(
    settings: *mut tego_settings,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(settings);

        let handle = Handle::try_from(settings)?;
        tego_settings_map().get_mut(&handle)?.flush()?;
        Ok(())
    });
}

//
// Getters
//

//
// Setters
//
