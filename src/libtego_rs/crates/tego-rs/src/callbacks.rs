// std
use std::ffi::CString;
// extern
use anyhow::{Context, Result};
use tor_interface::tor_crypto::V3OnionServiceId;

// internal crates
use crate::ffi::*;
use crate::macros::*;

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
    /*
        HostOnionServiceStateChanged {
            state: tego_host_onion_service_state,
        },
        ChatRequestReceived {
            service_id: V3OnionServiceId,
            message: String,
        },
        ChatRequestResponseReceived {
            service_id: V3OnionServiceId,
            accepted_request: bool,
        },
        MessageReceived {
            service_id: V3OnionServiceId,
            timestamp: std::time::SystemTime,
            message_id: tego_message_id,
            message: String,
        },
        MessageAcknowledged {
            service_id: V3OnionServiceId,
            message_id: tego_message_id,
            accepted: bool,
        },
        FileTransferRequestReceived {
            sender: V3OnionServiceId,
            file_transfer_id: tego_file_transfer_id,
            file_name: String,
            file_size: u64,
        },
        FileTransferRequestAcknowledged {
            service_id: V3OnionServiceId,
            file_transfer_id: tego_file_transfer_id,
            accepted: bool,
        },
        FileTransferRequestResponseReceived {
            service_id: V3OnionServiceId,
            file_transfer_id: tego_file_transfer_id,
            response: tego_file_transfer_response,
        },
        FileTransferProgress {
            user_id: V3OnionServiceId,
            file_transfer_id: tego_file_transfer_id,
            direction: tego_file_transfer_direction,
            bytes_complete: u64,
            bytes_total: u64,
        },
        FileTransferComplete {
            user_id: V3OnionServiceId,
            file_transfer_id: tego_file_transfer_id,
            direction: tego_file_transfer_direction,
            result: tego_file_transfer_result,
        },
        UserStatusChanged {
            service_id: V3OnionServiceId,
            status: tego_user_status,
        },
    */
}

#[derive(Default)]
pub(crate) struct Callbacks {
    // pub on_tor_network_status_changed: tego_tor_network_status_changed_callback,
    pub on_tor_provider_initialized: tego_tor_provider_initialized_callback,
    pub on_tor_bootstrap_status_changed: tego_tor_bootstrap_status_changed_callback,
    pub on_tor_bootstrap_complete: tego_tor_bootstrap_complete_callback,
    pub on_tor_log_received: tego_tor_log_received_callback,
    // pub on_host_onion_service_state_changed: tego_host_onion_service_state_changed_callback,
    // pub on_chat_request_received: tego_chat_request_received_callback,
    // pub on_chat_request_response_received: tego_chat_request_response_received_callback,
    // pub on_message_received: tego_message_received_callback,
    // pub on_message_acknowledged: tego_message_acknowledged_callback,
    // pub on_file_transfer_request_received: tego_file_transfer_request_received_callback,
    // pub on_file_transfer_request_acknowledged: tego_file_transfer_request_acknowledged_callback,
    // pub on_file_transfer_request_response_received:
    //     tego_file_transfer_request_response_received_callback,
    // pub on_file_transfer_progress: tego_file_transfer_progress_callback,
    // pub on_file_transfer_complete: tego_file_transfer_complete_callback,
    // pub on_user_status_changed: tego_user_status_changed_callback,
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
            // HostOnionServiceStateChanged { state } => {
            //     let on_host_onion_service_state_changed = self
            //         .on_host_onion_service_state_changed
            //         .context("missing on_host_onion_service_state_changed callback")?;
            //     log_trace!("invoke on_host_onion_service_state_changed");

            //     on_host_onion_service_state_changed(context, state);
            // }
            // ChatRequestReceived {
            //     service_id,
            //     message,
            // } => {
            //     let on_chat_request_received = self
            //         .on_chat_request_received
            //         .context("missing on_chat_request_received callback")?;
            //     log_trace!("invoke on_chat_request_received");

            //     let sender = tego_user_id_map().insert(service_id);
            //     let message = CString::new(message.replace("\0", ""))
            //         .expect("chat request message contains null-byte");
            //     let message_len = message.as_bytes().len();

            //     on_chat_request_received(
            //         context,
            //         sender.into(),
            //         message.as_c_str().as_ptr(),
            //         message_len,
            //     );

            //     tego_user_id_map().remove(&sender)?;
            // }
            // ChatRequestResponseReceived {
            //     service_id,
            //     accepted_request,
            // } => {
            //     let on_chat_request_response_received = self
            //         .on_chat_request_response_received
            //         .context("missing on_chat_request_response_received callback")?;
            //     log_trace!("invoke on_chat_request_response_received");

            //     let sender = tego_user_id_map().insert(service_id);

            //     on_chat_request_response_received(context, sender.into(), accepted_request);

            //     tego_user_id_map().remove(&sender)?;
            // }
            // MessageReceived {
            //     service_id,
            //     timestamp,
            //     message_id,
            //     message,
            // } => {
            //     let on_message_received = self
            //         .on_message_received
            //         .context("missing on_message_received callback")?;
            //     log_trace!("invoke on_message_received");

