// standard
use std::collections::BTreeMap;
use std::ffi::c_char;
use std::path::PathBuf;

// extern
use anyhow::{bail, Context, Result};
use tor_interface::tor_crypto::V3OnionServiceId;

// internal
use crate::error::translate_failures;
use crate::ffi::*;
use crate::macros::*;

/// Initialize a new tego_context
///
/// @param out_context : returned context
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_initialize(
    out_context: *mut *mut tego_context,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(out_context);

        let context = tego_context_map().insert(Default::default());
        unsafe {
            *out_context = context.into();
        }
        tego_context_map()
            .get_mut(&context)?
            .start_event_loop(context);
        Ok(())
    })
}

/// Begin connecting to the tor network with the given tego_tor_config
///
/// @param context : the current tego context
/// @param tor_config : the tor config to use
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_begin_bootstrap(
    context: *mut tego_context,
    tor_config: *const tego_tor_config,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(tor_config);

        let tor_config = Handle::try_from(tor_config)?;
        let tor_config = tego_tor_config_map().get(&tor_config)?.clone();

        let context = Handle::try_from(context)?;
        tego_context_map()
            .get_mut(&context)?
            .begin_bootstrap(tor_config)?;

        Ok(())
    })
}

/// Cancel on-going bootstrap attempt and drop backing TorProvider
///
/// @param context : the current tego context
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_cancel_bootstrap(
    context: *mut tego_context,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);

        let context = Handle::try_from(context)?;
        tego_context_map().get_mut(&context)?.cancel_bootstrap()?;

        Ok(())
    })
}

/// Get the current status of the tor daemon's connection
/// to the tor network
///
/// @param context : the current tego context
/// @param out_status : destination to save network status
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_tor_network_status(
    context: *const tego_context,
    out_status: *mut tego_tor_network_status,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(out_status);

        let context = Handle::try_from(context)?;
        use tego_tor_network_status::*;
        let status = if tego_context_map().get(&context)?.connect_complete() {
            tego_tor_network_status_ready
        } else {
            tego_tor_network_status_offline
        };

        unsafe { *out_status = status };
        Ok(())
    })
}

/// Send a text message from the host to the given user
///
/// @param context : the current tego context
/// @param user : the user to send a message to
/// @param message : utf8 text message to send
/// @param message_length : length of message not including null-terminator
/// @param out_id : filled with assigned message id for callbacks
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_send_message(
    context: *mut tego_context,
    user: *const tego_user_id,
    message: *const c_char,
    message_length: usize,
    out_id: *mut tego_message_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(user);
        bail_if_null!(message);
        bail_if_equal!(message_length, 0usize);
        bail_if_null!(out_id);

        let user = Handle::try_from(user)?;
        let user = tego_user_id_map().get(&user)?.clone();

        let message = raw_to_str!(message, message_length)?;
        let message = message.to_string();
        use rico_protocol::v3::message::chat_channel::MessageText;
        let message: MessageText = message.try_into()?;

        let context = Handle::try_from(context)?;
        let message_id = tego_context_map()
            .get(&context)?
            .send_message(user, message)?;
        unsafe { *out_id = message_id };

        Ok(())
    })
}

/// Request to send a file to the given user
///
/// @param context : the current tego context
/// @param user : the user to send a file to
/// @param file_path : utf8 path to file to send
/// @param file_path_length : length of file_path not including null-terminator
/// @param out_id : optional, filled with assigned file transfer id for callbacks
/// @param out_file_size : optional, filled with the size of the file in bytes
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised
#[no_mangle]
pub unsafe extern "C" fn tego_context_send_file_transfer_request(
    context: *mut tego_context,
    user: *const tego_user_id,
    file_path: *const c_char,
    file_path_length: usize,
    out_id: *mut tego_file_transfer_id,
    out_file_size: *mut tego_file_size,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(user);
        bail_if_null!(file_path);
        bail_if_equal!(file_path_length, 0usize);

        let user = Handle::try_from(user)?;
        let user = tego_user_id_map().get(&user)?.clone();

        let file_path = raw_to_str!(file_path, file_path_length)?;
        let file_path = PathBuf::from(file_path);

        let context = Handle::try_from(context)?;
        let (id, file_size) = tego_context_map()
            .get(&context)?
            .send_file_transfer_request(user, file_path)?;

        // write out results
        unsafe {
            if !out_id.is_null() {
                *out_id = id;
            }
            if !out_file_size.is_null() {
                *out_file_size = file_size;
            }
        }

        Ok(())
    })
}

