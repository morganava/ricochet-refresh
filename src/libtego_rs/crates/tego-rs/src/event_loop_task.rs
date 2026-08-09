// standard
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsString;
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, Write};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, Weak,
};
use std::time::{Duration, Instant};

// extern
use anyhow::{anyhow, Context as AnyhowContext, Result};
use rico_profile::v4::profile::{User, UserType};
use rico_protocol::v3::file_hasher::*;
use rico_protocol::v3::packet_handler::*;
use rico_protocol::v3::Error;
use tor_interface::legacy_tor_client::LegacyTorClientConfig;
use tor_interface::legacy_tor_client::*;
use tor_interface::legacy_tor_version::LegacyTorVersion;
use tor_interface::tor_crypto::{Ed25519PrivateKey, V3OnionServiceId};
use tor_interface::tor_provider::{OnionStream, TorEvent, TorProvider};

// internal crates
use crate::callbacks::*;
use crate::command_queue::*;
use crate::context::*;
use crate::ffi::*;
use crate::macros::*;
use crate::session::*;

pub(crate) struct EventLoopTask {
    context_handle: TegoContextHandle,
    callbacks: Weak<Mutex<Callbacks>>,
    command_queue: CommandQueue,

    tor_provider: Option<Box<dyn TorProvider>>,
    // map of async connection handles to the session which requested them
    connect_handles: BTreeMap<tor_interface::tor_provider::ConnectHandle, SessionHandle>,

    // our open sessions
    session_map: BTreeMap<SessionHandle, Session>,
    // the next session id
    next_session_handle: SessionHandle,

    callback_queue: Vec<CallbackData>,
    task_complete: bool,
}

impl EventLoopTask {
    pub fn new(
        context_handle: TegoContextHandle,
        callbacks: Weak<Mutex<Callbacks>>,
        command_queue: CommandQueue,
    ) -> Self {
        Self {
            context_handle,
            callbacks,
            command_queue,
            tor_provider: None,
            connect_handles: Default::default(),
            session_map: Default::default(),
            next_session_handle: 0,
            callback_queue: Default::default(),
            task_complete: false,
        }
    }

    pub fn run(mut self) -> Result<()> {
        log_trace!();

        // TODO: add a check here to sleep a frame if the previous iteration did not do any work
        while !self.task_complete {
            // handle tor provider events
            self.handle_tor_events()?;

            // get and handle pending commands
            self.handle_commands()?;

            self.handle_sessions()?;

            // read any pending bytes and update the packet handler
            // self.handle_connections()?;

            // trigger callbacks for frontend
            self.handle_callbacks()?;
        }

        // TODO: we should trigger exit callback here?

        Ok(())
    }

    fn retry_delay(failure_count: usize) -> Duration {
        let delay = match failure_count {
            // todo: immediately retry a few times first before 30s delay
            0..=10 => 30u64,
            11..=15 => 60u64,
            16..=20 => 120u64,
            21.. => 600u64,
        };
        Duration::from_secs(delay)
    }

    fn unwrap_tor_provider(
        tor_provider: &mut Option<Box<dyn TorProvider>>,
    ) -> Result<&mut Box<dyn TorProvider>> {
        tor_provider.as_mut().context("Missing TorProvider")
    }

    fn handle_tor_events(&mut self) -> Result<()> {
        if let Some(tor_client) = &mut self.tor_provider {
            // handle tor events
            for event in tor_client.update()? {
                if let Err(err) = self.handle_tor_event(event) {
                    log_error!("{err}");
                }
            }
        }
        Ok(())
    }

