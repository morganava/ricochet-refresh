# Ricochet-Refresh v4.0.X Configuration Specification

#### Morgan <[morgan@blueprintforfreespeech.net](mailto:morgan@blueprintforfreespeech.net)>

---

## Introduction

Ricochet-Refresh is a peer-to-peer instant messaging application built on Tor onion-services. This document describes the format of the `settings.json` configuration file used in the Ricochet-Refresh 3.0.X series.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED",  "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119[^1].

## Overview

This document defines the JSON structure, types, defaults, validation rules, and semantics for the settings file used by Ricochet-Refresh v4.0.X series. Implementations MUST parse and validate against the rules below.

By default, the config file is located in the following locations based on host operating system:

- **Windows** `%USERPROFILE%/AppData/Local/ricochet-refresh/settings.json`
- **macOS** `~/Library/Preferences/ricochet-refresh/settings.json`
- **Linux** `~/.config/ricochet-refresh/settings.json`

## File format

- The file MUST be UTF-8 encoded JSON[^2].
- The top-level value MUST be a JSON object.
- Unless otherwise specified, any unknown fields MUST be ignored when reading but MAY not be discarded when writing.

## Top-level object

Top-level object MAY contain the following members:

- `version` : string
    - Purpose: identifies the configuration file format version
    - Presence: REQUIRED.
    - Allowed values: "4.0.0"
    - Implementations MUST reject configurations with unsupported version values.

### General Settings

- `start_only_single_instance` : boolean
    - Purpose: whether to allow only a single instance of the application to run
    - Presence: OPTIONAL.
    - If omitted, defaults to true.

- `check_for_updates_automatically` : boolean
    - Purpose: whether to check for application updates automatically on launch
    - Presence: OPTIONAL.
    - If omitted, defaults to true.

### Interface Settings

- `language` : string
    - Purpose: user interface language preference
    - Presence: OPTIONAL.
    - Allowed values: "system", "ar", "de", "en", "es", "nl"
    - If omitted, defaults to "system" which is interpreted as the system default language.
    - Implementations MUST reject unknown language codes.

- `show_toolbar` : boolean
    - Purpose: whether to display the application toolbar
    - Presence: OPTIONAL.
    - If omitted, defaults to true.

- `show_desktop_notifications` : boolean
    - Purpose: whether to show desktop notifications for incoming messages
    - Presence: OPTIONAL.
    - If omitted, defaults to false.

- `blink_taskbar_icon` : boolean
    - Purpose: whether to blink the taskbar icon when receiving messages
    - Presence: OPTIONAL.
    - If omitted, defaults to false.

- `play_audio_notifications` : boolean
    - Purpose: whether to play audio notifications for incoming messages
    - Presence: OPTIONAL.
    - If omitted, defaults to false.

- `minimize_instead_of_exit` : boolean
    - Purpose: whether closing the window minimizes instead of exiting the application
    - Presence: OPTIONAL.
    - If omitted, defaults to false.

- `show_system_tray_icon` : boolean
    - Purpose: whether to display a system tray icon
    - Presence: OPTIONAL.
    - If omitted, defaults to false.

- `minimize_to_system_tray` : boolean
    - Purpose: whether to minimize to the system tray instead of the taskbar
    - Presence: OPTIONAL.
    - If omitted, defaults to false.

### Connection Settings

- `tor_backend` : string
    - Purpose: specifies which Tor backend implementation to use
    - Presence: OPTIONAL.
    - Allowed values: "bundled-tor", "system-tor", "arti-client"
    - If omitted, defaults to "bundled-tor".
    - Implementations MUST reject unknown tor backend values.

- `connect_automatically` : boolean
    - Purpose: whether to connect to Tor automatically on startup
    - Presence: OPTIONAL.
    - If omitted, defaults to false.

- `bridge_config` : string or object
    - Purpose: configuration for connecting to the Tor network through bridges
    - Presence: OPTIONAL.
    - Allowed values:
        - `"none"` (string) : No bridge is configured.
        - `"built-in-obfs4"` (string) : Use built-in obfs4 bridges.
        - `"built-in-meek"` (string) : Use built-in meek bridges.
        - `"built-in-snowflake"` (string) : Use built-in Snowflake bridges.
        - `{ "custom": [bridge_string, ...] }` (object) : Use custom bridge strings.
            - Format: Each bridge_string MUST be a valid bridge string[^3].
            - MUST NOT be an empty array.
            - Duplicates SHOULD be ignored by consumers.
    - If omitted, defaults to `"none"`.
    - Implementations MUST reject unknown bridge types.

- `proxy_config` : string or object
    - Purpose: configuration for connecting through a proxy to the Tor network
    - Presence: OPTIONAL.
    - Allowed values:
        - `"none"` (string) : No proxy is configured.
        - `{ "socks4": { "host": string, "port": number } }` (object) : Use SOCKS4 proxy.
            - `host` : MUST be an IP address or DNS name.
            - `port` : MUST be an integer in range 1..65535.
        - `{ "socks5": { "host": string, "port": number, "username": string?, "password": string? } }` (object) : Use SOCKS5 proxy.
            - `host` : MUST be an IP address or DNS name.
            - `port` : MUST be an integer in range 1..65535.
            - `username` : OPTIONAL. If omitted or null, no username is used.
            - `password` : OPTIONAL. If omitted or null, no password is used.
        - `{ "https": { "host": string, "port": number, "username": string?, "password": string? } }` (object) : Use HTTPS proxy.
            - `host` : MUST be an IP address or DNS name.
            - `port` : MUST be an integer in range 1..65535.
            - `username` : OPTIONAL. If omitted or null, no username is used.
            - `password` : OPTIONAL. If omitted or null, no password is used.
    - If omitted, defaults to `"none"`.
    - Implementations MUST reject unknown proxy types.

- `firewall_config` : string or object
    - Purpose: configuration for restricting which ports the application can connect to
    - Presence: OPTIONAL.
    - Format: This field uses Serde enum representation. The value MUST be one of:
        - `"none"` (string) : No firewall restrictions are configured.
        - `{ "allowed_ports": [port, ...] }` (object) : Restrict to specified ports.
            - Each port MUST be an integer in range 1..65535.
            - MUST NOT be an empty array.
            - MUST NOT contain duplicate entries.
    - If omitted, defaults to `"none"`.

---

[^1]: RFC 2119 [https://www.rfc-editor.org/rfc/rfc2119](https://www.rfc-editor.org/rfc/rfc2119)

[^2]: JSON specification [https://json.org](https://json.org)

[^3]: Bridge line format: [https://tpo.pages.torproject.net/core/doc/rust/tor_guardmgr/bridge/struct.BridgeConfig.html#string-representation](https://tpo.pages.torproject.net/core/doc/rust/tor_guardmgr/bridge/struct.BridgeConfig.html#string-representation)
