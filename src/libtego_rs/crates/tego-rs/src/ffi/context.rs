// standard
use std::collections::BTreeMap;
use std::ffi::{c_char, c_void};
use std::path::PathBuf;

// extern
use anyhow::{bail, Result};
use tor_interface::tor_crypto::{Ed25519PrivateKey, V3OnionServiceId};

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

        let object = TegoObject::Context(Default::default());
        let key = get_object_map().insert(object);
        unsafe {
            *out_context = key as *mut tego_context;
        }
        if let Some(TegoObject::Context(context)) = get_object_map().get_mut(&key) {
            context.set_tego_key(key);
        } else {
            bail!("");
        }
        Ok(())
    })
}

/// Get the host's user_id (derived from private key)
///
/// @param context : the current tego context
/// @param out_host_user : returned user id
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_host_user_id(
    context: *const tego_context,
    out_host_user: *mut *mut tego_user_id,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(out_host_user);

        let key = context as TegoKey;
        let service_id = match get_object_map().get(&key) {
            Some(TegoObject::Context(context)) => {
                if let Some(service_id) = context.host_service_id() {
                    service_id
                } else {
                    bail!("no host key defined");
                }
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        };

        let host_user = get_object_map().insert(TegoObject::UserId(service_id));

        unsafe { *out_host_user = host_user as *mut tego_user_id };
        Ok(())
    })
}

/// Launch+configure tor, bootstrap, and begin event+network threads
///
/// @param context : the current tego context
/// @param tor_config : tor configuration params
/// @param host_private_key : the hosts private ed25519 key, or null if
///  we want to create a new identity
/// @param user_buffer : the list of all users we care about
/// @param user_type_buffer : the types associated with all of our users
/// @param user_count : the length of the user and user type buffers
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_begin(
    context: *mut tego_context,
    tor_config: *const tego_tor_daemon_config,
    host_private_key: *const tego_ed25519_private_key,
    user_buffer: *const *const tego_user_id,
    user_type_buffer: *const tego_user_type,
    user_count: usize,
    error: *mut *mut tego_error,
) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        bail_if_null!(tor_config);
        bail_if_null!(host_private_key);
        if user_count > 0 {
            bail_if_null!(user_buffer);
            bail_if_null!(user_type_buffer);
        }

        let handle = Handle::try_from(tor_config)?;
        let tor_config = tego_tor_daemon_config_map().get(&handle)?.clone();

        let key = host_private_key as TegoKey;
        let host_private_key: Ed25519PrivateKey = match get_object_map().get(&key) {
            Some(TegoObject::Ed25519PrivateKey(private_key)) => private_key.clone(),
            Some(_) => bail!(
                "not a tego_ed25519_private_key pointer: {:?}",
                key as *const c_void
            ),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        };
        let host_service_id = V3OnionServiceId::from_private_key(&host_private_key);

        let user_buffer = std::slice::from_raw_parts(user_buffer, user_count);
        let user_type_buffer = std::slice::from_raw_parts(user_type_buffer, user_count);

        let mut users: BTreeMap<V3OnionServiceId, tego_user_type> = Default::default();

        for (user_id, user_type) in user_buffer.iter().zip(user_type_buffer.iter()) {
            let key = *user_id as TegoKey;
            let user_id = match get_object_map().get(&key) {
                Some(TegoObject::UserId(service_id)) => service_id.clone(),
                Some(_) => bail!("not a tego_user_id pointer: {:?}", key as *const c_void),
                None => bail!("not a valid pointer: {:?}", key as *const c_void),
            };

            // ensure we have no dupes
            bail_if!(users.contains_key(&user_id));
            // ensure we're not adding our own service id
            bail_if_equal!(host_service_id, user_id);

            use tego_user_type::*;
            match user_type {
                tego_user_type_host => bail!("user type may not be tego_user_type_host"),
                _ => users.insert(user_id, *user_type),
            };
        }

        let key = context as TegoKey;
        match get_object_map().get_mut(&key) {
            Some(TegoObject::Context(context)) => {
                context.begin(tor_config, host_private_key.clone(), users)
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }
    })
}