    fn handle_tor_event(&mut self, event: TorEvent) -> Result<()> {
        match event {
            TorEvent::BootstrapStatus {
                progress,
                tag,
                summary: _,
            } => {
                log_trace!();
                self.callback_queue
                    .push(CallbackData::TorBootstrapStatusChanged { progress, tag });
            }
            TorEvent::BootstrapComplete => {
                self.callback_queue.push(CallbackData::TorBootstrapComplete);
            }
            TorEvent::LogReceived { line } => {
                self.callback_queue
                    .push(CallbackData::TorLogReceived { line });
            }
            TorEvent::OnionServicePublished { service_id: _ } => {
                log_info!("Onion service published");
                // self.callback_queue.push(
                //     CallbackData::HostOnionServiceStateChanged{state: tego_host_onion_service_state::tego_host_onion_service_state_service_published});
            }
            TorEvent::ConnectComplete { handle, stream } => {
                let session_handle = self
                    .connect_handles
                    .remove(&handle)
                    .context("Received ConnectComplete event for unknown ConnectHandle")?;
                if let Some(session) = self.session_map.get_mut(&session_handle) {
                    session.on_connect_complete(handle, stream)?;
                }
            }
            TorEvent::ConnectFailed {
                handle: connection_handle,
                ..
            } => {
                let session_handle = self
                    .connect_handles
                    .remove(&connection_handle)
                    .context("Received ConnectFailed event for unknown ConnectHandle")?;

                if let Some(session) = self.session_map.get_mut(&session_handle) {
                    session.on_connect_failed(connection_handle, &mut self.command_queue)?;
                }
            }
        }
        Ok(())
    }

    fn handle_commands(&mut self) -> Result<()> {
        let mut command_queue = self.command_queue.take();

        while let Some(cmd) = command_queue.peek() {
            if *cmd.start_time() > Instant::now() {
                break;
            }

            let cmd = command_queue
                .pop()
                .expect("command_queue should not be empty");

            if let Err(err) = self.handle_command(cmd.data()) {
                log_error!("{err}");
            }
        }

        // merge remainng commands
        if !command_queue.is_empty() {
            self.command_queue.append(command_queue);
        }

        Ok(())
    }

