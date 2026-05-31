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

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_start_only_single_instance(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_check_for_updates_automatically(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_language(
    _settings: *const tego_settings,
    _out_value: *mut tego_language,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_show_toolbar(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_show_desktop_notifications(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_blink_taskbar_icon(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_play_audio_notifications(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_minimize_instead_of_exit(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_show_system_tray_icon(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_minimize_to_system_tray(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_connect_automatically(
    _settings: *const tego_settings,
    _out_value: *mut tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_get_tor_config(
    _settings: *const tego_settings,
    _out_value: *mut *mut tego_tor_config,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        log_trace!();
        Ok(())
    });
}

//
// Setters
//

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_start_only_single_instance(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_check_for_updates_automatically(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_language(
    _settings: *mut tego_settings,
    _value: tego_language,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_show_toolbar(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_show_desktop_notifications(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_blink_taskbar_icon(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_play_audio_notifications(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_minimize_instead_of_exit(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_show_system_tray_icon(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_minimize_to_system_tray(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_connect_automatically(
    _settings: *mut tego_settings,
    _value: tego_bool,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn tego_settings_set_tor_config(
    _settings: *mut tego_settings,
    _value: *const tego_tor_config,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        Ok(())
    });
}
