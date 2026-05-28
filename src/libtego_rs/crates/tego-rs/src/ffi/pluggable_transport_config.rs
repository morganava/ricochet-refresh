// standard
use std::ffi::c_char;

// extern
use anyhow::Result;
use tor_interface::censorship_circumvention::PluggableTransportConfig;

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

/// Init a pluggable transport config struct
///
/// @param out_config : destination to write ponter to initalised configuration
/// @param binary_path : utf8 encoded absolute path to pluggable-transport binary
/// @param binary_path_length : length of binary_path not counting the null terminator
/// @param transports : pointer to array of utf8 encoded transport strings
/// @param transport_lengths: array of lengths of the strings stored in 'transports',
///  does not include any NULL terminators
/// @param transport_count: the number of transport strings being passed in
/// @param options : pointer to array of utf8 encoded option strings
/// @param option_lengths: array of lengths of the strings stored in 'options',
///  does not include any NULL terminators
/// @param option_count: the number of option strings being passed in
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_pluggable_transport_config_initialize(
    out_config: *mut *mut tego_pluggable_transport_config,
    binary_path: *const c_char,
    binary_path_length: usize,
    transports: *const *const c_char,
    transport_lengths: *const usize,
    transport_count: usize,
    options: *const *const c_char,
    option_lengths: *const usize,
    option_count: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_config);
        bail_if_null!(binary_path);
        bail_if_equal!(binary_path_length, 0usize);
        bail_if_null!(transports);
        bail_if_null!(transport_lengths);
        bail_if_null!(options);
        bail_if_null!(option_lengths);

        // construct binary path
        let binary_path = raw_to_str!(binary_path, binary_path_length)?;
        let binary_path = std::path::Path::new(binary_path);
        binary_path.canonicalize()?;

        // construct list of transports
        let transports = std::slice::from_raw_parts(transports, transport_count);
        let transport_lengths = std::slice::from_raw_parts(transport_lengths, transport_count);

        let mut transport_vec: Vec<String> = Vec::with_capacity(transport_count);
        for (transport, transport_len) in transports.iter().zip(transport_lengths.iter()) {
            bail_if!(transport.is_null());
            let transport = raw_to_str!(*transport, *transport_len)?;

            transport_vec.push(transport.to_string());
        }

        let mut pluggable_transport_config =
            PluggableTransportConfig::new(transport_vec, binary_path.into())?;

        // construct list of options
        let options = std::slice::from_raw_parts(options, option_count);
        let option_lengths = std::slice::from_raw_parts(option_lengths, option_count);

        for (option, option_len) in options.iter().zip(option_lengths.iter()) {
            bail_if!(option.is_null());
            let option = raw_to_str!(*option, *option_len)?;

            pluggable_transport_config.add_option(option.to_string());
        }

        let handle = tego_pluggable_transport_config_map().insert(pluggable_transport_config);
        unsafe {
            *out_config = handle.into();
        }
        Ok(())
    })
}
