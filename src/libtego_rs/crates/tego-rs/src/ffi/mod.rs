pub mod context;
pub mod ed25519_private_key;
pub mod error;
pub mod handle;
#[cfg(feature = "logging")]
pub mod logger;
pub mod object_map;
pub mod profile;
pub mod settings;
pub mod string;
pub mod tor_config;
pub mod user_id;
pub mod v3_onion_service_id;

// standard
use std::ffi::{c_char, c_int, CString};

// extern
use anyhow::{bail, Result};
use rico_profile::v4::profile::{Profile, UserProfile};
use rico_settings::common::{BridgeConfig, BuiltInBridge, FirewallConfig};
use rico_settings::v4::settings::{ButtonStyle, Language, TorConfig};
use tor_interface::censorship_circumvention::PluggableTransportConfig;
use tor_interface::censorship_circumvention::*;
use tor_interface::legacy_tor_client::LegacyTorClientConfig;
use tor_interface::proxy::ProxyConfig;
use tor_interface::tor_crypto::{Ed25519PrivateKey, V3OnionServiceId};

// internal
use crate::context::Context;
use crate::error::{translate_failures, Error};
use crate::ffi;
use crate::ffi::handle::*;
use crate::macros::*;
use crate::settings::SettingsStore;

// tags for handles representing each of our FFI types
impl_handle_tags!(
    TEGO_ERROR_TAG,
    TEGO_STRING_TAG,
    TEGO_CONTEXT_TAG,
    TEGO_SETTINGS_TAG,
    TEGO_PROFILE_TAG,
    TEGO_ED25519_PRIVATE_KEY_TAG,
    TEGO_V3_ONION_SERVICE_ID_TAG,
    TEGO_USER_ID_TAG,
    TEGO_TOR_CONFIG_TAG,
    TEGO_BRIDGE_CONFIG_TAG,
    TEGO_FIREWALL_CONFIG_TAG,
    TEGO_PROXY_CONFIG_TAG,
    _TEGO_MAX_TAG,
);
// the number of bits requird to store the various TEGO_.*_TAG constants
pub(crate) const TEGO_TAG_BITS: usize = (_TEGO_MAX_TAG - 1usize).ilog2() as usize + 1usize;

impl_object_map!(tego_error, Error);
impl_object_map!(tego_string, CString);
impl_object_map!(tego_context, Context);
impl_object_map!(tego_settings, SettingsStore);
impl_object_map!(tego_profile, Profile);
impl_object_map!(tego_ed25519_private_key, Ed25519PrivateKey);
impl_object_map!(tego_v3_onion_service_id, V3OnionServiceId);
impl_object_map!(tego_user_id, V3OnionServiceId);
impl_object_map!(tego_tor_config, TorConfig);
impl_object_map!(tego_bridge_config, BridgeConfig);
impl_object_map!(tego_firewall_config, FirewallConfig);
impl_object_map!(tego_proxy_config, ProxyConfig);

pub const TEGO_TRUE: i32 = 1;
pub const TEGO_FALSE: i32 = 0;

/// number of bytes in an ed25519 signature
pub const TEGO_ED25519_SIGNATURE_SIZE: usize = 64usize;
/// length of a valid v3 service id string not including null terminator
pub const TEGO_V3_ONION_SERVICE_ID_LENGTH: usize = 56usize;
/// length of a v3 service id string including null terminator
pub const TEGO_V3_ONION_SERVICE_ID_SIZE: usize = TEGO_V3_ONION_SERVICE_ID_LENGTH + 1usize;
/// length of the ed25519 keyblob string not including null terminator
pub const TEGO_ED25519_KEYBLOB_LENGTH: usize = 99usize;
/// length of an ed25519 keyblob string including null terminator
pub const TEGO_ED25519_KEYBLOB_SIZE: usize = TEGO_ED25519_KEYBLOB_LENGTH + 1usize;

pub type tego_bool = i32;
pub struct tego_error;
pub struct tego_string;
pub struct tego_context;
pub struct tego_settings;
pub struct tego_profile;

#[repr(C)]
pub enum tego_language {
    tego_language_system,
    tego_language_ar,
    tego_language_de,
    tego_language_en,
    tego_language_es,
    tego_language_nl,
}

