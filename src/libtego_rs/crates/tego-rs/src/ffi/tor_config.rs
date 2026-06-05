// std
use std::str::FromStr;

// external
use rico_settings::common::{BridgeConfig, FirewallConfig};
use rico_settings::v4::settings::TorConfig;
use tor_interface::proxy::*;
use tor_interface::tor_provider::TargetAddr;

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

//
// TorConfig
//

#[no_mangle]
pub unsafe extern "C" fn tego_tor_config_get_type(
    tor_config: *const tego_tor_config,
    out_tor_config_type: *mut tego_tor_config_type,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(tor_config);
        bail_if_null!(out_tor_config_type);

        let tor_config = Handle::try_from(tor_config)?;
        let tor_config_type = match tego_tor_config_map().get(&tor_config)? {
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor { .. } => tego_tor_config_type::tego_tor_config_type_bundled_tor,
            #[cfg(feature = "external-tor")]
            TorConfig::ExternalTor => tego_tor_config_type::tego_tor_config_type_external_tor,
            #[cfg(feature = "arti-client")]
            TorConfig::ArtiClient => tego_tor_config_type::tego_tor_config_type_arti_client,
        };

        unsafe {
            *out_tor_config_type = tor_config_type;
        }

        Ok(())
    });
}

/// Initialize a new empty tego_tor_config of the given type
///
/// @param out_tor_config: destination for config
/// @param bridge_config: bridge config to use; null if no bridge config required
/// @param proxy_config: proxy config to use; null if no proxy config required
/// @param fireawll_config: firewall config to use; null if no firewall config required
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
#[cfg(feature = "bundled-tor")]
pub unsafe extern "C" fn tego_tor_config_new_bundled_tor(
    out_tor_config: *mut *mut tego_tor_config,
    bridge_config: *const tego_bridge_config,
    proxy_config: *const tego_proxy_config,
    firewall_config: *const tego_firewall_config,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_tor_config);

        let bridge_config = if !bridge_config.is_null() {
            let bridge_config = Handle::try_from(bridge_config)?;
            let bridge_config = tego_bridge_config_map().get(&bridge_config)?.clone();
            Some(bridge_config)
        } else {
            None
        };

        let proxy_config = if !proxy_config.is_null() {
            let proxy_config = Handle::try_from(proxy_config)?;
            let proxy_config = tego_proxy_config_map().get(&proxy_config)?.clone();
            Some(proxy_config)
        } else {
            None
        };

        let firewall_config = if !firewall_config.is_null() {
            let firewall_config = Handle::try_from(firewall_config)?;
            let firewall_config = tego_firewall_config_map().get(&firewall_config)?.clone();
            Some(firewall_config)
        } else {
            None
        };

        let tor_config = TorConfig::BundledTor {
            bridge_config,
            proxy_config,
            firewall_config,
        };
        let tor_config = tego_tor_config_map().insert(tor_config);

        unsafe {
            *out_tor_config = tor_config.into();
        }

        Ok(())
    });
}

/// Get the tego_bridge_config from this tego_tor_config. Fails if the
/// underlying tego_tor_config does not have a tego_bridge_config.
///
/// @param tor_config: the tor config we want he briedge config of
/// @param out_bridge_config: destination to store a copy of tor_config's bridge config
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_config_get_bridge_config(
    tor_config: *const tego_tor_config,
    out_bridge_config: *mut *mut tego_bridge_config,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(tor_config);
        bail_if_null!(out_bridge_config);

        let tor_config = Handle::try_from(tor_config)?;
        let bridge_config = match tego_tor_config_map().get(&tor_config)? {
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                bridge_config: Some(bridge_config),
                ..
            } => {
                let bridge_config = tego_bridge_config_map().insert(bridge_config.clone());
                bridge_config.into()
            }
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                bridge_config: None,
                ..
            } => std::ptr::null_mut(),
        };

        unsafe {
            *out_bridge_config = bridge_config;
        }

        Ok(())
    })
}

/// Get the tego_proxy_config from this tego_tor_config. Fails if the underlying
/// tego_tor_config does not have a tego_proxy_config.
///
/// @param tor_config: the tor config we want he briedge config of
/// @param out_proxy_config: destination to store a copy of tor config's proxy config
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_config_get_proxy_config(
    tor_config: *const tego_tor_config,
    out_proxy_config: *mut *mut tego_proxy_config,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(tor_config);
        bail_if_null!(out_proxy_config);

        let tor_config = Handle::try_from(tor_config)?;
        let proxy_config = match tego_tor_config_map().get(&tor_config)? {
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                proxy_config: Some(proxy_config),
                ..
            } => {
                let proxy_config = tego_proxy_config_map().insert(proxy_config.clone());
                proxy_config.into()
            }
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                proxy_config: None, ..
            } => std::ptr::null_mut(),
        };

        unsafe {
            *out_proxy_config = proxy_config;
        }

        Ok(())
    })
}

