// std
use std::str::FromStr;

// extern
use tor_interface::censorship_circumvention::BridgeLine;
use tor_interface::proxy::*;
use tor_interface::tor_provider::TargetAddr;

// internal
use rico_settings::common::*;
use rico_settings::v4::settings::*;

#[test]
fn test_round_trip() -> anyhow::Result<()> {
    let mut settings_vec: Vec<Settings> = Default::default();
    #[cfg(feature = "bundled-tor")]
    settings_vec.append(&mut vec![
        Settings::new(TorConfig::default_bundled_tor()),
        Settings{start_only_single_instance: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{start_only_single_instance: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{check_for_updates_automatically: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{check_for_updates_automatically: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{language: Language::System, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{language: Language::Arabic, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{language: Language::German, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{language: Language::English, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{language: Language::Spanish, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{language: Language::Dutch, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{show_toolbar: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{show_toolbar: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{blink_taskbar_icon: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{blink_taskbar_icon: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{play_audio_notifications: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{play_audio_notifications: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{minimize_instead_of_exit: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{minimize_instead_of_exit: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{show_system_tray_icon: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{show_system_tray_icon: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{minimize_to_system_tray: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{minimize_to_system_tray: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{connect_automatically: true, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings{connect_automatically: false, ..Settings::new(TorConfig::default_bundled_tor())},
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: None, firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: Some(BuiltInBridge::Obfs4.into()) ,proxy_config: None, firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: Some(BuiltInBridge::Meek.into()) ,proxy_config: None, firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: Some(BuiltInBridge::Snowflake.into()) ,proxy_config: None, firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: Some(vec![BridgeLine::from_str("meek_lite 192.0.2.20:80 url=https://1603026938.rsc.cdn77.org front=www.phpmyadmin.net utls=HelloRandomizedALPN")?].try_into().unwrap()), proxy_config: None, firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks4ProxyConfig::new(TargetAddr::from_str("127.0.0.1:4")?)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks4ProxyConfig::new(TargetAddr::from_str("example.com:4")?)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks5ProxyConfig::new(TargetAddr::from_str("127.0.0.1:5")?, None, None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks5ProxyConfig::new(TargetAddr::from_str("example.com:5")?, None, None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks5ProxyConfig::new(TargetAddr::from_str("127.0.0.1:5")?, Some("alice".to_string()), None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks5ProxyConfig::new(TargetAddr::from_str("127.0.0.1:5")?, None, Some("123456".to_string()))?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks5ProxyConfig::new(TargetAddr::from_str("127.0.0.1:5")?, Some("alice".to_string()), None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(Socks5ProxyConfig::new(TargetAddr::from_str("127.0.0.1:5")?, Some("alice".to_string()), Some("123456".to_string()))?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(HttpsProxyConfig::new(TargetAddr::from_str("127.0.0.1:443")?, None, None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(HttpsProxyConfig::new(TargetAddr::from_str("example.com:443")?, None, None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(HttpsProxyConfig::new(TargetAddr::from_str("127.0.0.1:443")?, Some("alice".to_string()), None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(HttpsProxyConfig::new(TargetAddr::from_str("127.0.0.1:443")?, None, Some("123456".to_string()))?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(HttpsProxyConfig::new(TargetAddr::from_str("127.0.0.1:443")?, Some("alice".to_string()), None)?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: Some(HttpsProxyConfig::new(TargetAddr::from_str("127.0.0.1:443")?, Some("alice".to_string()), Some("123456".to_string()))?.into()), firewall_config: None}),
        Settings::new(TorConfig::BundledTor{bridge_config: None, proxy_config: None, firewall_config: Some(FirewallConfig::try_from(vec![80, 443, 8080]).unwrap())}),
    ]);

    #[cfg(feature = "external-tor")]
    settings_vec.append(&mut vec![Settings::new(TorConfig::default_external_tor())]);
    #[cfg(feature = "arti-client")]
    settings_vec.append(&mut vec![Settings::new(TorConfig::default_arti_client())]);

    // verify settings objects can round-trip successfully
    for settings in settings_vec {
        let json = settings.to_string();
        println!("---\n{json}");
        assert_eq!(settings, Settings::from_str(json.as_ref()).unwrap());
    }

    Ok(())
}