impl From<tego_language> for Language {
    fn from(value: tego_language) -> Self {
        match value {
            tego_language::tego_language_system => Language::System,
            tego_language::tego_language_ar => Language::Arabic,
            tego_language::tego_language_de => Language::German,
            tego_language::tego_language_en => Language::English,
            tego_language::tego_language_es => Language::Spanish,
            tego_language::tego_language_nl => Language::Dutch,
        }
    }
}

impl From<Language> for tego_language {
    fn from(value: Language) -> Self {
        match value {
            Language::System => tego_language::tego_language_system,
            Language::Arabic => tego_language::tego_language_ar,
            Language::German => tego_language::tego_language_de,
            Language::English => tego_language::tego_language_en,
            Language::Spanish => tego_language::tego_language_es,
            Language::Dutch => tego_language::tego_language_nl,
        }
    }
}

#[repr(C)]
pub enum tego_button_style {
    tego_button_style_icons,
    tego_button_style_text,
    tego_button_style_icons_and_text,
    tego_button_style_icons_beside_text,
}

impl From<tego_button_style> for ButtonStyle {
    fn from(value: tego_button_style) -> Self {
        match value {
            tego_button_style::tego_button_style_icons => ButtonStyle::Icons,
            tego_button_style::tego_button_style_text => ButtonStyle::Text,
            tego_button_style::tego_button_style_icons_and_text => ButtonStyle::IconsAndText,
            tego_button_style::tego_button_style_icons_beside_text => ButtonStyle::IconsBesideText,
        }
    }
}

impl From<ButtonStyle> for tego_button_style {
    fn from(value: ButtonStyle) -> Self {
        match value {
            ButtonStyle::Icons => tego_button_style::tego_button_style_icons,
            ButtonStyle::Text => tego_button_style::tego_button_style_text,
            ButtonStyle::IconsAndText => tego_button_style::tego_button_style_icons_and_text,
            ButtonStyle::IconsBesideText => tego_button_style::tego_button_style_icons_beside_text,
        }
    }
}

pub struct tego_tor_config;
#[repr(C)]
pub enum tego_tor_config_type {
    #[cfg(feature = "bundled-tor")]
    tego_tor_config_type_bundled_tor,
    #[cfg(feature = "external-tor")]
    tego_tor_config_type_external_tor,
    #[cfg(feature = "arti-client")]
    tego_tor_config_type_arti_client,
}

pub struct tego_bridge_config;
#[repr(C)]
pub enum tego_bridge_config_type {
    tego_bridge_config_type_builtin,
    tego_bridge_config_type_custom,
}

#[repr(C)]
pub enum tego_bridge_builtin {
    tego_bridge_builtin_obfs4,
    tego_bridge_builtin_meek,
    tego_bridge_builtin_snowflake,
}

impl From<tego_bridge_builtin> for BuiltInBridge {
    fn from(value: tego_bridge_builtin) -> Self {
        match value {
            tego_bridge_builtin::tego_bridge_builtin_obfs4 => BuiltInBridge::Obfs4,
            tego_bridge_builtin::tego_bridge_builtin_meek => BuiltInBridge::Meek,
            tego_bridge_builtin::tego_bridge_builtin_snowflake => BuiltInBridge::Snowflake,
        }
    }
}

impl From<BuiltInBridge> for tego_bridge_builtin {
    fn from(value: BuiltInBridge) -> Self {
        match value {
            BuiltInBridge::Obfs4 => tego_bridge_builtin::tego_bridge_builtin_obfs4,
            BuiltInBridge::Meek => tego_bridge_builtin::tego_bridge_builtin_meek,
            BuiltInBridge::Snowflake => tego_bridge_builtin::tego_bridge_builtin_snowflake,
        }
    }
}

pub struct tego_firewall_config;
pub struct tego_proxy_config;
#[repr(C)]
pub enum tego_proxy_type {
    tego_proxy_type_socks4,
    tego_proxy_type_socks5,
    tego_proxy_type_https,
}
pub struct tego_ed25519_private_key;
pub struct tego_v3_onion_service_id;
pub struct tego_user_id;

/// State of the host user's onion service
#[repr(C)]
pub enum tego_host_onion_service_state {
    tego_host_onion_service_state_none,
    tego_host_onion_service_state_service_added,
    tego_host_onion_service_state_service_published,
}

/// TODO: figure out which statuses we need later
#[repr(C)]
pub enum tego_user_status {
    tego_user_status_none,
    tego_user_status_online,
    tego_user_status_offline,
}

