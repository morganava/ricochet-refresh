// std
use std::ffi::CString;
// extern
use anyhow::{Context, Result};
use rico_profile::v4::profile::UserType;

// internal crates
use crate::ffi::*;
use crate::macros::*;
use crate::session::*;

#[allow(dead_code)]
pub(crate) enum CallbackData {
    TorNetworkStatusChanged {
        status: tego_tor_network_status,
    },
    TorProviderInitialized {
        tor_config_type: tego_tor_config_type,
        version: Option<String>,
    },
    TorBootstrapStatusChanged {
        progress: u32,
        tag: String,
    },
    TorBootstrapComplete,
    TorLogReceived {
        line: String,
    },
    SessionBegan {
        session_handle: SessionHandle,
        // list of user handles and their display name
        users: Vec<(UserHandle, UserType, String)>,
    },
    // HostOnionServiceStateChanged {
    //     state: tego_host_onion_service_state,
    // },
    UserAdded {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        user_type: UserType,
    },
    UserRemoved {
        session_handle: SessionHandle,
        user_handle: UserHandle,
    },
    UserStatusChanged {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        status: tego_user_status,
    },
    ChatRequestReceived {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        message: String,
    },
    ChatRequestResponseReceived {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        accepted_request: bool,
    },
    MessageReceived {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        timestamp: std::time::SystemTime,
        message_id: tego_message_id,
        message: String,
    },
    MessageAcknowledged {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        message_id: tego_message_id,
        accepted: bool,
    },
    FileTransferRequestReceived {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        file_name: String,
        file_size: u64,
    },
    FileTransferRequestAcknowledged {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        accepted: bool,
    },
    FileTransferRequestResponseReceived {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        response: tego_file_transfer_response,
    },
    FileTransferProgress {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        direction: tego_file_transfer_direction,
        bytes_complete: u64,
        bytes_total: u64,
    },
    FileTransferComplete {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        direction: tego_file_transfer_direction,
        result: tego_file_transfer_result,
    },
}

#[derive(Default)]
pub(crate) struct Callbacks {
    // pub on_tor_network_status_changed: tego_tor_network_status_changed_callback,
    pub on_tor_provider_initialized: tego_tor_provider_initialized_callback,
    pub on_tor_bootstrap_status_changed: tego_tor_bootstrap_status_changed_callback,
    pub on_tor_bootstrap_complete: tego_tor_bootstrap_complete_callback,
    pub on_tor_log_received: tego_tor_log_received_callback,
    pub on_session_began: tego_session_began_callback,
    // pub on_host_onion_service_state_changed: tego_host_onion_service_state_changed_callback,
    pub on_user_added: tego_user_added_callback,
    pub on_user_removed: tego_user_removed_callback,
    pub on_user_status_changed: tego_user_status_changed_callback,
    pub on_chat_request_received: tego_chat_request_received_callback,
    pub on_chat_request_response_received: tego_chat_request_response_received_callback,
    pub on_message_received: tego_message_received_callback,
    pub on_message_acknowledged: tego_message_acknowledged_callback,
    pub on_file_transfer_request_received: tego_file_transfer_request_received_callback,
    pub on_file_transfer_request_acknowledged: tego_file_transfer_request_acknowledged_callback,
    pub on_file_transfer_request_response_received:
        tego_file_transfer_request_response_received_callback,
    pub on_file_transfer_progress: tego_file_transfer_progress_callback,
    pub on_file_transfer_complete: tego_file_transfer_complete_callback,
}