/// Get the tego_firewall_config from this tego_tor_config. Fails if the underlying
/// tego_tor_config does not have a tego_firewall_config
///
/// @param tor_config: the tor config we want he briedge config of
/// @param out_firewall_config: destination to store a copy of tor config's firewall config
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_config_get_firewall_config(
    tor_config: *const tego_tor_config,
    out_firewall_config: *mut *mut tego_firewall_config,
    error: *mut *mut tego_error,
) {
    log_trace!();
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(tor_config);
        bail_if_null!(out_firewall_config);

        let tor_config = Handle::try_from(tor_config)?;
        let firewall_config = match tego_tor_config_map().get(&tor_config)? {
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                firewall_config: Some(firewall_config),
                ..
            } => {
                let firewall_config = tego_firewall_config_map().insert(firewall_config.clone());
                firewall_config.into()
            }
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                firewall_config: None,
                ..
            } => std::ptr::null_mut(),
        };

        unsafe {
            *out_firewall_config = firewall_config;
        }

        Ok(())
    })
}

//
// BridgeConfig
//

/// Get the type of BridgeConfig of a tego_bridge_config
///
/// @param bridge_config: the bridge config to get the type of
/// @param out_bridge_config_type: filled with the config's tego_bridge_config_type
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_bridge_config_get_type(
    bridge_config: *const tego_bridge_config,
    out_bridge_config_type: *mut tego_bridge_config_type,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(bridge_config);
        bail_if_null!(out_bridge_config_type);

        let bridge_config = Handle::try_from(bridge_config)?;
        let bridge_config_type = match tego_bridge_config_map().get(&bridge_config)? {
            BridgeConfig::BuiltIn(_) => tego_bridge_config_type::tego_bridge_config_type_builtin,
            BridgeConfig::Custom(_, _) => tego_bridge_config_type::tego_bridge_config_type_custom,
        };

        unsafe {
            *out_bridge_config_type = bridge_config_type;
        }

        Ok(())
    });
}

// BuiltIn BridgeConfig

/// Initialize a BridgeConfig using built-in bridges
///
/// @param: out_bridge_config: destination for config
/// @param: builtin: the type of builtin bridge to use
/// @param: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_bridge_config_new_builtin(
    out_bridge_config: *mut *mut tego_bridge_config,
    builtin: tego_bridge_builtin,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_bridge_config);

        let bridge_config = BridgeConfig::BuiltIn(builtin.into());
        let bridge_config = tego_bridge_config_map().insert(bridge_config);
        unsafe {
            *out_bridge_config = bridge_config.into();
        }

        Ok(())
    });
}

/// Get the type of BuiltinBridge of a tego_bridge_config
/// Must be a tego_bridge_config_type_builtin
///
/// @param bridge_config: the bridge config to get the type of
/// @param out_builtin_bridge: filled with the config's tego_bridge_builtin
/// @param error: filled one error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_bridge_config_get_bridge_builtin(
    bridge_config: *const tego_bridge_config,
    out_bridge_builtin: *mut tego_bridge_builtin,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(bridge_config);
        bail_if_null!(out_bridge_builtin);

        let bridge_config = Handle::try_from(bridge_config)?;
        let bridge_builtin = match tego_bridge_config_map().get(&bridge_config)? {
            BridgeConfig::BuiltIn(builtin) => (*builtin).into(),
            _ => bail!("bridge_config not a BridgeConfig::BuiltIn"),
        };

        unsafe {
            *out_bridge_builtin = bridge_builtin;
        }

        Ok(())
    });
}

// Custom BridgeConfig

/// Initialize a BridgeConfig using custom bridge lines
///
/// @param out_bridge_config: destination for config
/// @param bridge_lines : bridge lines in one string delimitted by '\n' character
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_bridge_config_new_custom(
    out_bridge_config: *mut *mut tego_bridge_config,
    bridge_lines: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_bridge_config);
        bail_if_null!(bridge_lines);

        let handle = Handle::try_from(bridge_lines)?;
        let mut bridge_lines: Vec<BridgeLine> = Vec::default();
        for bridge_line in tego_string_map().get(&handle)?.to_str()?.split('\n') {
            // skip over empty lines
            let bridge_line = bridge_line.trim();
            if !bridge_line.is_empty() {
                let bridge_line = BridgeLine::from_str(bridge_line)?;
                bridge_lines.push(bridge_line);
            }
        }
        bail_if!(bridge_lines.is_empty());
        let first = bridge_lines.remove(0);

        let bridge_config = BridgeConfig::Custom(first, bridge_lines);
        let bridge_config = tego_bridge_config_map().insert(bridge_config);

        unsafe {
            *out_bridge_config = bridge_config.into();
        }

        Ok(())
    });
}