/// enum for user type
#[repr(C)]
#[derive(Clone, Copy)]
pub enum tego_user_type {
    /// the host user
    tego_user_type_host,
    /// in host's contact list
    tego_user_type_allowed,
    /// users who have added host but the host has not replied yet
    // todo: remove requesting type
    tego_user_type_requesting,
    /// users who have added host but the host has blocked
    tego_user_type_blocked,
    /// users the host has added but who have not replied yet
    tego_user_type_pending,
    /// user the host has added but who have replied with rejection
    tego_user_type_rejected,
}

pub struct tego_tor_daemon_config;
pub struct tego_pluggable_transport_config;

#[repr(C)]
pub enum tego_tor_network_status {
    tego_tor_network_status_unknown,
    tego_tor_network_status_offline,
    tego_tor_network_status_ready,
}

#[repr(C)]
pub enum tego_tor_bootstrap_tag {
    tego_tor_bootstrap_tag_invalid = -1,
    tego_tor_bootstrap_tag_starting,
    tego_tor_bootstrap_tag_conn_pt,
    tego_tor_bootstrap_tag_conn_done_pt,
    tego_tor_bootstrap_tag_conn_proxy,
    tego_tor_bootstrap_tag_conn_done_proxy,
    tego_tor_bootstrap_tag_conn,
    tego_tor_bootstrap_tag_conn_done,
    tego_tor_bootstrap_tag_handshake,
    tego_tor_bootstrap_tag_handshake_done,
    tego_tor_bootstrap_tag_onehop_create,
    tego_tor_bootstrap_tag_requesting_status,
    tego_tor_bootstrap_tag_loading_status,
    tego_tor_bootstrap_tag_loading_keys,
    tego_tor_bootstrap_tag_requesting_descriptors,
    tego_tor_bootstrap_tag_loading_descriptors,
    tego_tor_bootstrap_tag_enough_dirinfo,
    tego_tor_bootstrap_tag_ap_conn_pt_summary,
    tego_tor_bootstrap_tag_ap_conn_done_pt,
    tego_tor_bootstrap_tag_ap_conn_proxy,
    tego_tor_bootstrap_tag_ap_conn_done_proxy,
    tego_tor_bootstrap_tag_ap_conn,
    tego_tor_bootstrap_tag_ap_conn_done,
    tego_tor_bootstrap_tag_ap_handshake,
    tego_tor_bootstrap_tag_ap_handshake_done,
    tego_tor_bootstrap_tag_circuit_create,
    tego_tor_bootstrap_tag_done,

    tego_tor_bootstrap_tag_count,
}

impl From<&str> for tego_tor_bootstrap_tag {
    fn from(value: &str) -> Self {
        use tego_tor_bootstrap_tag::*;
        match value {
            "starting" => tego_tor_bootstrap_tag_starting,
            "conn_pt" => tego_tor_bootstrap_tag_conn_pt,
            "conn_done_pt" => tego_tor_bootstrap_tag_conn_done_pt,
            "conn_proxy" => tego_tor_bootstrap_tag_conn_proxy,
            "conn_done_proxy" => tego_tor_bootstrap_tag_conn_done_proxy,
            "conn" => tego_tor_bootstrap_tag_conn,
            "conn_done" => tego_tor_bootstrap_tag_conn_done,
            "handshake" => tego_tor_bootstrap_tag_handshake,
            "handshake_done" => tego_tor_bootstrap_tag_handshake_done,
            "onehop_create" => tego_tor_bootstrap_tag_onehop_create,
            "requesting_status" => tego_tor_bootstrap_tag_requesting_status,
            "loading_status" => tego_tor_bootstrap_tag_loading_status,
            "loading_keys" => tego_tor_bootstrap_tag_loading_keys,
            "requesting_descriptors" => tego_tor_bootstrap_tag_requesting_descriptors,
            "loading_descriptors" => tego_tor_bootstrap_tag_loading_descriptors,
            "enough_dirinfo" => tego_tor_bootstrap_tag_enough_dirinfo,
            "ap_conn_pt" => tego_tor_bootstrap_tag_ap_conn_pt_summary,
            "ap_conn_done_pt" => tego_tor_bootstrap_tag_ap_conn_done_pt,
            "ap_conn_proxy" => tego_tor_bootstrap_tag_ap_conn_proxy,
            "ap_conn_done_proxy" => tego_tor_bootstrap_tag_ap_conn_done_proxy,
            "ap_conn" => tego_tor_bootstrap_tag_ap_conn,
            "ap_conn_done" => tego_tor_bootstrap_tag_ap_conn_done,
            "ap_handshake" => tego_tor_bootstrap_tag_ap_handshake,
            "ap_handshake_done" => tego_tor_bootstrap_tag_ap_handshake_done,
            "circuit_create" => tego_tor_bootstrap_tag_circuit_create,
            "done" => tego_tor_bootstrap_tag_done,
            _ => tego_tor_bootstrap_tag_invalid,
        }
    }
}