/// Acknowledges a request to send an file_transfer
///
/// @param context : the current tego context
/// @param user : the user that sent the file transfer request
/// @param id : which file transfer to respond to
/// @param response : how to respond to the request
/// @param dest_path : optional, destination to save the file
/// @param dest_path_length : length of dest_path not including the null-terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_respond_file_transfer_request(
    context: *mut tego_context,
    user: *const tego_user_id,
    id: tego_file_transfer_id,
    response: tego_file_transfer_response,
    dest_path: *const c_char,
    dest_path_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(user);

        let context = Handle::try_from(context)?;

        let user = Handle::try_from(user)?;
        let user = tego_user_id_map().get(&user)?.clone();

        match response {
            tego_file_transfer_response::tego_file_transfer_response_accept => {
                bail_if_null!(dest_path);
                bail_if_equal!(dest_path_length, 0usize);
                let dest_path = raw_to_str!(dest_path, dest_path_length)?;
                let dest_path = PathBuf::from(dest_path);

                tego_context_map()
                    .get(&context)?
                    .accept_file_transfer_request(user, id, dest_path)?;
            }
            tego_file_transfer_response::tego_file_transfer_response_reject => {
                bail_if_not_null!(dest_path);
                bail_if_not_equal!(dest_path_length, 0usize);

                tego_context_map()
                    .get(&context)?
                    .reject_file_transfer_request(user, id)?;
            }
        }

        Ok(())
    })
}

/// Cancel an in-progress file transfer
///
/// @param context : the current tego context
/// @param user : the user that is sending/receiving the transfer
/// @param id : the file transfer to cancel
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_cancel_file_transfer(
    context: *mut tego_context,
    user: *const tego_user_id,
    id: tego_file_transfer_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(user);

        let user = Handle::try_from(user)?;
        let user = tego_user_id_map().get(&user)?.clone();

        let context = Handle::try_from(context)?;
        tego_context_map()
            .get(&context)?
            .cancel_file_transfer(user, id)?;

        Ok(())
    })
}

/// Sends a request to chat to a user
///
/// @param context : the current tego context
/// @param user : the user we want to chat with
/// @param mesage : utf8 text greeting message to send
/// @param message_length : length of message not including null-terminator
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_send_chat_request(
    context: *mut tego_context,
    user: *const tego_user_id,
    message: *const c_char,
    message_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        let user = Handle::try_from(user)?;
        let user = tego_user_id_map().get(&user)?.clone();

        let message = raw_to_str!(message, message_length)?;
        let message = message.to_string();
        use rico_protocol::v3::message::contact_request_channel::MessageText;
        let message: MessageText = message.try_into()?;

        let context = Handle::try_from(context)?;
        tego_context_map()
            .get(&context)?
            .send_contact_request(user, message);

        Ok(())
    })
}

/// Acknowledges chat request sent from another user. Would be called after receiving
/// a chat_request_received callback.
///
/// @param context : the current tego context
/// @param user : the user that sent the chat request
/// @param response : how to respond to the request
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_acknowledge_chat_request(
    context: *mut tego_context,
    user: *const tego_user_id,
    response: tego_chat_acknowledge,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        let user = Handle::try_from(user)?;
        let user = tego_user_id_map().get(&user)?.clone();

        let context = Handle::try_from(context)?;
        tego_context_map()
            .get(&context)?
            .acknowledge_contact_request(user, response);

        Ok(())
    })
}

/// Forget about a given user, said user will be removed
/// from all internal lists and will be needed to be re-added
/// to chat
///
/// @param context : the current tego context
/// @param user : the user to forget
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_forget_user(
    context: *mut tego_context,
    user: *const tego_user_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        let user = Handle::try_from(user)?;
        let user = tego_user_id_map().get(&user)?.clone();

        let context = Handle::try_from(context)?;
        tego_context_map().get_mut(&context)?.forget_user(user)?;

        Ok(())
    })
}