/// Get the number of custom bridge lines in a custom tego_bridge_config
///
/// @param bridge_config: config to get the bridge line count from
/// @param out_bridge_lines: filled with number of custom bridges lines in this config
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_bridge_config_get_bridge_lines(
    bridge_config: *const tego_bridge_config,
    out_bridge_lines: *mut *mut tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(bridge_config);
        bail_if_null!(out_bridge_lines);

        let bridge_config = Handle::try_from(bridge_config)?;
        let bridge_lines = match tego_bridge_config_map().get(&bridge_config)? {
            BridgeConfig::Custom(first, lines) => {
                let bridge_line_count = 1usize + lines.len();
                let mut bridge_lines: Vec<String> = Vec::with_capacity(bridge_line_count);
                bridge_lines.push(first.to_string());
                for bridge_line in lines {
                    bridge_lines.push(bridge_line.to_string());
                }
                CString::from_str(bridge_lines.join("\n").as_str())?
            }
            _ => bail!("bridge_config not a BridgeConfig::Custom"),
        };
        let bridge_lines = tego_string_map().insert(bridge_lines);
        unsafe {
            *out_bridge_lines = bridge_lines.into();
        }
        Ok(())
    });
}

//
// ProxyConfig
//

/// Get the type of ProxyConfig of a tego_proxy_config
///
/// @param proxy_config: the proxy config to get the type of
/// @param out_proxy_config_type: filled with the config's tego_proxy_config_type
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_proxy_config_get_type(
    proxy_config: *const tego_proxy_config,
    out_proxy_type: *mut tego_proxy_type,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(proxy_config);
        bail_if_null!(out_proxy_type);

        let proxy_config = Handle::try_from(proxy_config)?;
        let proxy_type = match tego_proxy_config_map().get(&proxy_config)? {
            ProxyConfig::Socks4(_) => tego_proxy_type::tego_proxy_type_socks4,
            ProxyConfig::Socks5(_) => tego_proxy_type::tego_proxy_type_socks5,
            ProxyConfig::Https(_) => tego_proxy_type::tego_proxy_type_https,
        };
        unsafe {
            *out_proxy_type = proxy_type;
        }
        Ok(())
    });
}

/// Create a new socks4 proxy config
///
/// @param out_proxy_config: destination for the config
/// @param host: the proxy's host
/// @param port: the proxy's port
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_proxy_config_new_socks4(
    out_proxy_config: *mut *mut tego_proxy_config,
    host: *const tego_string,
    port: u16,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_proxy_config);
        bail_if_null!(host);

        let host = Handle::try_from(host)?;
        let host = tego_string_map().get(&host)?.to_str()?.to_string();

        let address = TargetAddr::try_from((host, port))?;

        let proxy_config = ProxyConfig::Socks4(Socks4ProxyConfig::new(address)?);
        let proxy_config = tego_proxy_config_map().insert(proxy_config);
        unsafe {
            *out_proxy_config = proxy_config.into();
        }

        Ok(())
    });
}

/// Create a new socks5 proxy config
///
/// @param out_proxy_config: destination for the config
/// @param host: the proxy's host
/// @param port: the proxy's port
/// @param username: optional username
/// @param password: optional password
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_proxy_config_new_socks5(
    out_proxy_config: *mut *mut tego_proxy_config,
    host: *const tego_string,
    port: u16,
    username: *const tego_string,
    password: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_proxy_config);
        bail_if_null!(host);

        let host = Handle::try_from(host)?;
        let host = tego_string_map().get(&host)?.to_str()?.to_string();

        let address = TargetAddr::try_from((host, port))?;

        let username = if !username.is_null() {
            let username = Handle::try_from(username)?;
            Some(tego_string_map().get(&username)?.to_str()?.to_string())
        } else {
            None
        };

        let password = if !password.is_null() {
            let password = Handle::try_from(password)?;
            Some(tego_string_map().get(&password)?.to_str()?.to_string())
        } else {
            None
        };

        let proxy_config =
            ProxyConfig::Socks5(Socks5ProxyConfig::new(address, username, password)?);
        let proxy_config = tego_proxy_config_map().insert(proxy_config);
        unsafe {
            *out_proxy_config = proxy_config.into();
        }

        Ok(())
    });
}