/// Get the summary string associated with the given bootstrap tag
///
/// @param tag : the tag to get the summary of
/// @param error : filled on error
/// @return : utf8 null-terminated summary string, NULL on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_tor_bootstrap_tag_to_summary(
    tag: tego_tor_bootstrap_tag,
    error: *mut *mut tego_error,
) -> *const c_char {
    translate_failures(std::ptr::null(), error, || -> Result<*const c_char> {
        use tego_tor_bootstrap_tag::*;
        let summary = match tag {
            tego_tor_bootstrap_tag_starting => "Starting\0",
            tego_tor_bootstrap_tag_conn_pt => "Connecting to pluggable transport\0",
            tego_tor_bootstrap_tag_conn_done_pt => "Connected to pluggable transport\0",
            tego_tor_bootstrap_tag_conn_proxy => "Connecting to proxy\0",
            tego_tor_bootstrap_tag_conn_done_proxy => "Connected to proxy\0",
            tego_tor_bootstrap_tag_conn => "Connecting to a relay\0",
            tego_tor_bootstrap_tag_conn_done => "Connected to a relay\0",
            tego_tor_bootstrap_tag_handshake => "Handshaking with a relay\0",
            tego_tor_bootstrap_tag_handshake_done => "Handshake with a relay done\0",
            tego_tor_bootstrap_tag_onehop_create => {
                "Establishing an encrypted directory connection\0"
            }
            tego_tor_bootstrap_tag_requesting_status => "Asking for networkstatus consensus\0",
            tego_tor_bootstrap_tag_loading_status => "Loading networkstatus consensus\0",
            tego_tor_bootstrap_tag_loading_keys => "Loading authority key certs\0",
            tego_tor_bootstrap_tag_requesting_descriptors => "Asking for relay descriptors\0",
            tego_tor_bootstrap_tag_loading_descriptors => "Loading relay descriptors\0",
            tego_tor_bootstrap_tag_enough_dirinfo => {
                "Loaded enough directory info to build circuits\0"
            }
            tego_tor_bootstrap_tag_ap_conn_pt_summary => {
                "Connecting to pluggable transport to build circuits\0"
            }
            tego_tor_bootstrap_tag_ap_conn_done_pt => {
                "Connected to pluggable transport to build circuits\0"
            }
            tego_tor_bootstrap_tag_ap_conn_proxy => "Connecting to proxy to build circuits\0",
            tego_tor_bootstrap_tag_ap_conn_done_proxy => "Connected to proxy to build circuits\0",
            tego_tor_bootstrap_tag_ap_conn => "Connecting to a relay to build circuits\0",
            tego_tor_bootstrap_tag_ap_conn_done => "Connected to a relay to build circuits\0",
            tego_tor_bootstrap_tag_ap_handshake => {
                "Finishing handshake with a relay to build circuits\0"
            }
            tego_tor_bootstrap_tag_ap_handshake_done => {
                "Handshake finished with a relay to build circuits\0"
            }
            tego_tor_bootstrap_tag_circuit_create => "Establishing a Tor circuit\0",
            tego_tor_bootstrap_tag_done => "Done\0",
            _ => bail!("unknown tego_tor_bootstrap_tag: {}", tag as c_int),
        };
        Ok(summary.as_ptr() as *const c_char)
    })
}

/// milliseconds since 1970-01-01T00:00:00 utc.
pub type tego_time = u64;
/// unique (per user) message identifier
pub type tego_message_id = u64;
/// unique (per user) file transfer identifier
pub type tego_file_transfer_id = u64;
/// integer type for file size
pub type tego_file_size = u64;

