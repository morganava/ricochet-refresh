// std
use std::fmt::{Display, Formatter};
use std::str::FromStr;

// extern
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tor_interface::censorship_circumvention::BridgeLine;
use tor_interface::proxy::*;
use tor_interface::tor_provider::TargetAddr;

// crate
use crate::common::{BridgeConfig, BuiltInBridge, FirewallConfig};
use crate::v3;

//
// Settings Raw
//

#[derive(Deserialize, Serialize)]
pub struct SettingsRaw {
    version: Version,
    // general settings
    start_only_single_instance: bool,
    check_for_updates_automatically: bool,
    // interface settings
    language: Language,
    show_toolbar: bool,
    button_style: ButtonStyle,
    show_desktop_notifications: bool,
    blink_taskbar_icon: bool,
    play_audio_notifications: bool,
    minimize_instead_of_exit: bool,
    show_system_tray_icon: bool,
    minimize_to_system_tray: bool,
    // connection settings
    connect_automatically: bool,
    tor_config: TorConfigRaw,
}

impl From<&Settings> for SettingsRaw {
    fn from(value: &Settings) -> SettingsRaw {
        let version = Version::V4_0_0;
        let start_only_single_instance = value.start_only_single_instance;
        let check_for_updates_automatically = value.check_for_updates_automatically;
        let language = value.language;
        let show_toolbar = value.show_toolbar;
        let button_style = value.button_style;
        let show_desktop_notifications = value.show_desktop_notifications;
        let blink_taskbar_icon = value.blink_taskbar_icon;
        let play_audio_notifications = value.play_audio_notifications;
        let minimize_instead_of_exit = value.minimize_instead_of_exit;
        let show_system_tray_icon = value.show_system_tray_icon;
        let minimize_to_system_tray = value.minimize_to_system_tray;
        let connect_automatically = value.connect_automatically;
        let tor_config = TorConfigRaw::from(&value.tor_config);

        SettingsRaw {
            version,
            start_only_single_instance,
            check_for_updates_automatically,
            language,
            show_toolbar,
            button_style,
            show_desktop_notifications,
            blink_taskbar_icon,
            play_audio_notifications,
            minimize_instead_of_exit,
            show_system_tray_icon,
            minimize_to_system_tray,
            connect_automatically,
            tor_config,
        }
    }
}

//
// Settings
//
#[derive(Debug, PartialEq)]
pub struct Settings {
    // general settings
    pub start_only_single_instance: bool,
    pub check_for_updates_automatically: bool,
    // interface settings
    pub language: Language,
    pub show_toolbar: bool,
    pub button_style: ButtonStyle,
    pub show_desktop_notifications: bool,
    pub blink_taskbar_icon: bool,
    pub play_audio_notifications: bool,
    pub minimize_instead_of_exit: bool,
    pub show_system_tray_icon: bool,
    pub minimize_to_system_tray: bool,
    // connection settings
    pub connect_automatically: bool,
    pub tor_config: TorConfig,
}

impl Settings {
    pub fn new(tor_config: TorConfig) -> Self {
        Self {
            start_only_single_instance: true,
            check_for_updates_automatically: true,
            language: Language::System,
            show_toolbar: true,
            button_style: ButtonStyle::Icons,
            show_desktop_notifications: false,
            blink_taskbar_icon: false,
            play_audio_notifications: false,
            minimize_instead_of_exit: false,
            show_system_tray_icon: false,
            minimize_to_system_tray: false,
            connect_automatically: false,
            tor_config,
        }
    }

    #[cfg(feature = "bundled-tor")]
    pub fn default_bundled_tor() -> Self {
        Self::new(TorConfig::default_bundled_tor())
    }

    #[cfg(feature = "external-tor")]
    pub fn default_external_tor() -> Self {
        Self::new(TorConfig::default_external_tor())
    }

    #[cfg(feature = "arti-client")]
    pub fn default_arti_client() -> Self {
        Self::new(TorConfig::default_arti_client())
    }
}

impl FromStr for Settings {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: Settings = serde_json::from_str(s)?;
        Ok(result)
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json: String = serde_json::to_string_pretty(self)
            .expect("Settins should always be Serializable to JSON");
        write!(f, "{json}")
    }
}

impl TryFrom<SettingsRaw> for Settings {
    type Error = crate::Error;

    fn try_from(value: SettingsRaw) -> Result<Self, Self::Error> {
        let start_only_single_instance = value.start_only_single_instance;
        let check_for_updates_automatically = value.check_for_updates_automatically;
        let language = value.language;
        let show_toolbar = value.show_toolbar;
        let button_style = value.button_style;
        let show_desktop_notifications = value.show_desktop_notifications;
        let blink_taskbar_icon = value.blink_taskbar_icon;
        let play_audio_notifications = value.play_audio_notifications;
        let minimize_instead_of_exit = value.minimize_instead_of_exit;
        let show_system_tray_icon = value.show_system_tray_icon;
        let minimize_to_system_tray = value.minimize_to_system_tray;
        let connect_automatically = value.connect_automatically;
        let tor_config = TorConfig::try_from(value.tor_config)?;

        Ok(Self {
            start_only_single_instance,
            check_for_updates_automatically,
            language,
            show_toolbar,
            button_style,
            show_desktop_notifications,
            blink_taskbar_icon,
            play_audio_notifications,
            minimize_instead_of_exit,
            show_system_tray_icon,
            minimize_to_system_tray,
            connect_automatically,
            tor_config,
        })
    }
}

