// standard
use std::ffi::{c_char, c_void};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

// extern
use anyhow::{bail, Result};
use tor_interface::legacy_tor_client::LegacyTorClientConfig;
use tor_interface::proxy::*;
use tor_interface::tor_provider::{DomainAddr, TargetAddr};

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

/// Returns a tor daemon config struct with default params
///
/// @param out_config : destination for config
/// @param data_directory : our desired data directory
/// @param data_directory_length : length of data_directory string not counting the
///  null termiantor
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_daemon_config_initialize(
    out_config: *mut *mut tego_tor_daemon_config,
    data_directory: *const c_char,
    data_directory_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_config);
        bail_if_null!(data_directory);
        bail_if_equal!(data_directory_length, 0usize);

        let tor_bin_path = Context::tor_bin_path()?;

        let data_directory = raw_to_str!(data_directory, data_directory_length)?;
        let data_directory = PathBuf::from(data_directory);
        let data_directory = std::path::absolute(data_directory)?;

        let bundled_tor_config = LegacyTorClientConfig::BundledTor {
            tor_bin_path,
            data_directory,
            proxy_settings: None,
            allowed_ports: None,
            pluggable_transports: None,
            bridge_lines: None,
        };

        let handle = tego_tor_daemon_config_map().insert(bundled_tor_config);
        unsafe {
            *out_config = handle.into();
        }
        Ok(())
    })
}

/// Set up SOCKS4 proxy params, overwrites any existing
/// proxy settings
///
/// @param config : config to update
/// @param address : proxy addess as encoded utf8 string
/// @param address_length : length of the address not counting
///  the null terminator
/// @param port : proxy port, 0 not allowed
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_daemon_config_set_proxy_socks4(
    config: *mut tego_tor_daemon_config,
    address: *const c_char,
    address_length: usize,
    port: u16,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(config);
        bail_if_null!(address);
        bail_if_equal!(address_length, 0usize);
        bail_if_equal!(port, 0u16);

        // convert args to a proxy config
        let address = raw_to_str!(address, address_length)?;

        let proxy_address = if let Ok(address) = Ipv4Addr::from_str(address) {
            let address = SocketAddr::new(IpAddr::V4(address), port);
            TargetAddr::Socket(address)
        } else if let Ok(address) = Ipv6Addr::from_str(address) {
            let address = SocketAddr::new(IpAddr::V6(address), port);
            TargetAddr::Socket(address)
        } else if let Ok(domain) = DomainAddr::try_from((address.to_string(), port)) {
            TargetAddr::Domain(domain)
        } else {
            bail!("address is not a valid proxy address: {address}");
        };

        let proxy_config = ProxyConfig::Socks4(Socks4ProxyConfig::new(proxy_address)?);

        // update the config
        let handle = Handle::try_from(config)?;
        if let Ok(LegacyTorClientConfig::BundledTor {
                proxy_settings,
                ..
            }) = tego_tor_daemon_config_map().get_mut(&handle) {
                *proxy_settings = Some(proxy_config);
            }

        Ok(())
    })
}