#[repr(C)]
pub enum tego_file_transfer_response {
    /// proceed with a file transfer
    tego_file_transfer_response_accept,
    /// reject the file transfer
    tego_file_transfer_response_reject,
}

#[repr(C)]
pub enum tego_chat_acknowledge {
    /// allows the user to chat with us
    tego_chat_acknowledge_accept,
    // do not allow the user to chat with us
    tego_chat_acknowledge_reject,
    // do not allow and reject all future requests
    tego_chat_acknowledge_block,
}

//
// Callbacks for frontend to respond to events
// Provides no guarantees on what thread they are running on or thread safety
// All parameters (such as tego_error*) are automatically destroyed after user
//  callback is invoked, so duplicate/marshall data as necessary
//

/// Callback fired when the tor daemon's network status changes
///
/// @param context : the current tego context
/// @param status : the new network status
pub type tego_tor_network_status_changed_callback =
    Option<extern "C" fn(context: *mut tego_context, status: tego_tor_network_status) -> ()>;

/// Callback fired when a tor provider has been initialized
///
/// @param context: the current tego context
/// @param tor_config_type: the type of config used to initialize the tor provider
/// @param version: the version string of the initialized tor provider (may be NULL in which case the version is unknown)
pub type tego_tor_provider_initialized_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        tor_config_type: tego_tor_config_type,
        version: *const tego_string,
    ) -> (),
>;

/// Callback fired when tor's bootstrap status changes
///
/// @param context : the current tego context
/// @param progress : the bootstrap progress percent
/// @param tag : the bootstrap tag
pub type tego_tor_bootstrap_status_changed_callback = Option<
    extern "C" fn(context: *mut tego_context, progress: i32, tag: tego_tor_bootstrap_tag) -> (),
>;

/// Callback fired when tor's bootstrap proces is completed
///
/// @param context : the current tego context
pub type tego_tor_bootstrap_complete_callback =
    Option<extern "C" fn(context: *mut tego_context) -> ()>;

/// Callback fired when a log entry is received from the tor daemon
///
/// @param context : the current tego context
/// @param message : a null-terminated log entry string
/// @param message_length : length of the message not including null-terminator
pub type tego_tor_log_received_callback =
    Option<extern "C" fn(context: *mut tego_context, line: *const tego_string) -> ()>;

/// Callback fired when the host user state changes
///
/// @param context : the current tego context
/// @param state : the current host user state
pub type tego_host_onion_service_state_changed_callback =
    Option<extern "C" fn(context: *mut tego_context, state: tego_host_onion_service_state) -> ()>;

/// Callback fired when the host receives a chat request from another user
///
/// @param context : the current tego context
/// @param sender : the user that wants to chat
/// @param message : null-terminated message string received from the requesting user
/// @param message_length : length of the message not including null-terminator
pub type tego_chat_request_received_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        sender: *const tego_user_id,
        message: *const c_char,
        message_length: usize,
    ) -> (),
>;

/// Callback fired when the host receives a response to their sent chat request
///
/// @param context : the current tego context
/// @param sender : the user responding to our chat request
/// @param accepted_request : TEGO_TRUE if request accepted, TEGO_FALSE if rejected
pub type tego_chat_request_response_received_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        sender: *const tego_user_id,
        accepted_request: tego_bool,
    ) -> (),
>;

/// Callback fired when the host receives a message from another user
///
/// @param context : the current tego context
/// @param sender : the user that sent host the message
/// @param timestamp : the time the message was sent
/// @param message_id : id of the message received
/// @param message : null-terminated message string
/// @param message_length : length of the message not including null-terminator
pub type tego_message_received_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        sender: *const tego_user_id,
        timestamp: tego_time,
        message_id: tego_message_id,
        message: *const c_char,
        message_length: usize,
    ) -> (),
>;

/// Callback fired when a chat message is received and acknowledge
/// by the recipient
///
/// @param context : the current tego context
/// @param user_id : the user the message was sent to
/// @param message_id : id of the message being acknowledged
/// @param message_acked : TEGO_TRUE if acknowledged, TEGO_FALSE if error
pub type tego_message_acknowledged_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        user_id: *const tego_user_id,
        message_id: tego_message_id,
        message_acked: tego_bool,
    ) -> (),
>;