/// Create a new https proxy config
///
/// @param out_proxy_config: destination for the config
/// @param host: the proxy's host
/// @param port: the proxy's port
/// @param username: optional username
/// @param password: optional password
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_proxy_config_new_https(
    out_proxy_config: *mut *mut tego_proxy_config,
    host: *const tego_string,
    port: u16,
    username: *const tego_string,
    password: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_proxy_config);
        bail_if_null!(host);

        let host = Handle::try_from(host)?;
        let host = tego_string_map().get(&host)?.to_str()?.to_string();

        let address = TargetAddr::try_from((host, port))?;

        let username = if !username.is_null() {
            let username = Handle::try_from(username)?;
            Some(tego_string_map().get(&username)?.to_str()?.to_string())
        } else {
            None
        };

        let password = if !password.is_null() {
            let password = Handle::try_from(password)?;
            Some(tego_string_map().get(&password)?.to_str()?.to_string())
        } else {
            None
        };

        let proxy_config = ProxyConfig::Https(HttpsProxyConfig::new(address, username, password)?);
        let proxy_config = tego_proxy_config_map().insert(proxy_config);
        unsafe {
            *out_proxy_config = proxy_config.into();
        }

        Ok(())
    });
}

/// Get the address of this proxy
///
/// @param proxy_config: the proxy config
/// @param out_host: destination for the proxy's hostname
/// @param out_port: destination for the proxy's port
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_proxy_config_get_address(
    proxy_config: *const tego_proxy_config,
    out_host: *mut *mut tego_string,
    out_port: *mut u16,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(proxy_config);
        bail_if_null!(out_host);
        bail_if_null!(out_port);

        let proxy_config = Handle::try_from(proxy_config)?;
        let address = match tego_proxy_config_map().get(&proxy_config)? {
            ProxyConfig::Socks4(config) => config.address(),
            ProxyConfig::Socks5(config) => config.address(),
            ProxyConfig::Https(config) => config.address(),
        }
        .clone();
        let (host, port) = (address.host(), address.port());
        let host = CString::from_str(host.as_str())?;

        let host = tego_string_map().insert(host);
        unsafe {
            *out_host = host.into();
            *out_port = port;
        }
        Ok(())
    });
}

/// Get the login credentials of this proxy
///
/// @param proxy_config: the proxy config
/// @param out_username: destination for username
/// @param out_password: destination for password
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_proxy_config_get_credentials(
    proxy_config: *const tego_proxy_config,
    out_username: *mut *mut tego_string,
    out_password: *mut *mut tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(proxy_config);
        bail_if_null!(out_username);
        bail_if_null!(out_password);

        let proxy_config = Handle::try_from(proxy_config)?;
        let (username, password) = match tego_proxy_config_map().get(&proxy_config)? {
            ProxyConfig::Socks4(_) => bail!("socks4 proxy does not support credentials"),
            ProxyConfig::Socks5(config) => (config.username().clone(), config.password().clone()),
            ProxyConfig::Https(config) => (config.username().clone(), config.password().clone()),
        };

        let username = if let Some(username) = username {
            let username = CString::from_str(username.as_str())?;
            let username = tego_string_map().insert(username);
            username.into()
        } else {
            std::ptr::null_mut()
        };

        let password = if let Some(password) = password {
            let password = CString::from_str(password.as_str())?;
            let password = tego_string_map().insert(password);
            password.into()
        } else {
            std::ptr::null_mut()
        };

        unsafe {
            *out_username = username;
            *out_password = password;
        }
        Ok(())
    });
}

//
// FirewallConfig
//

/// Create a firewall config from a list of ports
///
/// @param out_firewall_config: destination for config
/// @param port_list: comma delimitted list of ports as string
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_firewall_config_new(
    out_firewall_config: *mut *mut tego_firewall_config,
    port_list: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_firewall_config);
        bail_if_null!(port_list);

        let port_list = Handle::try_from(port_list)?;
        let firewall_config =
            FirewallConfig::try_from(tego_string_map().get(&port_list)?.to_str()?)?;
        let firewall_config = tego_firewall_config_map().insert(firewall_config);

        unsafe {
            *out_firewall_config = firewall_config.into();
        }
        Ok(())
    })
}

/// Get a firewall config's port list as a comma delimited list
///
/// @param firewall_config: the config to get the port list from
/// @param out_port_list: destination to write port list
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_firewall_config_get_allowed_ports(
    firewall_config: *const tego_firewall_config,
    out_port_list: *mut *mut tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(firewall_config);
        bail_if_null!(out_port_list);

        let firewall_config = Handle::try_from(firewall_config)?;
        let port_list = tego_firewall_config_map()
            .get(&firewall_config)?
            .allowed_ports()
            .into_iter()
            .map(|port| port.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let port_list = CString::new(port_list)?;
        let port_list = tego_string_map().insert(port_list);

        unsafe {
            *out_port_list = port_list.into();
        }

        Ok(())
    })
}