/// Set up SOCKS5 proxy params, overwrites any existing
/// proxy settings
///
/// @param config : config to update
/// @param address : proxy addess encoded as utf8 string
/// @param address_length : length of the address not counting
///  any NULL terminator
/// @param port : proxy port, 0 not allowed
/// @param username : authentication username encoded as utf8
///  string, may be NULL or empty string if not needed
/// @param username_length : length of username string not counting
///  any NULL terminator
/// @param password : authentication password encoded as utf8
///  string, may be NULL or empty string if not needed
/// @param password_length : lenght of the password string not
///  counting any NULL terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_daemon_config_set_proxy_socks5(
    config: *mut tego_tor_daemon_config,
    address: *const c_char,
    address_length: usize,
    port: u16,
    username: *const c_char,
    username_length: usize,
    password: *const c_char,
    password_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(config);
        bail_if_null!(address);
        bail_if_equal!(address_length, 0usize);
        bail_if_equal!(port, 0u16);
        bail_if!(username.is_null() && username_length != 0usize);
        bail_if!(password.is_null() && password_length != 0usize);

        // convert args to a proxy config
        let address = raw_to_str!(address, address_length)?;

        let proxy_address = if let Ok(address) = Ipv4Addr::from_str(address) {
            let address = SocketAddr::new(IpAddr::V4(address), port);
            TargetAddr::Socket(address)
        } else if let Ok(address) = Ipv6Addr::from_str(address) {
            let address = SocketAddr::new(IpAddr::V6(address), port);
            TargetAddr::Socket(address)
        } else if let Ok(domain) = DomainAddr::try_from((address.to_string(), port)) {
            TargetAddr::Domain(domain)
        } else {
            bail!("address is not a valid proxy address: {address}");
        };

        let username = if username.is_null() || username_length == 0usize {
            None
        } else {
            let username = raw_to_str!(username, username_length)?;
            Some(username.to_string())
        };

        let password = if password.is_null() || password_length == 0usize {
            None
        } else {
            let password = raw_to_str!(password, password_length)?;
            Some(password.to_string())
        };

        let proxy_config =
            ProxyConfig::Socks5(Socks5ProxyConfig::new(proxy_address, username, password)?);

        // update the config
        let handle = Handle::try_from(config)?;
        if let Ok(LegacyTorClientConfig::BundledTor {
                proxy_settings,
                ..
            }) = tego_tor_daemon_config_map().get_mut(&handle) {
                *proxy_settings = Some(proxy_config);
            }

        Ok(())
    })
}

/// Set up HTTPS proxy params, overwrites any existing
/// proxy settings
///
/// @param config : config to update
/// @param address : proxy addess encoded as utf8 string
/// @param address_length : length of the address not counting
///  any NULL terminator
/// @param port : proxy port, 0 not allowed
/// @param username : authentication username encoded as utf8
///  string, may be NULL or empty string if not needed
/// @param username_length : length of username string not counting
///  any NULL terminator
/// @param password : authentication password encoded as utf8
///  string, may be NULL or empty string if not needed
/// @param password_length : lenght of the password string not
///  counting any NULL terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_daemon_config_set_proxy_https(
    config: *mut tego_tor_daemon_config,
    address: *const c_char,
    address_length: usize,
    port: u16,
    username: *const c_char,
    username_length: usize,
    password: *const c_char,
    password_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(config);
        bail_if_null!(address);
        bail_if_equal!(address_length, 0usize);
        bail_if_equal!(port, 0u16);
        bail_if!(username.is_null() && username_length != 0usize);
        bail_if!(password.is_null() && password_length != 0usize);

        // convert args to a proxy config
        let address = raw_to_str!(address, address_length)?;

        let proxy_address = if let Ok(address) = Ipv4Addr::from_str(address) {
            let address = SocketAddr::new(IpAddr::V4(address), port);
            TargetAddr::Socket(address)
        } else if let Ok(address) = Ipv6Addr::from_str(address) {
            let address = SocketAddr::new(IpAddr::V6(address), port);
            TargetAddr::Socket(address)
        } else if let Ok(domain) = DomainAddr::try_from((address.to_string(), port)) {
            TargetAddr::Domain(domain)
        } else {
            bail!("address is not a valid proxy address: {address}");
        };

        let username = if username.is_null() || username_length == 0usize {
            None
        } else {
            let username = raw_to_str!(username, username_length)?;
            Some(username.to_string())
        };

        let password = if password.is_null() || password_length == 0usize {
            None
        } else {
            let password = raw_to_str!(password, password_length)?;
            Some(password.to_string())
        };

        let proxy_config =
            ProxyConfig::Https(HttpsProxyConfig::new(proxy_address, username, password)?);

        // update the config
        let handle = Handle::try_from(config)?;
        if let Ok(LegacyTorClientConfig::BundledTor {
                proxy_settings,
                ..
            }) = tego_tor_daemon_config_map().get_mut(&handle) {
                *proxy_settings = Some(proxy_config);
            }

        Ok(())
    })
}

