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

// todo: gate InProcessArti on feature gate
enum class TorBackend {
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
    Snowflake,
    Meek
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
// Conversation-related enums
//

enum class ContactGroup {
    Connected = 0,
    Disconnected,
    Requesting,
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