/// Tear down a running context; cancels bootstrap and ends event+network threads
///
/// @param context : the current tego context
/// @param error : filled on error
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_end(context: *mut tego_context, error: *mut *mut tego_error) {
    translate_failures((), error, || -> Result<()> {
        bail_if_null!(context);
        let key = context as TegoKey;

        match get_object_map().get_mut(&key) {
            Some(TegoObject::Context(context)) => context.end(),
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }
        Ok(())
    })
}

/// Returns the number of charactres required (including null) to
/// write out the tor logs
///
/// @param context : the current tego context
/// @param error : filled on error
/// @return : the number of characters required
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_tor_logs_size(
    context: *const tego_context,
    error: *mut *mut tego_error,
) -> usize {
    translate_failures(0usize, error, || -> Result<usize> {
        bail_if_null!(context);

        let context_ptr = context;
        let key = context_ptr as TegoKey;
        match get_object_map().get(&key) {
            Some(TegoObject::Context(context)) => Ok(context.tor_logs_size()),
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }
    })
}

/// Fill the passed in buffer with the tor daemon's logs, each entry delimitted
/// by newline character '\n'
///
/// @param context : the current tego context
/// @param out_log_buffer : user allocated buffer where tor log is to be written
/// @param log_buffer_size : the size of the passed in out_log_buffer buffer
/// @param error : filled on error
/// @return : the nuber of characters written (including null terminator) to
///  out_log_buffer
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_tor_logs(
    context: *const tego_context,
    out_log_buffer: *mut c_char,
    log_buffer_size: usize,
    error: *mut *mut tego_error,
) -> usize {
    translate_failures(0usize, error, || -> Result<usize> {
        bail_if_null!(context);
        bail_if_null!(out_log_buffer);

        if log_buffer_size == 0usize {
            return Ok(0usize);
        }

        let context_ptr = context;
        let key = context_ptr as TegoKey;
        match get_object_map().get(&key) {
            Some(TegoObject::Context(context)) => {
                let tor_logs = context.tor_logs();
                let tor_logs = tor_logs.as_str();

                // number of bytes to write
                let bytes = std::cmp::min(log_buffer_size - 1, tor_logs.len());

                unsafe {
                    let out_log_buffer =
                        std::slice::from_raw_parts_mut(out_log_buffer as *mut u8, log_buffer_size);
                    std::ptr::copy(tor_logs.as_ptr(), out_log_buffer.as_mut_ptr(), bytes);
                    out_log_buffer[bytes] = 0u8;
                }
                Ok(bytes)
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }
    })
}

/// Get the null-terminated tor version string
///
/// @param context : the curent tego context
/// @param error : filled on error
/// @return : the version string for the context's running tor daemon
///
/// # Safety
///
/// All pointers must be properly initialised or NULL
#[no_mangle]
pub unsafe extern "C" fn tego_context_get_tor_version_string(
    context: *const tego_context,
    error: *mut *mut tego_error,
) -> *const c_char {
    translate_failures(std::ptr::null(), error, || -> Result<*const c_char> {
        bail_if_null!(context);

        let key = context as TegoKey;
        match get_object_map().get_mut(&key) {
            Some(TegoObject::Context(context)) => {
                if let Some(tor_version) = context.tor_version_string() {
                    Ok(tor_version.as_c_str().as_ptr())
                } else {
                    Ok(std::ptr::null())
                }
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }
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

        let key = context as TegoKey;
        match get_object_map().get(&key) {
            Some(TegoObject::Context(context)) => {
                use tego_tor_network_status::*;
                let status = if context.connect_complete() {
                    tego_tor_network_status_ready
                } else {
                    tego_tor_network_status_offline
                };

                unsafe { *out_status = status };
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }

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

        let user = user as TegoKey;
        let user = match get_object_map().get(&user) {
            Some(TegoObject::UserId(user)) => user.clone(),
            Some(_) => bail!("not a tego_user_id pointer: {:?}", user as *const c_void),
            None => bail!("not a valid pointer: {:?}", user as *const c_void),
        };

        let message = raw_to_str!(message, message_length)?;
        let message = message.to_string();
        use rico_protocol::v3::message::chat_channel::MessageText;
        let message: MessageText = message.try_into()?;

        let context = context as TegoKey;
        match get_object_map().get(&context) {
            Some(TegoObject::Context(context)) => {
                let message_id = context.send_message(user, message)?;
                unsafe { *out_id = message_id };
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", context as *const c_void),
            None => bail!("not a valid pointer: {:?}", context as *const c_void),
        }

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

        let object_map = get_object_map();

        let context = context as TegoKey;
        let context = match object_map.get(&context) {
            Some(TegoObject::Context(context)) => context,
            Some(_) => bail!("not a tego_context pointer: {:?}", context as *const c_void),
            None => bail!("not a valid pointer: {:?}", context as *const c_void),
        };

        let user = user as TegoKey;
        let user = match object_map.get(&user) {
            Some(TegoObject::UserId(user)) => user.clone(),
            Some(_) => bail!("not a tego_user_id pointer: {:?}", user as *const c_void),
            None => bail!("not a valid pointer: {:?}", user as *const c_void),
        };

        let file_path = raw_to_str!(file_path, file_path_length)?;
        let file_path = PathBuf::from(file_path);

        let (id, file_size) = context.send_file_transfer_request(user, file_path)?;

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

        let object_map = get_object_map();

        let context = context as TegoKey;
        let context = match object_map.get(&context) {
            Some(TegoObject::Context(context)) => context,
            Some(_) => bail!("not a tego_context pointer: {:?}", context as *const c_void),
            None => bail!("not a valid pointer: {:?}", context as *const c_void),
        };

        let user = user as TegoKey;
        let user = match object_map.get(&user) {
            Some(TegoObject::UserId(user)) => user.clone(),
            Some(_) => bail!("not a tego_user_id pointer: {:?}", user as *const c_void),
            None => bail!("not a valid pointer: {:?}", user as *const c_void),
        };

        match response {
            tego_file_transfer_response::tego_file_transfer_response_accept => {
                bail_if_null!(dest_path);
                bail_if_equal!(dest_path_length, 0usize);
                let dest_path = raw_to_str!(dest_path, dest_path_length)?;
                let dest_path = PathBuf::from(dest_path);

                context.accept_file_transfer_request(user, id, dest_path)?;
            }
            tego_file_transfer_response::tego_file_transfer_response_reject => {
                bail_if_not_null!(dest_path);
                bail_if_not_equal!(dest_path_length, 0usize);

                context.reject_file_transfer_request(user, id)?;
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

        let object_map = get_object_map();

        let context = context as TegoKey;
        let context = match object_map.get(&context) {
            Some(TegoObject::Context(context)) => context,
            Some(_) => bail!("not a tego_context pointer: {:?}", context as *const c_void),
            None => bail!("not a valid pointer: {:?}", context as *const c_void),
        };

        let user = user as TegoKey;
        let user = match object_map.get(&user) {
            Some(TegoObject::UserId(user)) => user.clone(),
            Some(_) => bail!("not a tego_user_id pointer: {:?}", user as *const c_void),
            None => bail!("not a valid pointer: {:?}", user as *const c_void),
        };

        context.cancel_file_transfer(user, id)?;

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
        let key = user as TegoKey;
        let service_id = match get_object_map().get(&key) {
            Some(TegoObject::UserId(user_id)) => user_id.clone(),
            Some(_) => bail!("not a tego_user_id pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        };

        let message = raw_to_str!(message, message_length)?;
        let message = message.to_string();
        use rico_protocol::v3::message::contact_request_channel::MessageText;
        let message: MessageText = message.try_into()?;

        let key = context as TegoKey;
        match get_object_map().get(&key) {
            Some(TegoObject::Context(context)) => {
                context.send_contact_request(service_id, message);
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }

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
        let key = user as TegoKey;
        let service_id = match get_object_map().get(&key) {
            Some(TegoObject::UserId(user_id)) => user_id.clone(),
            Some(_) => bail!("not a tego_user_id pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        };

        let key = context as TegoKey;
        match get_object_map().get(&key) {
            Some(TegoObject::Context(context)) => {
                context.acknowledge_contact_request(service_id, response)
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }

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
        let key = user as TegoKey;
        let service_id = match get_object_map().get(&key) {
            Some(TegoObject::UserId(user_id)) => user_id.clone(),
            Some(_) => bail!("not a tego_user_id pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        };

        let key = context as TegoKey;
        match get_object_map().get_mut(&key) {
            Some(TegoObject::Context(context)) => {
                context.forget_user(service_id)?;
            }
            Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
            None => bail!("not a valid pointer: {:?}", key as *const c_void),
        }

        Ok(())
    })
}