/// Callback fired when a user wants to send recipient a file
///
/// @param context : the current tego context
/// @param sender : the user sending the request
/// @param id : id of the file transfer received
/// @param file_name : name of the file user wants to send
/// @param file_name_length : length of file_name not including the null-terminator
/// @param file_size : size of the file in bytes
/// @param file_hash : hash of the file
pub type tego_file_transfer_request_received_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        sender: *const tego_user_id,
        id: tego_file_transfer_id,
        file_name: *const c_char,
        file_name_length: usize,
        file_size: tego_file_size,
    ) -> (),
>;

/// Callback fired when a file transfer request message is received and
/// acknowledged by the recipient (not whether the recipient wishes to start
/// the file transfer)
///
/// @param context : the current tego cotext
/// @param receiver : the user acknowledging our request
/// @param id : the id of the file transfer that is being acknowledged
/// @param request_acked : TEGO_TRUE if acknowledged, TEGO_FALSE if error
pub type tego_file_transfer_request_acknowledged_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        receiver: *const tego_user_id,
        id: tego_file_transfer_id,
        request_acked: tego_bool,
    ) -> (),
>;

/// Callback fired when the user responds to an file transfer request
///
/// @param context : the current tego context
/// @param receiver : the user accepting or rejecting our request
/// @param id : the id of the file transfer that is being accepted
/// @param response : TEGO_TRUE if the recipients wants to recevie
///  our file, TEGO_FALSE otherwise
pub type tego_file_transfer_request_response_received_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        receiver: *const tego_user_id,
        id: tego_file_transfer_id,
        response: tego_file_transfer_response,
    ) -> (),
>;

#[repr(C)]
pub enum tego_file_transfer_direction {
    tego_file_transfer_direction_sending,
    tego_file_transfer_direction_receiving,
}

/// Callback fired when file transfer send or receive progress has changed
/// This callback is fired for both the sender and the receiver
///
/// @param context : the current tego context
/// @param user_id : the user sending/receiving the file
/// @param id : the file transfer associated with this callback
/// @param direction : the direction this file is going
/// @param bytes_complete : number of bytes sent/received
/// @param bytes_total : the total size of the file
pub type tego_file_transfer_progress_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        user_id: *const tego_user_id,
        id: tego_file_transfer_id,
        direction: tego_file_transfer_direction,
        bytes_complete: tego_file_size,
        bytes_total: tego_file_size,
    ) -> (),
>;

#[repr(C)]
pub enum tego_file_transfer_result {
    /// file transfer completed successfully
    tego_file_transfer_result_success,
    /// file transfer failed for unknown reason
    tego_file_transfer_result_failure,
    /// file transfer was cancelled by one of the participants after it had started
    tego_file_transfer_result_cancelled,
    /// file transfer request was rejected by the receiver
    tego_file_transfer_result_rejected,
    /// file transfer completed but final file's hash did not match the one advertised
    tego_file_transfer_result_bad_hash,
    /// file transfer failed due to connectivity problem
    tego_file_transfer_result_network_error,
    /// file transfer failed due to a file system error
    tego_file_transfer_result_filesystem_error,
}

/// Callback fired when a file transfer has completed
/// either successfully or in error
///
/// @param context : the current tego context
/// @param user_id : the user sending/receivintg the file
/// @param id : the file transfer associated with this callback
/// @param direction : the direction this file was going
/// @param result : how the transfer completed
pub type tego_file_transfer_complete_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        user_id: *const tego_user_id,
        id: tego_file_transfer_id,
        direction: tego_file_transfer_direction,
        result: tego_file_transfer_result,
    ) -> (),
>;

/// Callback fired when a user's status changes
///
/// @param context : the current tego context
/// @param user : the user whose status has changed
/// @param status: the user's new status
pub type tego_user_status_changed_callback = Option<
    extern "C" fn(
        context: *mut tego_context,
        user: *const tego_user_id,
        status: tego_user_status,
    ) -> (),
>;

//
// Setters for various callbacks
//

