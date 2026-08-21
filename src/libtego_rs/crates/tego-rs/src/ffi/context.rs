// standard
use std::path::PathBuf;

// extern
use anyhow::Result;

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

/// End an existing session, disconnect all connections
/// and close the profile
///
/// @param context : the current tego context
/// @param session_handle : the session to end
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
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

/// Send a text message from the host to the given user
///
/// @param context : the current tego context
/// @param session_handle : the sesion the recipient user belongs to
/// @param user_handle : the user to send a message to
/// @param message : message to send
/// @param out_id : filled with assigned message id for callbacks
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_send_message(
    context: *mut tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    message: *const tego_string,
    out_id: *mut tego_message_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(message);
        bail_if_null!(out_id);

        let message = message.try_into()?;
        let message = tego_string_map().get(&message)?.clone();
        let message = message.into_string()?;

        use rico_protocol::v3::message::chat_channel::MessageText;
        let message: MessageText = message.try_into()?;

        let context = Handle::try_from(context)?;
        let message_id =
            tego_context_map()
                .get(&context)?
                .send_message(session_handle, user_handle, message)?;
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
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    file_path: *const tego_string,
    out_id: *mut tego_file_transfer_id,
    out_file_size: *mut tego_file_size,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(file_path);

        let file_path = file_path.try_into()?;
        let file_path = tego_string_map().get(&file_path)?.clone().into_string()?;
        let file_path = PathBuf::from(file_path);

        let context = Handle::try_from(context)?;
        let (id, file_size) = tego_context_map()
            .get(&context)?
            .send_file_transfer_request(session_handle, user_handle, file_path)?;

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
/// @param session_handle : the session the user belongs to
/// @param user_handle : the user that sent the file transfer request
/// @param id : which file transfer to respond to
/// @param response : how to respond to the request
/// @param dest_path : optional, destination to save the file
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_respond_file_transfer_request(
    context: *mut tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    id: tego_file_transfer_id,
    response: tego_file_transfer_response,
    dest_path: *const tego_string,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);

        let context = Handle::try_from(context)?;
        match response {
            tego_file_transfer_response::tego_file_transfer_response_accept => {
                bail_if_null!(dest_path);

                let dest_path = Handle::try_from(dest_path)?;
                let dest_path = tego_string_map().get(&dest_path)?.clone().into_string()?;
                let dest_path = PathBuf::from(dest_path);
                tego_context_map()
                    .get(&context)?
                    .accept_file_transfer_request(session_handle, user_handle, id, dest_path)?;
            }
            tego_file_transfer_response::tego_file_transfer_response_reject => {
                bail_if_not_null!(dest_path);

                tego_context_map()
                    .get(&context)?
                    .reject_file_transfer_request(session_handle, user_handle, id)?;
            }
        }

        Ok(())
    })
}

/// Cancel an in-progress file transfer
///
/// @param context : the current tego context
/// @param session_handle : the session the file transfer is on
/// @param user_handle : the user that is sending/receiving the transfer
/// @param file_transfer_id : the file transfer to cancel
/// @param error: filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_cancel_file_transfer(
    context: *mut tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    file_transfer_id: tego_file_transfer_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);

        let context = Handle::try_from(context)?;
        tego_context_map().get(&context)?.cancel_file_transfer(
            session_handle,
            user_handle,
            file_transfer_id,
        )?;

        Ok(())
    })
}

/// Sends a request to chat to a user
///
/// @param context : the current tego context
/// @param session_handle : the session to send chat  request on
/// @param service_id : the service id of the user we want to chat with
/// @param pet_name : the pet name to give this pending contact
/// @param mesage : text greeting message to send
/// @param out_user_handle : the new user handle for this user
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_send_chat_request(
    context: *mut tego_context,
    session_handle: tego_session_handle,
    service_id: *const tego_v3_onion_service_id,
    pet_name: *const tego_string,
    message: *const tego_string,
    out_user_handle: *mut tego_user_handle,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(service_id);
        bail_if_null!(pet_name);
        bail_if_null!(message);
        bail_if_null!(out_user_handle);

        let service_id = Handle::try_from(service_id)?;
        let service_id = tego_v3_onion_service_id_map().get(&service_id)?.clone();

        let pet_name = Handle::try_from(pet_name)?;
        let pet_name = tego_string_map().get(&pet_name)?.clone().into_string()?;

        let message = Handle::try_from(message)?;
        let message = tego_string_map().get(&message)?.clone().into_string()?;
        use rico_protocol::v3::message::contact_request_channel::MessageText;
        let message: MessageText = message.try_into()?;

        let context = Handle::try_from(context)?;
        let user_handle = tego_context_map().get(&context)?.send_contact_request(
            session_handle,
            service_id,
            pet_name,
            message,
        )?;

        unsafe {
            *out_user_handle = user_handle;
        }

        Ok(())
    })
}

/// Acknowledges chat request sent from another user. Would be called after receiving
/// a chat_request_received callback.
///
/// @param context : the current tego context
/// @param session_handle : the session to ack on
/// @param user_handle : the user that sent the chat request
/// @param response : how to respond to the request
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_acknowledge_chat_request(
    context: *mut tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    response: tego_chat_acknowledge,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        let context = Handle::try_from(context)?;
        tego_context_map()
            .get(&context)?
            .acknowledge_contact_request(session_handle, user_handle, response)?;

        Ok(())
    })
}

/// Forget about a given user, said user will be removed
/// from all internal lists and will be needed to be re-added
/// to chat
///
/// @param context : the current tego context
/// @param session_handle : the session the user is in
/// @param user_handle : the user to forget
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_forget_user(
    context: *mut tego_context,
    session_handle: tego_session_handle,
    user_handle: tego_user_handle,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        let context = Handle::try_from(context)?;
        tego_context_map()
            .get_mut(&context)?
            .forget_user(session_handle, user_handle)?;

        Ok(())
    })
}