/// Set the allowed ports the tor daemon may use
///
/// @param config : config to update
/// @param ports : array of allowed ports
/// @param ports_count : the number of ports in list
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_daemon_config_set_allowed_ports(
    config: *mut tego_tor_daemon_config,
    ports: *const u16,
    ports_count: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(config);

        let ports: Option<Vec<u16>> = if ports.is_null() {
            None
        } else {
            let ports = unsafe { std::slice::from_raw_parts(ports, ports_count) };
            Some(ports.into())
        };

        // update the config
        let handle = Handle::try_from(config)?;
        if let Ok(LegacyTorClientConfig::BundledTor {
                allowed_ports,
                ..
            }) = tego_tor_daemon_config_map().get_mut(&handle) {
                *allowed_ports = ports;
            }

        Ok(())
    })
}

/// Set the list of pluggable transports for tor to use
///
/// @param config : config to update
/// @param pluggable_transport_configs : array of pluggable transport configs
/// @param pluggable_transport_config_count : the number of puggable transport configs
///  being  passed in
/// @param : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_daemon_config_set_pluggable_transport_configs(
    config: *mut tego_tor_daemon_config,
    pluggable_transport_configs: *const *const tego_pluggable_transport_config,
    pluggable_transport_config_count: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(config);
        bail_if!(
            pluggable_transport_configs.is_null() && pluggable_transport_config_count != 0usize
        );
        bail_if!(
            !pluggable_transport_configs.is_null() && pluggable_transport_config_count == 0usize
        );

        let pluggable_transport_configs = if pluggable_transport_configs.is_null() {
            None
        } else {
            let pluggable_transport_configs = std::slice::from_raw_parts(
                pluggable_transport_configs,
                pluggable_transport_config_count,
            );

            let mut pluggable_transport_config_vec: Vec<PluggableTransportConfig> =
                Vec::with_capacity(pluggable_transport_config_count);
            for pluggable_transport_config in pluggable_transport_configs {
                let key = *pluggable_transport_config as TegoKey;
                match get_object_map().get(&key) {
                    Some(TegoObject::PluggableTransportConfig(pluggable_transport_config)) => {
                        pluggable_transport_config_vec.push(pluggable_transport_config.clone());
                    }
                    Some(_) => bail!(
                        "not a tego_pluggable_trasnport_config pointer: {:?}",
                        key as *const c_void
                    ),
                    None => bail!("not a valid pointer: {:?}", key as *const c_void),
                }
            }
            Some(pluggable_transport_config_vec)
        };

        // update the config
        let handle = Handle::try_from(config)?;
        if let Ok(LegacyTorClientConfig::BundledTor {
                pluggable_transports,
                ..
            }) = tego_tor_daemon_config_map().get_mut(&handle) {
                *pluggable_transports = pluggable_transport_configs;
            }

        Ok(())
    })
}

/// Set the list of bridges for tor to use
///
/// @param config : config to update
/// @param bridge_lines : array of utf8 encoded bridge-line strings
/// @param bridge_line_lengths : array of lengths of the strings stored
///  in 'bridge_lines', does not include any NULL terminators
/// @param bridge_count : the number of bridge strings being
///  passed in
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_daemon_config_set_bridges(
    config: *mut tego_tor_daemon_config,
    bridge_lines: *const *const c_char,
    bridge_line_lengths: *const usize,
    bridge_count: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(config);

        let bridge_lines_vec: Option<Vec<BridgeLine>> = if bridge_lines.is_null() {
            None
        } else {
            let bridge_lines = unsafe { std::slice::from_raw_parts(bridge_lines, bridge_count) };
            bail_if_null!(bridge_line_lengths);
            let bridge_line_lengths =
                unsafe { std::slice::from_raw_parts(bridge_line_lengths, bridge_count) };
            let mut bridge_line_vec: Vec<BridgeLine> = Vec::with_capacity(bridge_count);
            for (bridge_line, bridge_line_length) in
                bridge_lines.iter().zip(bridge_line_lengths.iter())
            {
                let bridge_line = raw_to_str!(*bridge_line, *bridge_line_length)?;
                let bridge_line = BridgeLine::from_str(bridge_line)?;

                bridge_line_vec.push(bridge_line);
            }

            Some(bridge_line_vec)
        };

        // update the config
        let handle = Handle::try_from(config)?;
        if let Ok(LegacyTorClientConfig::BundledTor {
                bridge_lines,
                ..
            }) = tego_tor_daemon_config_map().get_mut(&handle) {
                *bridge_lines = bridge_lines_vec;
            }

        Ok(())
    })
}