#[no_mangle]
pub extern "C" fn tego_context_set_tor_network_status_changed_callback(
    context: *mut tego_context,
    callback: tego_tor_network_status_changed_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_tor_network_status_changed, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_tor_provider_initialized_callback(
    context: *mut tego_context,
    callback: tego_tor_provider_initialized_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_tor_provider_initialized, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_tor_bootstrap_status_changed_callback(
    context: *mut tego_context,
    callback: tego_tor_bootstrap_status_changed_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_tor_bootstrap_status_changed, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_tor_bootstrap_complete_callback(
    context: *mut tego_context,
    callback: tego_tor_bootstrap_complete_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_tor_bootstrap_complete, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_tor_log_received_callback(
    context: *mut tego_context,
    callback: tego_tor_log_received_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_tor_log_received, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_host_onion_service_state_changed_callback(
    context: *mut tego_context,
    callback: tego_host_onion_service_state_changed_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(
        on_host_onion_service_state_changed,
        context,
        callback,
        error
    );
}

#[no_mangle]
pub extern "C" fn tego_context_set_chat_request_received_callback(
    context: *mut tego_context,
    callback: tego_chat_request_received_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_chat_request_received, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_chat_request_response_received_callback(
    context: *mut tego_context,
    callback: tego_chat_request_response_received_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_chat_request_response_received, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_message_received_callback(
    context: *mut tego_context,
    callback: tego_message_received_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_message_received, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_message_acknowledged_callback(
    context: *mut tego_context,
    callback: tego_message_acknowledged_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_message_acknowledged, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_file_transfer_request_received_callback(
    context: *mut tego_context,
    callback: tego_file_transfer_request_received_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_file_transfer_request_received, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_file_transfer_request_acknowledged_callback(
    context: *mut tego_context,
    callback: tego_file_transfer_request_acknowledged_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(
        on_file_transfer_request_acknowledged,
        context,
        callback,
        error
    );
}

#[no_mangle]
pub extern "C" fn tego_context_set_file_transfer_request_response_received_callback(
    context: *mut tego_context,
    callback: tego_file_transfer_request_response_received_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(
        on_file_transfer_request_response_received,
        context,
        callback,
        error
    );
}

#[no_mangle]
pub extern "C" fn tego_context_set_file_transfer_progress_callback(
    context: *mut tego_context,
    callback: tego_file_transfer_progress_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_file_transfer_progress, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_file_transfer_complete_callback(
    context: *mut tego_context,
    callback: tego_file_transfer_complete_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_file_transfer_complete, context, callback, error);
}

#[no_mangle]
pub extern "C" fn tego_context_set_user_status_changed_callback(
    context: *mut tego_context,
    callback: tego_user_status_changed_callback,
    error: *mut *mut tego_error,
) {
    impl_callback_setter!(on_user_status_changed, context, callback, error);
}

//
// Destructors for various tego types
//

#[no_mangle]
pub extern "C" fn tego_error_delete(value: *mut tego_error) {
    impl_object_deleter!(tego_error, value);
}

#[no_mangle]
pub extern "C" fn tego_string_delete(value: *mut tego_string) {
    impl_object_deleter!(tego_string, value);
}

#[no_mangle]
pub extern "C" fn tego_context_delete(value: *mut tego_context) {
    impl_object_deleter!(tego_context, value);
}

#[no_mangle]
pub extern "C" fn tego_settings_delete(value: *mut tego_settings) {
    impl_object_deleter!(tego_settings, value);
}

#[no_mangle]
pub extern "C" fn tego_profile_delete(value: *mut tego_profile) {
    impl_object_deleter!(tego_profile, value);
}

#[no_mangle]
pub extern "C" fn tego_ed25519_private_key_delete(value: *mut tego_ed25519_private_key) {
    impl_object_deleter!(tego_ed25519_private_key, value);
}

#[no_mangle]
pub extern "C" fn tego_v3_onion_service_id_delete(value: *mut tego_v3_onion_service_id) {
    impl_object_deleter!(tego_v3_onion_service_id, value);
}

#[no_mangle]
pub extern "C" fn tego_user_id_delete(value: *mut tego_user_id) {
    impl_object_deleter!(tego_user_id, value);
}

#[no_mangle]
pub extern "C" fn tego_tor_config_delete(value: *mut tego_tor_config) {
    impl_object_deleter!(tego_tor_config, value);
}

#[no_mangle]
pub extern "C" fn tego_bridge_config_delete(value: *mut tego_bridge_config) {
    impl_object_deleter!(tego_bridge_config, value);
}

#[no_mangle]
pub extern "C" fn tego_proxy_config_delete(value: *mut tego_proxy_config) {
    impl_object_deleter!(tego_proxy_config, value);
}

#[no_mangle]
pub extern "C" fn tego_firewall_config_delete(value: *mut tego_firewall_config) {
    impl_object_deleter!(tego_firewall_config, value);
}