            //     let user = tego_user_id_map().insert(service_id);
            //     let timestamp = timestamp
            //         .duration_since(std::time::UNIX_EPOCH)
            //         .unwrap_or(std::time::Duration::ZERO);
            //     let timestamp = timestamp.as_millis() as tego_time;
            //     assert!(timestamp > 0);
            //     let message = CString::new(message.replace("\0", ""))
            //         .expect("chat message contains null-byte");
            //     let message_len = message.as_bytes().len();

            //     on_message_received(
            //         context,
            //         user.into(),
            //         timestamp,
            //         message_id,
            //         message.as_c_str().as_ptr(),
            //         message_len,
            //     );

            //     tego_user_id_map().remove(&user)?;
            // }
            // MessageAcknowledged {
            //     service_id,
            //     message_id,
            //     accepted,
            // } => {
            //     let on_message_acknowledged = self
            //         .on_message_acknowledged
            //         .context("missing on_message_acknowledged callback")?;
            //     log_trace!("invoke on_message_acknowledged");

            //     let user = tego_user_id_map().insert(service_id);

            //     on_message_acknowledged(context, user.into(), message_id, accepted);

            //     tego_user_id_map().remove(&user)?;
            // }
            // FileTransferRequestReceived {
            //     sender,
            //     file_transfer_id,
            //     file_name,
            //     file_size,
            // } => {
            //     let on_file_transfer_request_received = self
            //         .on_file_transfer_request_received
            //         .context("missing on_file_transfer_request_received callback")?;
            //     log_trace!("invoke on_file_transfer_request_received");

            //     let sender = tego_user_id_map().insert(sender);
            //     let file_name = CString::new(file_name.replace("\0", ""))
            //         .expect("file name contains null-byte");
            //     let file_name_length = file_name.as_bytes().len();
            //     let file_name = file_name.as_c_str().as_ptr();

            //     on_file_transfer_request_received(
            //         context,
            //         sender.into(),
            //         file_transfer_id,
            //         file_name,
            //         file_name_length,
            //         file_size,
            //     );

            //     tego_user_id_map().remove(&sender)?;
            // }
            // FileTransferRequestAcknowledged {
            //     service_id,
            //     file_transfer_id,
            //     accepted,
            // } => {
            //     let on_file_transfer_request_acknowledged = self
            //         .on_file_transfer_request_acknowledged
            //         .context("missing on_file_transfer_request_acknowledged callback")?;
            //     log_trace!("invoke on_file_transfer_request_acknowledged");

            //     let user = tego_user_id_map().insert(service_id);

            //     on_file_transfer_request_acknowledged(
            //         context,
            //         user.into(),
            //         file_transfer_id,
            //         accepted,
            //     );

            //     tego_user_id_map().remove(&user)?;
            // }
            // FileTransferRequestResponseReceived {
            //     service_id,
            //     file_transfer_id,
            //     response,
            // } => {
            //     let on_file_transfer_request_response_received = self
            //         .on_file_transfer_request_response_received
            //         .context("missing on_file_transfer_request_response_received callback")?;
            //     log_trace!("invoke on_file_transfer_request_response_received");

            //     let user = tego_user_id_map().insert(service_id);

            //     on_file_transfer_request_response_received(
            //         context,
            //         user.into(),
            //         file_transfer_id,
            //         response,
            //     );
            //     tego_user_id_map().remove(&user)?;
            // }
            // FileTransferProgress {
            //     user_id,
            //     file_transfer_id,
            //     direction,
            //     bytes_complete,
            //     bytes_total,
            // } => {
            //     let on_file_transfer_progress = self
            //         .on_file_transfer_progress
            //         .context("missing on_file_transfer_progress callback")?;
            //     log_trace!("invoke on_file_transfer_progress");

            //     let user_id = tego_user_id_map().insert(user_id);

            //     on_file_transfer_progress(
            //         context,
            //         user_id.into(),
            //         file_transfer_id,
            //         direction,
            //         bytes_complete,
            //         bytes_total,
            //     );

            //     tego_user_id_map().remove(&user_id)?;
            // }
            // FileTransferComplete {
            //     user_id,
            //     file_transfer_id,
            //     direction,
            //     result,
            // } => {
            //     let on_file_transfer_complete = self
            //         .on_file_transfer_complete
            //         .context("missing on_file_transfer_complete callback")?;
            //     log_trace!("invoke on_file_transfer_complete");

            //     let user_id = tego_user_id_map().insert(user_id);

            //     on_file_transfer_complete(
            //         context,
            //         user_id.into(),
            //         file_transfer_id,
            //         direction,
            //         result,
            //     );

            //     tego_user_id_map().remove(&user_id)?;
            // }
            // UserStatusChanged { service_id, status } => {
            //     let on_user_status_changed = self
            //         .on_user_status_changed
            //         .context("missing on_user_status_changed callback")?;
            //     log_trace!("invoke on_user_status_changed");

            //     let user = tego_user_id_map().insert(service_id);

            //     on_user_status_changed(context, user.into(), status);

            //     tego_user_id_map().remove(&user)?;
            // }
            _ => {
                log_trace!("unhandled callback");
            }
        }
        Ok(())
    }
}
