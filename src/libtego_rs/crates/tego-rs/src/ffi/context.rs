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

/// Create a new session from an unlocked profile
///
/// @param context : the current tego context
/// @param out_session_handle : destination to save handle
/// @param profile : profile associated with this session; consumed by this
///  function if arguments
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_begin_session(
    context: *mut tego_context,
    out_session_handle: *mut tego_session_handle,
    profile: *mut tego_profile,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(out_session_handle);
        bail_if_null!(profile);

        let profile = Handle::try_from(profile)?;
        let profile = tego_profile_map().remove(&profile)?;

        let context = Handle::try_from(context)?;
        let session_handle = tego_context_map()
            .get_mut(&context)?
            .begin_session(profile)?;

        unsafe {
            *out_session_handle = session_handle;
        }

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn tego_context_end_session(
    context: *mut tego_context,
    session_handle: tego_session_handle,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_equal!(session_handle, TEGO_INVALID_SESSION_HANDLE);

        let context = Handle::try_from(context)?;
        tego_context_map()
            .get_mut(&context)?
            .end_session(session_handle)?;

        Ok(())
    })
}

/// Get the total number of users known to this session
///
/// @param context : the current tego context
/// @param session_handle : the session to get user number of
/// @param out_user_count : numbe of users is stored here
/// @param error : filled on eror
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub extern "C" fn tego_context_get_user_count(
    context: *const tego_context,
    session_handle: tego_session_handle,
    out_user_count: *mut usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_equal!(session_handle, TEGO_INVALID_SESSION_HANDLE);
        bail_if_null!(out_user_count);

        let context = Handle::try_from(context)?;
        let user_count = tego_context_map()
            .get(&context)?
            .get_user_count(session_handle)?;

        unsafe {
            *out_user_count = user_count;
        }

        Ok(())
    })
}

/// Get the user handles for al users in this session
///
/// @param context : the current tego context
/// @param session_handle : the session to get the user handles from
/// @param out_user_handles_buffer : buffer to store all the user handles
/// @param user_handles_buffer_length : the number of handles which can be stored in
///  out_user_handles_buffer
/// @param error : filled on error
///
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub extern "C" fn tego_context_get_user_handles(
    context: *const tego_context,
    session_handle: tego_session_handle,
    out_user_handles_buffer: *mut tego_user_handle,
    user_handles_buffer_length: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_equal!(session_handle, TEGO_INVALID_SESSION_HANDLE);
        bail_if!(out_user_handles_buffer.is_null() || user_handles_buffer_length == 0usize);

        let context = Handle::try_from(context)?;
        let user_handles = match tego_context_map().get(&context) {
            Ok(context) => context.get_user_handles(session_handle)?,
            Err(err) => return Err(err),
        };
        bail_if!(user_handles.len() != user_handles_buffer_length);
        let out_user_handles_buffer = unsafe {
            std::slice::from_raw_parts_mut(out_user_handles_buffer, user_handles_buffer_length)
        };
        out_user_handles_buffer.copy_from_slice(user_handles.as_slice());

        Ok(())
    })
}

/// Get a particular user's type
///
/// @param context : the current tego context
/// @param session_handle : the session to the user is in
/// @param user_handle : the user to get the type of
/// @param out_user_type : user type stored here
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_user_type(
    context: *const tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    out_user_type: *mut tego_user_type,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_equal!(session_handle, TEGO_INVALID_SESSION_HANDLE);
        bail_if_equal!(user_handle, TEGO_INVALID_USER_HANDLE);
        bail_if_null!(out_user_type);

        let context = Handle::try_from(context)?;
        let user_type = tego_context_map()
            .get(&context)?
            .get_user_type(session_handle, user_handle)?;

        unsafe {
            *out_user_type = user_type.into();
        }
        Ok(())
    })
}

/// Get a particular user's nickname
///
/// @param context : the current tego context
/// @param session_handle : the session to the user is in
/// @param user_handle : the user to get the type of
/// @param out_user_nickname : user nickname stored here
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_user_nickname(
    context: *const tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    out_user_nickname: *mut *mut tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_equal!(session_handle, TEGO_INVALID_SESSION_HANDLE);
        bail_if_equal!(user_handle, TEGO_INVALID_USER_HANDLE);
        bail_if_null!(out_user_nickname);

        let context = Handle::try_from(context)?;
        let user_nickname = tego_context_map()
            .get(&context)?
            .get_user_nickname(session_handle, user_handle)?;
        let user_nickname = CString::new(user_nickname)?;
        let user_nickname = tego_string_map().insert(user_nickname);
        unsafe {
            *out_user_nickname = user_nickname.into();
        }
        Ok(())
    })
}

/// Get a particular user's pet name
///
/// @param context : the current tego context
/// @param session_handle : the session to the user is in
/// @param user_handle : the user to get the pet name of
/// @param out_user_type : user pet name stored here
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_user_pet_name(
    context: *const tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    out_user_pet_name: *mut *mut tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_equal!(session_handle, TEGO_INVALID_SESSION_HANDLE);
        bail_if_equal!(user_handle, TEGO_INVALID_USER_HANDLE);
        bail_if_null!(out_user_pet_name);

        let context = Handle::try_from(context)?;
        let user_pet_name = tego_context_map()
            .get(&context)?
            .get_user_pet_name(session_handle, user_handle)?;
        let user_pet_name = if let Some(user_pet_name) = user_pet_name {
            let user_pet_name = CString::new(user_pet_name)?;
            let user_pet_name = tego_string_map().insert(user_pet_name);
            user_pet_name.into()
        } else {
            std::ptr::null_mut()
        };

        unsafe {
            *out_user_pet_name = user_pet_name;
        }
        Ok(())
    })
}

/*
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
*/