impl From<v3::settings::Settings> for Settings {
    fn from(value: v3::settings::Settings) -> Self {
        #[cfg(feature = "bundled-tor")]
        let tor_config = TorConfig::BundledTor {
            bridge_config: value.bridge_config.clone(),
            proxy_config: value.proxy_config.clone(),
            firewall_config: value.firewall_config.clone(),
        };
        Self {
            language: match value.language {
                v3::settings::Language::SystemDefault => Language::System,
                v3::settings::Language::German => Language::German,
                v3::settings::Language::English => Language::English,
                v3::settings::Language::Spanish => Language::Spanish,
                v3::settings::Language::Dutch => Language::Dutch,
                _ => Language::System,
            },
            play_audio_notifications: value.play_audio_notification,
            ..Self::new(tor_config)
        }
    }
}

impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let settings_raw =
            SettingsRaw::deserialize(deserializer).map_err(serde::de::Error::custom)?;

        Settings::try_from(settings_raw).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Settings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let settings_raw = SettingsRaw::from(self);
        settings_raw.serialize(serializer)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Version {
    #[default]
    #[serde(rename = "4.0.0")]
    V4_0_0,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum Language {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "nl")]
    Dutch,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum ButtonStyle {
    #[default]
    #[serde(rename = "icons")]
    Icons,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "icons_and_text")]
    IconsAndText,
    #[serde(rename = "icons_beside_text")]
    IconsBesideText,
}

#[derive(Deserialize, Serialize)]
enum TorConfigRaw {
    #[cfg(feature = "bundled-tor")]
    #[serde(rename = "bundled-tor")]
    BundledTor {
        bridge_config: BridgeConfigRaw,
        proxy_config: ProxyConfigRaw,
        firewall_config: FirewallConfigRaw,
    },
    #[cfg(feature = "external-tor")]
    #[serde(rename = "external-tor")]
    ExternalTor {
        // todo
    },
    #[cfg(feature = "arti-client")]
    #[serde(rename = "arti-client")]
    ArtiClient {
        // todo
    },
}