    fn handle_command(&mut self, cmd: CommandData) -> Result<()> {
        match cmd {
            CommandData::EndEventLoop => self.task_complete = true,
            CommandData::BeginLegacyTorBootstrap {
                legacy_tor_client_config,
            } => {
                log_trace!();
                // todo: surface this error to the user
                let mut tor_provider = LegacyTorClient::new(legacy_tor_client_config)?;
                self.callback_queue
                    .push(CallbackData::TorProviderInitialized {
                        tor_config_type: tego_tor_config_type::tego_tor_config_type_bundled_tor,
                        version: Some(tor_provider.version().to_string()),
                    });
                tor_provider.bootstrap()?;
                self.tor_provider = Some(Box::new(tor_provider));
            }
            CommandData::CancelTorBootstrap => {
                self.tor_provider = None;
            }
            CommandData::BeginSession { profile, result } => {
                //
                // Create Session
                //
                let session_handle = self.next_session_handle;
                self.next_session_handle += 1;

                let mut session = Session::new(session_handle, profile)?;

                let tor_client = Self::unwrap_tor_provider(&mut self.tor_provider)?;

                // TODO: in future starting onions service and connecting to contacts
                // will be directly requested by user setting the Visibility Level in
                // the application

                //
                // Start Onion Services
                //

                // TODO: schedule creating a new onion listener in the future?
                log_info!("Starting onion service");
                session.start_onion_listener(tor_client)?;

                //
                // Start connecting to Allowed and Pending Contacts
                //

                log_info!("Connecting to contacts");
                session.connect_all_known_contacts(&mut self.command_queue);

                //
                // Prepare Contacts list for the UI
                //

                let users: Vec<(UserHandle, UserType, String)> = session
                    .get_users()
                    .iter()
                    .map(|(user_handle, user)| {
                        let pet_name = user.pet_name.clone();
                        (*user_handle, user.user_type, pet_name)
                    })
                    .collect();

                //
                // Trigger callback
                //

                self.callback_queue.push(CallbackData::SessionBegan {
                    session_handle,
                    users,
                });

                self.session_map.insert(session_handle, session);

                result.resolve(Ok(session_handle));
            }
            CommandData::EndSession { session_handle } => {
                unimplemented!();
            }
            CommandData::ConnectContact {
                session_handle,
                user_handle,
                contact_request_message,
            } => {
                // TODO: a new user should be added to the profile *first* before we make an attempt to connect to a contact
                // if !self.users.contains_key(&service_id) {
                //     self.users.insert(
                //         service_id.clone(),
                //         UserData::new(tego_user_type::tego_user_type_pending),
                //     );
                // }

                log_info!("Handle CommandData::ConnectContact");

                let tor_client = Self::unwrap_tor_provider(&mut self.tor_provider)?;

                if let Some(session) = self.session_map.get_mut(&session_handle) {
                    if let Some(connect_handle) = session.connect_contact(
                        user_handle,
                        tor_client,
                        contact_request_message,
                        &mut self.command_queue,
                    )? {
                        self.connect_handles.insert(connect_handle, session_handle);
                    }
                }
            }
            CommandData::SendMessage {
                session_handle,
                user_handle,
                message_text,
                message_id,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    session.send_message(user_handle, message_text)
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                message_id.resolve(result);
            }
            CommandData::AcceptFileTransferRequest {
                session_handle,
                user_handle,
                file_transfer_id,
                dest_path,
                result,
            } => {
                let accept_result = if let Some(session) = self.session_map.get_mut(&session_handle)
                {
                    session.accept_file_transfer_request(user_handle, file_transfer_id, dest_path)
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result.resolve(accept_result);
            }
        }
        Ok(())
        // TODO: migrate command match statement here

        //     CommandData::ForgetUser { service_id, result } => {
        //         let mut handle_forget_user = || -> Result<()> {
        //             // remove from our set of users
        //             if let Some(user_data) = self.users.remove(&service_id) {
        //                 // kill open connection
        //                 if let Some(connection_handle) = user_data.connection_handle {
        //                     self.connections.remove(&connection_handle);
        //                 }
        //             }
        //             // remove from packet handler
        //             self.packet_handler.forget_user(&service_id);

        //             Ok(())
        //         };
        //         result.resolve(handle_forget_user());
        //     }
        //     CommandData::BeginServerHandshake { stream } => {
        //         let handle_begin_server_handshake = || -> Result<()> {
        //             let handle = self.packet_handler.new_incoming_connection()?;

        //             let connection = Connection {
        //                 service_id: None,
        //                 stream,
        //                 read_bytes: Default::default(),
        //                 read_packets: Default::default(),
        //                 write_packets: Default::default(),
        //                 file_downloads: Default::default(),
        //                 file_uploads: Default::default(),
        //             };

        //             log_info!("begin server handshake: {connection:?}");

        //             self.connections.insert(handle, connection);
        //             Ok(())
        //         };
        //         let _ = handle_begin_server_handshake();
        //     }
        //     CommandData::AcknowledgeContactRequest {
        //         service_id,
        //         response,
        //     } => {
        //         let mut replies: Vec<Packet> = Default::default();
        //         use tego_chat_acknowledge::*;
        //         let (result, remove) = match response {
        //             tego_chat_acknowledge_accept => (
        //                 self.packet_handler
        //                     .accept_contact_request(service_id.clone(), &mut replies),
        //                 false,
        //             ),
        //             tego_chat_acknowledge_reject => (
        //                 self.packet_handler
        //                     .reject_contact_request(service_id.clone(), &mut replies),
        //                 true,
        //             ),
        //             tego_chat_acknowledge_block => todo!(),
        //         };

        //         match result {
        //             Ok(connection_handle) => {
        //                 if let Some(connection) = self.connections.get_mut(&connection_handle) {
        //                     connection.write_packets.append(&mut replies);
        //                     if let tego_chat_acknowledge_accept = response {
        //                         self.users.insert(
        //                             service_id,
        //                             UserData::new(tego_user_type::tego_user_type_allowed),
        //                         );
        //                     } else if remove {
        //                         self.to_remove.insert(connection_handle);
        //                     }
        //                 }
        //             }
        //             Err(_err) => log_error!("failure ack'ing contact request: {_err}"),
        //         }
        //     }

        //     CommandData::SendFileTransferRequest {
        //         service_id,
        //         file_path,
        //         result,
        //     } => {
        //         let handle_send_file_transfer_request =
        //             || -> Result<(tego_file_transfer_id, tego_file_size)> {
        //                 // we only deal in absolute paths
        //                 bail_if!(!file_path.is_absolute());

        //                 let file_upload = FileUpload::new(file_path)?;
        //                 let file_name = file_upload.name();
        //                 let file_size = file_upload.size();

        //                 let file_hash = file_upload.hash();

        //                 //construct reply packets
        //                 let mut replies: Vec<Packet> = Vec::with_capacity(1);
        //                 let (connection_handle, file_transfer_handle) =
        //                     self.packet_handler.send_file_transfer_request(
        //                         service_id.clone(),
        //                         file_name.clone(),
        //                         file_size,
        //                         file_hash,
        //                         &mut replies,
        //                     )?;
        //                 let connection = self
        //                     .connections
        //                     .get_mut(&connection_handle)
        //                     .context("missing Connection struct")?;

        //                 // queue packets for writing
        //                 connection.write_packets.append(&mut replies);

        //                 // queue copies of requests to resend in event of reconnect
        //                 let user_data = self
        //                     .users
        //                     .get_mut(&service_id)
        //                     .context(format!("no user data for service id {service_id}"))?;
        //                 let file_transfer_id = user_data.next_message_id();
        //                 user_data
        //                     .file_transfer_id_to_handle
        //                     .insert(file_transfer_id, file_transfer_handle);
        //                 user_data
        //                     .file_transfer_handle_to_id
        //                     .insert(file_transfer_handle, file_transfer_id);
        //                 user_data.queued_messages.push_back(
        //                     UnAckedMessage::FileTransferRequest {
        //                         gui_id: file_transfer_id,
        //                         network_handle: file_transfer_handle,
        //                         file_upload,
        //                     },
        //                 );
        //                 Ok((file_transfer_id, file_size))
        //             };
        //         result.resolve(handle_send_file_transfer_request());
        //     }
        //     CommandData::RejectFileTransferRequest {
        //         service_id,
        //         file_transfer_id,
        //         result,
        //     } => {
        //         let handle_reject_file_transfer_request = || -> Result<()> {
        //             let user_data = self
        //                 .users
        //                 .get_mut(&service_id)
        //                 .context(format!("no user data for service id {service_id}"))?;
        //             let file_transfer_handle = *user_data
        //                 .file_transfer_id_to_handle
        //                 .get(&file_transfer_id)
        //                 .context(format!(
        //                     "no file transfer associated with id {file_transfer_id}"
        //                 ))?;

        //             // construct reply packets
        //             let mut replies: Vec<Packet> = Vec::with_capacity(1);
        //             let connection_handle = self.packet_handler.reject_file_transfer_request(
        //                 &service_id,
        //                 file_transfer_handle,
        //                 &mut replies,
        //             )?;

        //             // remove our file download struct
        //             let connection = self
        //                 .connections
        //                 .get_mut(&connection_handle)
        //                 .context("missing Connection struct")?;
        //             connection
        //                 .file_downloads
        //                 .remove(&file_transfer_handle)
        //                 .context("missing FileDownload struct")?;

        //             // queue packets for writing
        //             connection.write_packets.append(&mut replies);

        //             // fire callback
        //             let direction =
        //                 tego_file_transfer_direction::tego_file_transfer_direction_receiving;
        //             self.callback_queue
        //                 .push(CallbackData::FileTransferComplete {
        //                     user_id: service_id,
        //                     file_transfer_id,
        //                     direction,
        //                     result:
        //                         tego_file_transfer_result::tego_file_transfer_result_rejected,
        //                 });

        //             Ok(())
        //         };

        //         result.resolve(handle_reject_file_transfer_request());
        //     }
        //     CommandData::CancelFileTransfer {
        //         service_id,
        //         file_transfer_id,
        //         result,
        //     } => {
        //         let handle_cancel_file_transfer = || -> Result<()> {
        //             let user_data = self
        //                 .users
        //                 .get_mut(&service_id)
        //                 .context(format!("no user data for service id {service_id}"))?;

        //             let file_transfer_handle = *user_data
        //                 .file_transfer_id_to_handle
        //                 .get(&file_transfer_id)
        //                 .context(format!(
        //                     "no file transfer associated with id {file_transfer_id}"
        //                 ))?;

        //             // construct reply packets
        //             let mut replies: Vec<Packet> = Vec::with_capacity(1);
        //             let connection_handle = self.packet_handler.cancel_file_transfer(
        //                 &service_id,
        //                 file_transfer_handle,
        //                 false,
        //                 &mut replies,
        //             )?;

        //             // remove our file download/upload struct
        //             let connection = self
        //                 .connections
        //                 .get_mut(&connection_handle)
        //                 .context("missing Connection struct")?;

        //             // remove our handle <-> id mappings
        //             let _ = user_data
        //                 .file_transfer_handle_to_id
        //                 .remove(&file_transfer_handle);
        //             let _ = user_data
        //                 .file_transfer_id_to_handle
        //                 .remove(&file_transfer_id);

        //             // remove un'ackd request if present
        //             for i in 0..user_data.queued_messages.len() {
        //                 if let UnAckedMessage::FileTransferRequest { gui_id, .. } =
        //                     user_data.queued_messages[i]
        //                 {
        //                     if gui_id == file_transfer_id {
        //                         let _ = user_data.queued_messages.remove(i);
        //                         break;
        //                     }
        //                 }
        //             }

        //             let direction = if connection
        //                 .file_downloads
        //                 .remove(&file_transfer_handle)
        //                 .is_some()
        //             {
        //                 tego_file_transfer_direction::tego_file_transfer_direction_receiving
        //             } else {
        //                 // it's possible an upload never made it to the file_uploads list
        //                 // if local user cancels before remote user accepts, so missing
        //                 // file_upload is not an error
        //                 let _ = connection.file_uploads.remove(&file_transfer_handle);
        //                 tego_file_transfer_direction::tego_file_transfer_direction_sending
        //             };

        //             // queue packets for writing
        //             connection.write_packets.append(&mut replies);

        //             // fire callback
        //             self.callback_queue
        //                 .push(CallbackData::FileTransferComplete {
        //                     user_id: service_id,
        //                     file_transfer_id,
        //                     direction,
        //                     result:
        //                         tego_file_transfer_result::tego_file_transfer_result_cancelled,
        //                 });

        //             Ok(())
        //         };

        //         result.resolve(handle_cancel_file_transfer());
        //     }
        // }
    }

    fn handle_sessions(&mut self) -> Result<()> {
        // if let Some(tor_provider) = self.tor_provider.as_mut() {
        for session in self.session_map.values_mut() {
            if let Err(err) = session.update_v3(&mut self.command_queue, &mut self.callback_queue) {
                log_error!("{err}")
            }
        }
        // }
        Ok(())
    }

    fn handle_callbacks(&mut self) -> Result<()> {
        let context: *mut tego_context = self.context_handle.into();

        let callbacks = self.callbacks.upgrade().context("callbacks dropped")?;
        let callbacks = callbacks.lock().expect("callbacks mutex poisoned");

        for cd in self.callback_queue.drain(..) {
            callbacks.invoke(context, cd)?;
        }
        Ok(())
    }
}
