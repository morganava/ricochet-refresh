#pragma once

//
// Locale-related enums
//

enum class LayoutDirection {
    LeftToRight,
    RightToLeft,
};

enum class Ordering {
    Less,
    Equal,
    Greater
};

//
// Bootstrap-related enums
//

enum class ConnectionStatus {
    Offline,
    Connecting,
    Online,
};

//
// Settings-related enums
//

enum class TorBackend {
    None,
    BundledLegacyTor,
    ExternalLegacyTor,
    InProcessArti,
};

enum class BridgeType {
    Builtin,
    Custom,
};

enum class BuiltinBridge {
    Obfs4,
    Meek,
    Snowflake,
};

enum class ProxyType {
    SOCKS4 = 0,
    SOCKS5,
    HTTPS,
};

enum class Language {
    System = 0,
    Arabic, // ar
    German, // de
    English, // en
    Spanish, // es
    Dutch, // nl
};

enum class ButtonStyle {
    Icons = 0,
    Text,
    IconsAndText,
    IconsBesideText,
};

enum class Settings {
    General = 0,
    Interface,
    Connection,
};

//
// New Profile-related enums
//

enum class NewProfile {
    Generate = 0,
    ImportLegacy,
};

enum class ProfileCreationStep {
    SetPaths = 0,
    CreateCredentials,
    Review,
    Count
};

//
// Conversation-related enums
//

enum class ContactGroup {
    Connected = 0,
    Disconnected,
    Requesting,
    Rejected,
    Blocked,
    Count,
};

enum class Visibility {
    Visible = 0,
    Restricted,
    Hidden,
    Offline,
    Count,
};

enum class MessageType {
    Text,
};