impl From<&TorConfig> for TorConfigRaw {
    fn from(value: &TorConfig) -> Self {
        match value {
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                bridge_config,
                proxy_config,
                firewall_config,
            } => {
                let bridge_config = match bridge_config {
                    None => BridgeConfigRaw::None,
                    Some(BridgeConfig::Custom(first, bridge_lines)) => {
                        let first = first.as_legacy_tor_setconf_value();
                        let mut bridge_strings: Vec<String> = bridge_lines
                            .iter()
                            .map(|bridge_line| bridge_line.as_legacy_tor_setconf_value())
                            .collect();
                        bridge_strings.insert(0, first);
                        BridgeConfigRaw::Custom(bridge_strings)
                    }
                    Some(BridgeConfig::BuiltIn(BuiltInBridge::Obfs4)) => {
                        BridgeConfigRaw::BuiltInObfs4
                    }
                    Some(BridgeConfig::BuiltIn(BuiltInBridge::Meek)) => {
                        BridgeConfigRaw::BuiltInMeek
                    }
                    Some(BridgeConfig::BuiltIn(BuiltInBridge::Snowflake)) => {
                        BridgeConfigRaw::BuiltInSnowflake
                    }
                };
                let proxy_config = match proxy_config {
                    None => ProxyConfigRaw::None,
                    Some(ProxyConfig::Socks4(config)) => {
                        let address = config.address();
                        let host = address.host();
                        let port = address.port();
                        ProxyConfigRaw::Socks4 { host, port }
                    }
                    Some(ProxyConfig::Socks5(config)) => {
                        let address = config.address();
                        let host = address.host();
                        let port = address.port();
                        let username = config.username().clone();
                        let password = config.password().clone();
                        ProxyConfigRaw::Socks5 {
                            host,
                            port,
                            username,
                            password,
                        }
                    }
                    Some(ProxyConfig::Https(config)) => {
                        let address = config.address();
                        let host = address.host();
                        let port = address.port();
                        let username = config.username().clone();
                        let password = config.password().clone();
                        ProxyConfigRaw::Https {
                            host,
                            port,
                            username,
                            password,
                        }
                    }
                };
                let firewall_config = match firewall_config {
                    None => FirewallConfigRaw::None,
                    Some(firewall_config) => {
                        FirewallConfigRaw::AllowedPorts(firewall_config.allowed_ports().clone())
                    }
                };
                TorConfigRaw::BundledTor {
                    bridge_config,
                    proxy_config,
                    firewall_config,
                }
            }
            #[cfg(feature = "external-tor")]
            TorConfig::ExternalTor => TorConfigRaw::ExternalTor {},
            #[cfg(feature = "arti-client")]
            TorConfig::ArtiClient => TorConfigRaw::ArtiClient {},
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TorConfig {
    #[cfg(feature = "bundled-tor")]
    BundledTor {
        bridge_config: Option<BridgeConfig>,
        proxy_config: Option<ProxyConfig>,
        firewall_config: Option<FirewallConfig>,
    },
    #[cfg(feature = "external-tor")]
    ExternalTor,
    #[cfg(feature = "arti-client")]
    ArtiClient,
}

impl TorConfig {
    #[cfg(feature = "bundled-tor")]
    pub fn default_bundled_tor() -> Self {
        TorConfig::BundledTor {
            bridge_config: None,
            proxy_config: None,
            firewall_config: None,
        }
    }
    #[cfg(feature = "external-tor")]
    pub fn default_external_tor() -> Self {
        TorConfig::ExternalTor
    }
    #[cfg(feature = "arti-client")]
    pub fn default_arti_client() -> Self {
        TorConfig::ArtiClient
    }
}

impl TryFrom<TorConfigRaw> for TorConfig {
    type Error = crate::Error;

    fn try_from(value: TorConfigRaw) -> Result<Self, Self::Error> {
        let tor_config = match value {
            #[cfg(feature = "bundled-tor")]
            TorConfigRaw::BundledTor {
                bridge_config,
                proxy_config,
                firewall_config,
            } => {
                let bridge_config = match bridge_config {
                    BridgeConfigRaw::None => None,
                    BridgeConfigRaw::Custom(bridge_strings) => {
                        if bridge_strings.is_empty() {
                            return Err(Self::Error::ConversionFailed(
                                "custom bridge_config must contain at least one bridge line",
                            ));
                        } else {
                            let mut bridge_lines: Vec<BridgeLine> =
                                Vec::with_capacity(bridge_strings.len());
                            for bridge_string in bridge_strings {
                                let bridge_line = BridgeLine::from_str(bridge_string.as_ref())?;
                                bridge_lines.push(bridge_line);
                            }
                            let first = bridge_lines.remove(0);
                            Some(BridgeConfig::Custom(first, bridge_lines))
                        }
                    }
                    BridgeConfigRaw::BuiltInObfs4 => {
                        Some(BridgeConfig::BuiltIn(BuiltInBridge::Obfs4))
                    }
                    BridgeConfigRaw::BuiltInMeek => {
                        Some(BridgeConfig::BuiltIn(BuiltInBridge::Meek))
                    }
                    BridgeConfigRaw::BuiltInSnowflake => {
                        Some(BridgeConfig::BuiltIn(BuiltInBridge::Snowflake))
                    }
                };
                let proxy_config = match proxy_config {
                    ProxyConfigRaw::None => None,
                    ProxyConfigRaw::Socks4 { host, port } => {
                        let address = TargetAddr::try_from((host, port))?;
                        let config = Socks4ProxyConfig::new(address)?;
                        Some(ProxyConfig::from(config))
                    }
                    ProxyConfigRaw::Socks5 {
                        host,
                        port,
                        username,
                        password,
                    } => {
                        let address = TargetAddr::try_from((host, port))?;
                        let config = Socks5ProxyConfig::new(address, username, password)?;
                        Some(ProxyConfig::from(config))
                    }
                    ProxyConfigRaw::Https {
                        host,
                        port,
                        username,
                        password,
                    } => {
                        let address = TargetAddr::try_from((host, port))?;
                        let config = HttpsProxyConfig::new(address, username, password)?;
                        Some(ProxyConfig::from(config))
                    }
                };
                let firewall_config = match firewall_config {
                    FirewallConfigRaw::None => None,
                    FirewallConfigRaw::AllowedPorts(allowed_ports_list) => {
                        Some(FirewallConfig::try_from(allowed_ports_list)?)
                    }
                };
                TorConfig::BundledTor {
                    bridge_config,
                    proxy_config,
                    firewall_config,
                }
            }
            #[cfg(feature = "external-tor")]
            TorConfigRaw::ExternalTor {} => TorConfig::ExternalTor,
            #[cfg(feature = "arti-client")]
            TorConfigRaw::ArtiClient {} => TorConfig::ArtiClient,
        };
        Ok(tor_config)
    }
}

#[derive(Default, Deserialize, Serialize)]
enum BridgeConfigRaw {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "built-in-obfs4")]
    BuiltInObfs4,
    #[serde(rename = "built-in-meek")]
    BuiltInMeek,
    #[serde(rename = "built-in-snowflake")]
    BuiltInSnowflake,
    #[serde(rename = "custom")]
    Custom(Vec<String>),
}

#[derive(Default, Deserialize, Serialize)]
enum ProxyConfigRaw {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "socks4")]
    Socks4 { host: String, port: u16 },
    #[serde(rename = "socks5")]
    Socks5 {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    },
    #[serde(rename = "https")]
    Https {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    },
}

#[derive(Default, Deserialize, Serialize)]
enum FirewallConfigRaw {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "allowed_ports")]
    AllowedPorts(Vec<u16>),
}