impl Callbacks {
    pub fn invoke(&self, context: *mut tego_context, callback_data: CallbackData) -> Result<()> {
        use CallbackData::*;
        match callback_data {
            // TorNetworkStatusChanged { status } => {
            //     let on_tor_network_status_changed = self
            //         .on_tor_network_status_changed
            //         .context("missing on_tor_network_status_changed callback")?;
            //     log_trace!("invoke on_tor_network_status_changed");

            //     on_tor_network_status_changed(context, status);
            // }
            TorProviderInitialized {
                tor_config_type,
                version,
            } => {
                let on_tor_provider_initialized = self
                    .on_tor_provider_initialized
                    .context("missing on_tor_provider_initialized callback")?;
                log_trace!("invoke on_tor_provider_initialized");

                if let Some(version) = version {
                    let version = tego_string_map()
                        .insert(CString::new(version).expect("version string contains null-byte"));
                    on_tor_provider_initialized(context, tor_config_type, version.into());
                    let _ = tego_string_map().remove(&version);
                } else {
                    on_tor_provider_initialized(context, tor_config_type, std::ptr::null_mut());
                }
            }
            TorBootstrapStatusChanged { progress, tag } => {
                let on_tor_bootstrap_status_changed = self
                    .on_tor_bootstrap_status_changed
                    .context("missing on_tor_bootstrap_status_changed callback")?;
                log_trace!("invoke on_tor_bootstrap_status_changed");

                on_tor_bootstrap_status_changed(context, progress as i32, tag.as_str().into());
            }
            TorBootstrapComplete => {
                let on_tor_bootstrap_complete = self
                    .on_tor_bootstrap_complete
                    .context("missing on_tor_bootstrap_complete callback")?;
                log_trace!("invoke on_tor_bootstrap_complete");

                on_tor_bootstrap_complete(context);
            }
            TorLogReceived { line } => {
                let on_tor_log_received = self
                    .on_tor_log_received
                    .context("missing on_tor_log_received callback")?;
                log_trace!("invoke on_tor_log_received");

                let line =
                    CString::new(line.replace("\0", "")).expect("tor log line contains null-byte");
                let line = tego_string_map().insert(line);
                on_tor_log_received(context, line.into());
                let _ = tego_string_map().remove(&line);
            }
            SessionBegan {
                session_handle,
                users,
            } => {
                let on_session_began = self
                    .on_session_began
                    .context("missing on_session_began callback")?;
                log_trace!("invoke on_session_began");

                let user_count = users.len();
                let mut user_handles: Vec<tego_user_handle> = Vec::with_capacity(user_count);
                let mut user_types: Vec<tego_user_type> = Vec::with_capacity(user_count);
                let mut user_display_names: Vec<TegoStringHandle> = Vec::with_capacity(user_count);

                for (user_handle, user_type, user_display_name) in users {
                    let user_display_name = CString::new(user_display_name)?;
                    let user_display_name = tego_string_map().insert(user_display_name);

                    user_handles.push(user_handle);
                    user_types.push(user_type.into());
                    user_display_names.push(user_display_name);
                }

                on_session_began(
                    context,
                    session_handle,
                    user_handles.as_ptr(),
                    user_types.as_ptr(),
                    user_display_names.as_ptr() as *const *const tego_string,
                    user_count,
                );

                for user_display_name in user_display_names {
                    tego_string_map().remove(&user_display_name)?;
                }
            }
            // HostOnionServiceStateChanged { state } => {
            //     let on_host_onion_service_state_changed = self
            //         .on_host_onion_service_state_changed
            //         .context("missing on_host_onion_service_state_changed callback")?;
            //     log_trace!("invoke on_host_onion_service_state_changed");

            //     on_host_onion_service_state_changed(context, state);
            // }
            UserAdded {
                session_handle,
                user_handle,
                user_type,
            } => {
                let on_user_added = self
                    .on_user_added
                    .context("missing on_user_added callback")?;
                log_trace!("invoke on_user_added");

                on_user_added(context, session_handle, user_handle, user_type.into());
            }
            UserRemoved {
                session_handle,
                user_handle,
            } => {
                let on_user_removed = self
                    .on_user_removed
                    .context("missing on_user_removed callback")?;
                log_trace!("invoke on_user_removed");

                on_user_removed(context, session_handle, user_handle);
            }
            UserStatusChanged {
                session_handle,
                user_handle,
                status,
            } => {
                let on_user_status_changed = self
                    .on_user_status_changed
                    .context("missing on_user_status_changed callback")?;
                log_trace!("invoke on_user_status_changed");

                on_user_status_changed(context, session_handle, user_handle, status);
            }
            ChatRequestReceived {
                session_handle,
                user_handle,
                message,
            } => {
                let on_chat_request_received = self
                    .on_chat_request_received
                    .context("missing on_chat_request_received callback")?;
                log_trace!("invoke on_chat_request_received");

                let sender_user_handle = user_handle;
                let message = CString::new(message.replace("\0", ""))
                    .expect("chat request message contains null-byte");
                let message_handle = tego_string_map().insert(message);

                on_chat_request_received(
                    context,
                    session_handle,
                    sender_user_handle,
                    message_handle.into(),
                );

                tego_string_map().remove(&message_handle)?;
            }
            ChatRequestResponseReceived {
                session_handle,
                user_handle,
                accepted_request,
            } => {
                let on_chat_request_response_received = self
                    .on_chat_request_response_received
                    .context("missing on_chat_request_response_received callback")?;
                log_trace!("invoke on_chat_request_response_received");

                on_chat_request_response_received(
                    context,
                    session_handle,
                    user_handle,
                    accepted_request,
                );
            }
            MessageReceived {
                session_handle,
                user_handle,
                timestamp,
                message_id,
                message,
            } => {
                let on_message_received = self
                    .on_message_received
                    .context("missing on_message_received callback")?;
                log_trace!("Invoke on_message_received");

                let timestamp = timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::ZERO);
                let timestamp = timestamp.as_millis() as tego_time;
                assert!(timestamp > 0);
                let message = CString::new(message.replace("\0", ""))
                    .expect("Chat message contains null-byte");
                let message = tego_string_map().insert(message);

                on_message_received(
                    context,
                    session_handle,
                    user_handle,
                    timestamp,
                    message_id,
                    message.into(),
                );

                tego_string_map().remove(&message)?;
            }
            MessageAcknowledged {
                session_handle,
                user_handle,
                message_id,
                accepted,
            } => {
                let on_message_acknowledged = self
                    .on_message_acknowledged
                    .context("missing on_message_acknowledged callback")?;
                log_trace!("Invoke on_message_acknowledged");

                on_message_acknowledged(context, session_handle, user_handle, message_id, accepted);
            }
            FileTransferRequestReceived {
                session_handle,
                user_handle,
                file_transfer_id,
                file_name,
                file_size,
            } => {
                let on_file_transfer_request_received = self
                    .on_file_transfer_request_received
                    .context("missing on_file_transfer_request_received callback")?;
                log_trace!("invoke on_file_transfer_request_received");

                let file_name = CString::new(file_name.replace("\0", ""))
                    .expect("file name contains null-byte");
                let file_name = tego_string_map().insert(file_name);

                on_file_transfer_request_received(
                    context,
                    session_handle,
                    user_handle,
                    file_transfer_id,
                    file_name.into(),
                    file_size,
                );

                tego_string_map().remove(&file_name)?;
            }
            FileTransferRequestAcknowledged {
                session_handle,
                user_handle,
                file_transfer_id,
                accepted,
            } => {
                let on_file_transfer_request_acknowledged = self
                    .on_file_transfer_request_acknowledged
                    .context("missing on_file_transfer_request_acknowledged callback")?;
                log_trace!("invoke on_file_transfer_request_acknowledged");

                on_file_transfer_request_acknowledged(
                    context,
                    session_handle,
                    user_handle,
                    file_transfer_id,
                    accepted,
                );
            }
            FileTransferRequestResponseReceived {
                session_handle,
                user_handle,
                file_transfer_id,
                response,
            } => {
                let on_file_transfer_request_response_received = self
                    .on_file_transfer_request_response_received
                    .context("missing on_file_transfer_request_response_received callback")?;
                log_trace!("invoke on_file_transfer_request_response_received");

                on_file_transfer_request_response_received(
                    context,
                    session_handle,
                    user_handle,
                    file_transfer_id,
                    response,
                );
            }
            FileTransferProgress {
                session_handle,
                user_handle,
                file_transfer_id,
                direction,
                bytes_complete,
                bytes_total,
            } => {
                let on_file_transfer_progress = self
                    .on_file_transfer_progress
                    .context("missing on_file_transfer_progress callback")?;
                log_trace!("invoke on_file_transfer_progress");

                on_file_transfer_progress(
                    context,
                    session_handle,
                    user_handle,
                    file_transfer_id,
                    direction,
                    bytes_complete,
                    bytes_total,
                );
            }
            FileTransferComplete {
                session_handle,
                user_handle,
                file_transfer_id,
                direction,
                result,
            } => {
                let on_file_transfer_complete = self
                    .on_file_transfer_complete
                    .context("missing on_file_transfer_complete callback")?;
                log_trace!("invoke on_file_transfer_complete");

                on_file_transfer_complete(
                    context,
                    session_handle,
                    user_handle,
                    file_transfer_id,
                    direction,
                    result,
                );
            }
            // todo: remove me
            _ => {
                log_trace!("unhandled callback");
            }
        }
        Ok(())
    }
}
