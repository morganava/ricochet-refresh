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
use anyhow::{Context as AnyhowContext, Result};
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

    // read_buffer: [u8; Self::READ_BUFFER_SIZE],
    // connections: BTreeMap<ConnectionHandle, Connection>,
    callback_queue: Vec<CallbackData>,
    task_complete: bool,
    // // file reader buffer for uploads
    // file_read_buffer: [u8; Self::FILE_READ_BUFFER_SIZE],
    // users: BTreeMap<V3OnionServiceId, UserData>,
    // // connections to be removed and cleaned up
    // to_remove: BTreeSet<ConnectionHandle>,
}

impl EventLoopTask {
    const READ_BUFFER_SIZE: usize = 64 * 1024;
    const FILE_READ_BUFFER_SIZE: usize = rico_protocol::v3::MAX_FILE_CHUNK_SIZE;

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
            // read_buffer: [0u8; Self::READ_BUFFER_SIZE],
            // connections: Default::default(),
            callback_queue: Default::default(),
            task_complete: false,
            // file_read_buffer: [0u8; Self::FILE_READ_BUFFER_SIZE],
            // users: user_data,
            // to_remove: Default::default(),
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
                for user_handle in session
                    .get_allowed_user_handles()
                    .into_iter()
                    .chain(session.get_pending_user_handles().into_iter())
                {
                    self.command_queue.push(
                        CommandData::ConnectContact {
                            session_handle,
                            user_handle,
                            contact_request_message: None,
                        },
                        Duration::ZERO,
                    );
                }

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
            _ => unimplemented!(),
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

        //     CommandData::SendMessage {
        //         service_id,
        //         message_text,
        //         message_id,
        //     } => {
        //         let handle_send_message = || -> Result<tego_message_id> {
        //             let mut replies: Vec<Packet> = Default::default();
        //             match self.packet_handler.send_message(
        //                 service_id.clone(),
        //                 message_text.clone(),
        //                 None,
        //                 &mut replies,
        //             ) {
        //                 Ok((connection_handle, message_handle)) => {
        //                     let connection = self.connections.get_mut(&connection_handle).context(format!("no connection associated with connection handle {connection_handle}"))?;
        //                     connection.write_packets.append(&mut replies);

        //                     // queue copies of messages to resend in event of reconnect
        //                     let user_data = self
        //                         .users
        //                         .get_mut(&service_id)
        //                         .context(format!("no user data for service id {service_id}"))?;
        //                     let message_id = user_data.next_message_id();
        //                     user_data
        //                         .queued_messages
        //                         .push_back(UnAckedMessage::ChatMessage {
        //                             gui_id: message_id,
        //                             network_handle: message_handle,
        //                             timestamp: std::time::Instant::now(),
        //                             text: message_text,
        //                         });
        //                     Ok(message_id)
        //                 }
        //                 Err(err) => Err(err.into()),
        //             }
        //         };
        //         message_id.resolve(handle_send_message());
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
        //     CommandData::AcceptFileTransferRequest {
        //         service_id,
        //         file_transfer_id,
        //         dest_path,
        //         result,
        //     } => {
        //         let handle_accept_file_transfer_request = || -> Result<()> {
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
        //             let connection_handle = self.packet_handler.accept_file_transfer_request(
        //                 &service_id,
        //                 file_transfer_handle,
        //                 &mut replies,
        //             )?;

        //             // setup file download
        //             let connection = self
        //                 .connections
        //                 .get_mut(&connection_handle)
        //                 .context("missing Connection struct")?;

        //             let file_download = connection
        //                 .file_downloads
        //                 .get_mut(&file_transfer_handle)
        //                 .context("missing FileDownload struct")?;
        //             file_download.start(dest_path)?;

        //             // queue packets for writing
        //             connection.write_packets.append(&mut replies);

        //             Ok(())
        //         };

        //         result.resolve(handle_accept_file_transfer_request());
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

    fn handle_connections(&mut self) -> Result<()> {
        // // TODO: we need some kind of exponential backoff for repeated failures to connect+authenticate
        // // blocked users will currently spam over and over again
        // let mut to_retry: BTreeSet<V3OnionServiceId> = Default::default();

        // // read bytes from each connection
        // for (&handle, connection) in self.connections.iter_mut() {
        //     let stream = &mut connection.stream;

        //     // handle reading
        //     match stream.read(&mut self.read_buffer) {
        //         Err(err) => match err.kind() {
        //             ErrorKind::WouldBlock | ErrorKind::TimedOut => (),
        //             _ => {
        //                 // some error
        //                 log_error!("stream read err: {err:?}");
        //                 self.to_remove.insert(handle);
        //                 if let Some(service_id) = &connection.service_id {
        //                     to_retry.insert(service_id.clone());
        //                 }
        //             }
        //         },
        //         Ok(0) => {
        //             // end of stream
        //             log_error!("stream read err: end of stream");
        //             self.to_remove.insert(handle);
        //             if let Some(service_id) = &connection.service_id {
        //                 to_retry.insert(service_id.clone());
        //             }
        //         }
        //         Ok(size) => {
        //             let read_buffer = &self.read_buffer[..size];
        //             connection
        //                 .read_bytes
        //                 .write_all(read_buffer)
        //                 .expect("read_bytes write failed");
        //         }
        //     }

        //     // handle reading packets
        //     let mut read_bytes = connection.read_bytes.as_slice();
        //     let read_packets = &mut connection.read_packets;
        //     let write_packets = &mut connection.write_packets;

        //     if !read_bytes.is_empty() {
        //         // total handled bytes
        //         let mut trim_count = 0usize;

        //         // parse read bytes into packets
        //         'packet_parse: loop {
        //             match self.packet_handler.try_parse_packet(handle, read_bytes) {
        //                 Ok((packet, size)) => {
        //                     log_packet!("read {packet:?}");
        //                     // move slice up by number of handled bytes
        //                     read_bytes = &read_bytes[size..];
        //                     trim_count += size;
        //                     // save off read bytes for handling
        //                     read_packets.push(packet);
        //                 }
        //                 Err(Error::NeedMoreBytes) => {
        //                     break 'packet_parse;
        //                 }
        //                 Err(_err) => {
        //                     log_error!("parse packet error: {_err:?}, read_bytes: {read_bytes:?}");
        //                     // drop connection
        //                     self.to_remove.insert(handle);
        //                     break 'packet_parse;
        //                 }
        //             }
        //         }

        //         // drop handled bytes off the front
        //         connection.read_bytes.drain(0..trim_count);

        //         // handle packets and queue responses
        //         'packet_handle: for packet in read_packets.drain(..) {
        //             match self
        //                 .packet_handler
        //                 .handle_packet(handle, packet, write_packets)
        //             {
        //                 Ok(Event::IntroductionReceived) => {
        //                     log_info!("introduction received");
        //                 }
        //                 Ok(Event::IntroductionResponseReceived) => {
        //                     log_info!("introduction response received");
        //                 }
        //                 Ok(Event::OpenChannelAuthHiddenServiceReceived) => {
        //                     log_info!("open auth hidden service received");
        //                 }
        //                 Ok(Event::ClientAuthenticated {
        //                     service_id,
        //                     duplicate_connection,
        //                 }) => {
        //                     // todo: handle closed connection
        //                     log_info!("client authenticated: peer: {service_id:?}, duplicate_connection: {duplicate_connection:?}");
        //                     if let Some(user_data) = self.users.get_mut(&service_id) {
        //                         user_data.connection_failures = 0usize;
        //                     }

        //                     connection.service_id = Some(service_id);

        //                     if let Some(connection_handle) = duplicate_connection {
        //                         self.to_remove.insert(connection_handle);
        //                     }
        //                 }
        //                 Ok(Event::BlockedClientAuthenticationAttempted {
        //                     service_id: _service_id,
        //                 }) => {
        //                     log_info!(
        //                         "blocked client attempted authentication, peer: {_service_id}"
        //                     );
        //                     self.to_remove.insert(handle);
        //                 }
        //                 Ok(Event::HostAuthenticated {
        //                     service_id,
        //                     is_known_contact,
        //                     duplicate_connection,
        //                 }) => {
        //                     // when we are connecting to a client we only
        //                     // want to clear our connection failure count
        //                     // if the remote host has short-cut the contact request
        //                     // machinery
        //                     // the remote host may still reject the user in the second
        //                     // case which we want to treat as a conneciton failure
        //                     // for the purposes of reconnect attempt delays to reduce
        //                     // cconnection spamming
        //                     // that is to say, if we were to *always* clear this counter
        //                     // just on authentication success, a blocked user would repeatdly connect over and over and over again which we
        //                     // do not want
        //                     if is_known_contact {
        //                         if let Some(user_data) = self.users.get_mut(&service_id) {
        //                             user_data.connection_failures = 0usize;
        //                         }
        //                     }
        //                     // todo: handle closed connection
        //                     log_info!("host authenticated: peer: {service_id:?}, duplicate_connection: {duplicate_connection:?}");
        //                     if let Some(connection_handle) = duplicate_connection {
        //                         self.to_remove.insert(connection_handle);
        //                     }
        //                 }
        //                 Ok(Event::DuplicateConnectionDropped {
        //                     duplicate_connection,
        //                 }) => {
        //                     // todo: handle closed connection
        //                     log_info!("duplicate connection dropped: {duplicate_connection}");
        //                     self.to_remove.insert(duplicate_connection);
        //                 }
        //                 Ok(Event::ContactRequestReceived {
        //                     service_id,
        //                     nickname: _,
        //                     message_text,
        //                 }) => {
        //                     log_info!("contact request received, peer: {service_id:?}, message_text: \"{message_text}\"");
        //                     self.callback_queue.push(CallbackData::ChatRequestReceived {
        //                         service_id,
        //                         message: message_text,
        //                     });
        //                 }
        //                 Ok(Event::ContactRequestResultPending {
        //                     service_id: _service_id,
        //                 }) => {
        //                     log_info!("contact request result pending, peer: {_service_id:?}");
        //                 }
        //                 Ok(Event::ContactRequestResultAccepted { service_id }) => {
        //                     log_info!("contact request result accepted, peer: {service_id:?}");
        //                     self.callback_queue
        //                         .push(CallbackData::ChatRequestResponseReceived {
        //                             service_id,
        //                             accepted_request: true,
        //                         });
        //                 }
        //                 Ok(Event::ContactRequestResultRejected { service_id }) => {
        //                     log_info!("contact request result rejected, peer: {service_id:?}");
        //                     self.to_remove.insert(handle);
        //                     if let Some(user_data) = self.users.get_mut(&service_id) {
        //                         user_data.user_type = tego_user_type::tego_user_type_rejected;
        //                     }
        //                     self.callback_queue
        //                         .push(CallbackData::ChatRequestResponseReceived {
        //                             service_id,
        //                             accepted_request: false,
        //                         });
        //                 }
        //                 Ok(Event::IncomingChatChannelOpened {
        //                     service_id: _service_id,
        //                 }) => {
        //                     log_info!("incoming chat channel opened, peer: {_service_id:?}");
        //                 }
        //                 Ok(Event::IncomingFileTransferChannelOpened {
        //                     service_id: _service_id,
        //                 }) => {
        //                     log_info!(
        //                         "incoming file transfer channel opened, peer: {_service_id:?}"
        //                     );
        //                 }
        //                 Ok(Event::OutgoingAuthHiddenServiceChannelOpened {
        //                     service_id: _service_id,
        //                 }) => {
        //                     log_info!("outgoing auth hidden service channel opened, peer: {_service_id:?}");
        //                 }
        //                 Ok(Event::OutgoingChatChannelOpened { service_id }) => {
        //                     log_info!("outgoing chat channel opened, peer: {service_id:?}");

        //                     // send queued messages
        //                     if let Some(user_data) = self.users.get_mut(&service_id) {
        //                         user_data.connection_failures = 0usize;

        //                         log_info!("re-sending un-acked messages");
        //                         if !user_data.queued_messages.is_empty() {
        //                             for message in user_data.queued_messages.iter_mut() {
        //                                 if let UnAckedMessage::ChatMessage {
        //                                     gui_id: _,
        //                                     network_handle,
        //                                     timestamp,
        //                                     text,
        //                                 } = message
        //                                 {
        //                                     match self.packet_handler.send_message(
        //                                         service_id.clone(),
        //                                         text.clone(),
        //                                         Some(
        //                                             std::time::Instant::now()
        //                                                 .duration_since(*timestamp),
        //                                         ),
        //                                         write_packets,
        //                                     ) {
        //                                         Ok((_connection_handle, message_handle)) => {
        //                                             // update the queued message with new message handle
        //                                             *network_handle = message_handle;
        //                                         }
        //                                         Err(_err) => {
        //                                             log_error!(
        //                                                 "error re-sending queued message: {_err}"
        //                                             )
        //                                         }
        //                                     }
        //                                 }
        //                             }
        //                         }

        //                         self.callback_queue.push(CallbackData::UserStatusChanged {
        //                             service_id,
        //                             status: tego_user_status::tego_user_status_online,
        //                         });
        //                     } else {
        //                         log_error!("no user data for service id: {service_id}");
        //                     }
        //                 }
        //                 Ok(Event::OutgoingFileTransferChannelOpened { service_id }) => {
        //                     log_info!(
        //                         "outgoing file transfer channel opened, peer: {service_id:?}"
        //                     );

        //                     // send queued messages
        //                     if let Some(user_data) = self.users.get_mut(&service_id) {
        //                         log_info!("re-sending un-acked file transfer requests");
        //                         if !user_data.queued_messages.is_empty() {
        //                             for message in user_data.queued_messages.iter_mut() {
        //                                 if let UnAckedMessage::FileTransferRequest {
        //                                     gui_id: file_transfer_id,
        //                                     network_handle,
        //                                     file_upload,
        //                                 } = message
        //                                 {
        //                                     let file_transfer_id = *file_transfer_id;
        //                                     match self.packet_handler.send_file_transfer_request(
        //                                         service_id.clone(),
        //                                         file_upload.name(),
        //                                         file_upload.size(),
        //                                         file_upload.hash(),
        //                                         write_packets,
        //                                     ) {
        //                                         Ok((_connection_handle, file_transfer_handle)) => {
        //                                             // update the queued upload request with new file transfer handle handle
        //                                             *network_handle = file_transfer_handle;
        //                                             if let Some(old_file_transfer_handle) =
        //                                                 user_data.file_transfer_id_to_handle.insert(
        //                                                     file_transfer_id,
        //                                                     file_transfer_handle,
        //                                                 )
        //                                             {
        //                                                 // remap ids and handles
        //                                                 user_data
        //                                                     .file_transfer_handle_to_id
        //                                                     .remove(&old_file_transfer_handle);
        //                                                 user_data
        //                                                     .file_transfer_handle_to_id
        //                                                     .insert(
        //                                                         file_transfer_handle,
        //                                                         file_transfer_id,
        //                                                     );
        //                                             }
        //                                         }
        //                                         Err(_err) => {
        //                                             log_error!(
        //                                                 "error re-sending queued file transfer request: {_err}"
        //                                             )
        //                                         }
        //                                     }
        //                                 }
        //                             }
        //                         }
        //                     }
        //                 }
        //                 Ok(Event::ChatMessageReceived {
        //                     service_id,
        //                     message_text,
        //                     message_handle: _message_handle,
        //                     time_delta,
        //                 }) => {
        //                     log_info!("chat message receved, peer: {service_id:?}, message: \"{message_text}, message_handle: {_message_handle:?}, time_delta: {time_delta:?}");
        //                     let handle_chat_message_received = || -> Result<()> {
        //                         let now = std::time::SystemTime::now();
        //                         let timestamp = now.checked_sub(time_delta).unwrap_or(now);
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let message_id = user_data.next_message_id();
        //                         let message = message_text;
        //                         self.callback_queue.push(CallbackData::MessageReceived {
        //                             service_id,
        //                             timestamp,
        //                             message_id,
        //                             message,
        //                         });
        //                         Ok(())
        //                     };
        //                     if let Err(_err) = handle_chat_message_received() {
        //                         log_error!("error receiving chat message: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::ChatAcknowledgeReceived {
        //                     service_id,
        //                     message_handle,
        //                     accepted,
        //                 }) => {
        //                     log_info!("chat ack received, peer: {service_id:?}, message_handle: {message_handle:?}, accepted: {accepted}");

        //                     // find message in queue and remove as it has been acked
        //                     if let Some(user_data) = self.users.get_mut(&service_id) {
        //                         let queued_messages = &mut user_data.queued_messages;
        //                         for i in 0..queued_messages.len() {
        //                             if let UnAckedMessage::ChatMessage {
        //                                 gui_id,
        //                                 network_handle,
        //                                 timestamp: _,
        //                                 text: _,
        //                             } = &mut queued_messages[i]
        //                             {
        //                                 if *network_handle == message_handle {
        //                                     let message_id = *gui_id;
        //                                     let _ = queued_messages.remove(i);
        //                                     self.callback_queue.push(
        //                                         CallbackData::MessageAcknowledged {
        //                                             service_id,
        //                                             message_id,
        //                                             accepted,
        //                                         },
        //                                     );
        //                                     break;
        //                                 }
        //                             }
        //                         }
        //                     } else {
        //                         log_error!("received chat ack for unknown user {service_id}");
        //                     }
        //                 }
        //                 Ok(Event::FileTransferRequestReceived {
        //                     service_id,
        //                     file_transfer_handle,
        //                     file_name,
        //                     file_size,
        //                 }) => {
        //                     log_info!("file transfer request received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, file_name: {file_name}, file_size: {file_size}");

        //                     let user_data = self
        //                         .users
        //                         .get_mut(&service_id)
        //                         .context(format!("no user data for service id {service_id}"))?;

        //                     let file_transfer_id = user_data.next_message_id();
        //                     user_data
        //                         .file_transfer_id_to_handle
        //                         .insert(file_transfer_id, file_transfer_handle);
        //                     user_data
        //                         .file_transfer_handle_to_id
        //                         .insert(file_transfer_handle, file_transfer_id);

        //                     // the protocol handler *shouldn't* be returning duplicate handles but we get them
        //                     // from the other party so really we have no control here :(
        //                     if connection
        //                         .file_downloads
        //                         .contains_key(&file_transfer_handle)
        //                     {
        //                         // if we have a collision, just cancel the old one
        //                         self.callback_queue.push(CallbackData::FileTransferComplete{
        //                             user_id: service_id.clone(),
        //                             file_transfer_id,
        //                             direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
        //                             result: tego_file_transfer_result::tego_file_transfer_result_cancelled
        //                         });
        //                     }

        //                     let file_download: FileDownload = FileDownload::new(file_size);
        //                     connection
        //                         .file_downloads
        //                         .insert(file_transfer_handle, file_download);

        //                     self.callback_queue
        //                         .push(CallbackData::FileTransferRequestReceived {
        //                             sender: service_id,
        //                             file_transfer_id,
        //                             file_name,
        //                             file_size,
        //                         });
        //                 }
        //                 Ok(Event::FileTransferRequestAcknowledgeReceived {
        //                     service_id,
        //                     file_transfer_handle,
        //                     accepted,
        //                 }) => {
        //                     log_info!("file transfer request ack received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, accepted: {accepted}");

        //                     // find file transfer request in queue and removeas it has been acked
        //                     if let Some(user_data) = self.users.get_mut(&service_id) {
        //                         let queued_messages = &mut user_data.queued_messages;
        //                         for i in 0..queued_messages.len() {
        //                             if let UnAckedMessage::FileTransferRequest {
        //                                 gui_id: _,
        //                                 network_handle,
        //                                 file_upload: _,
        //                             } = &mut queued_messages[i]
        //                             {
        //                                 if *network_handle == file_transfer_handle {
        //                                     if let Some(UnAckedMessage::FileTransferRequest {
        //                                         gui_id,
        //                                         network_handle,
        //                                         file_upload,
        //                                     }) = queued_messages.remove(i)
        //                                     {
        //                                         // save off file upload record
        //                                         connection
        //                                             .file_uploads
        //                                             .insert(network_handle, file_upload);

        //                                         let file_transfer_id = gui_id;
        //                                         self.callback_queue.push(
        //                                             CallbackData::FileTransferRequestAcknowledged {
        //                                                 service_id,
        //                                                 file_transfer_id,
        //                                                 accepted,
        //                                             },
        //                                         );
        //                                         break;
        //                                     }
        //                                 }
        //                             }
        //                         }
        //                     } else {
        //                         log_error!("received file transfer request ack for unknown user {service_id}");
        //                     }
        //                 }
        //                 Ok(Event::FileTransferRequestAccepted {
        //                     service_id,
        //                     file_transfer_handle,
        //                 }) => {
        //                     log_info!("file transfer request accepted, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}");

        //                     let handle_file_transfer_request_accepted = || -> Result<()> {
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let file_transfer_id = *user_data.file_transfer_handle_to_id.get(&file_transfer_handle).context(format!("no file transfer associated with handle {file_transfer_handle:?}"))?;

        //                         let file_upload = connection
        //                             .file_uploads
        //                             .get_mut(&file_transfer_handle)
        //                             .context(format!("no file upload associated with file transfer handle {file_transfer_handle:?}"))?;

        //                         // begin sending chunks
        //                         let bytes_read = file_upload.read(&mut self.file_read_buffer)?;
        //                         let chunk_data: Vec<u8> =
        //                             self.file_read_buffer[..bytes_read].to_vec();

        //                         self.packet_handler.send_file_chunk(
        //                             &service_id,
        //                             file_transfer_handle,
        //                             chunk_data,
        //                             write_packets,
        //                         )?;

        //                         // trigger callbacks
        //                         self.callback_queue
        //                             .push(CallbackData::FileTransferRequestResponseReceived {
        //                             service_id: service_id.clone(),
        //                             file_transfer_id,
        //                             response:
        //                                 tego_file_transfer_response::tego_file_transfer_response_accept,
        //                         });
        //                         self.callback_queue.push(CallbackData::FileTransferProgress{
        //                             user_id: service_id,
        //                             file_transfer_id,
        //                             direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
        //                             bytes_complete: file_upload.bytes_sent,
        //                             bytes_total: file_upload.size,
        //                         });

        //                         file_upload.bytes_sent += bytes_read as u64;

        //                         Ok(())
        //                     };
        //                     if let Err(_err) = handle_file_transfer_request_accepted() {
        //                         log_error!("error handling file transfer request accepted: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::FileTransferRequestRejected {
        //                     service_id,
        //                     file_transfer_handle,
        //                 }) => {
        //                     log_info!("file transfer request rejected, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}");

        //                     let handle_file_transfer_request_rejected = || -> Result<()> {
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let file_transfer_id = *user_data.file_transfer_handle_to_id.get(&file_transfer_handle).context(format!("no file transfer associated with handle {file_transfer_handle:?}"))?;

        //                         self.callback_queue
        //                             .push(CallbackData::FileTransferRequestResponseReceived {
        //                             service_id,
        //                             file_transfer_id,
        //                             response:
        //                                 tego_file_transfer_response::tego_file_transfer_response_reject,
        //                         });
        //                         Ok(())
        //                     };
        //                     if let Err(_err) = handle_file_transfer_request_rejected() {
        //                         log_error!("error handling file transfer request rejected: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::FileChunkReceived {
        //                     service_id,
        //                     file_transfer_handle,
        //                     data,
        //                     last_chunk,
        //                     hash_matches,
        //                 }) => {
        //                     log_info!("file chunk received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, data: [u8; {}], last_chunk: {last_chunk}, hash_matches: {hash_matches:?}", data.len());

        //                     let handle_file_chunk_received = || -> Result<()> {
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let file_transfer_id = *user_data.file_transfer_handle_to_id.get(&file_transfer_handle).context(format!("no file transfer associated with handle {file_transfer_handle:?}"))?;

        //                         // these two last_chunk checks get us a Option<FileDownload&>
        //                         // in both cases where we need to remove it and where we need to modify
        //                         // it in-place
        //                         let file_download = connection.file_downloads.get_mut(&file_transfer_handle).context(format!("no file download associated with handle {file_transfer_handle:?}"))?;

        //                         // write chunk to disk
        //                         match file_download.write(&data) {
        //                             Ok(()) => {
        //                                 self.callback_queue.push(CallbackData::FileTransferProgress{
        //                                     user_id: service_id.clone(),
        //                                     file_transfer_id,
        //                                     direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
        //                                     bytes_complete: file_download.bytes_written,
        //                                     bytes_total: file_download.expected_size,
        //                                 });
        //                                 // handle completed donwload
        //                                 if last_chunk {
        //                                     match hash_matches {
        //                                         Some(true) => {
        //                                             let result = match file_download.finalize() {
        //                                                 Ok(()) => tego_file_transfer_result::tego_file_transfer_result_success,
        //                                                 Err(_) => tego_file_transfer_result::tego_file_transfer_result_filesystem_error,
        //                                             };
        //                                             self.callback_queue.push(CallbackData::FileTransferComplete{
        //                                                 user_id: service_id,
        //                                                 file_transfer_id,
        //                                                 direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
        //                                                 result,
        //                                             });
        //                                         }
        //                                         Some(false) => {
        //                                             self.callback_queue.push(CallbackData::FileTransferComplete{
        //                                                 user_id: service_id,
        //                                                 file_transfer_id,
        //                                                 direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
        //                                                 result: tego_file_transfer_result::tego_file_transfer_result_bad_hash,
        //                                             });
        //                                         }
        //                                         None => unreachable!(),
        //                                     }
        //                                     connection.file_downloads.remove(&file_transfer_handle);
        //                                 }
        //                                 Ok(())
        //                             }
        //                             Err(err) => {
        //                                 self.callback_queue.push(CallbackData::FileTransferComplete{
        //                                     user_id: service_id,
        //                                     file_transfer_id,
        //                                     direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
        //                                     result: tego_file_transfer_result::tego_file_transfer_result_filesystem_error,
        //                                 });
        //                                 Err(err)
        //                             }
        //                         }
        //                     };
        //                     if let Err(_err) = handle_file_chunk_received() {
        //                         log_error!("error handling file chunk received: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::FileChunkAckReceived {
        //                     service_id,
        //                     file_transfer_handle,
        //                     offset,
        //                 }) => {
        //                     log_info!("file chunk ack received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, offset: {offset}");

        //                     let mut handle_file_chunk_ack_received = || -> Result<()> {
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let file_transfer_id = *user_data.file_transfer_handle_to_id.get(&file_transfer_handle).context(format!("no file transfer associated with handle {file_transfer_handle:?}"))?;

        //                         let file_upload = connection.file_uploads.get_mut(&file_transfer_handle).context(format!("no file upload associated with handle {file_transfer_handle:?}"))?;

        //                         self.callback_queue.push(CallbackData::FileTransferProgress{
        //                             user_id: service_id.clone(),
        //                             file_transfer_id,
        //                             direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
        //                             bytes_complete: file_upload.bytes_sent,
        //                             bytes_total: file_upload.size,
        //                         });

        //                         // todo: better error handling
        //                         assert_eq!(file_upload.bytes_sent, offset);

        //                         // send next chunk if there is more data to sesnd
        //                         if file_upload.bytes_sent < file_upload.size {
        //                             match file_upload.read(&mut self.file_read_buffer) {
        //                                 Ok(bytes_read) => {
        //                                     let chunk_data: Vec<u8> =
        //                                         self.file_read_buffer[..bytes_read].to_vec();

        //                                     let _ = self.packet_handler.send_file_chunk(
        //                                         &service_id,
        //                                         file_transfer_handle,
        //                                         chunk_data,
        //                                         write_packets,
        //                                     )?;
        //                                     file_upload.bytes_sent += bytes_read as u64;
        //                                 }
        //                                 Err(_err) => {
        //                                     log_error!("failed to read next file chunk for fille transfer {file_transfer_handle:?}: {_err}");
        //                                     // disk error, terminate this file transfer
        //                                     self.packet_handler.cancel_file_transfer(
        //                                         &service_id,
        //                                         file_transfer_handle,
        //                                         true,
        //                                         write_packets,
        //                                     )?;
        //                                 }
        //                             };
        //                         }
        //                         Ok(())
        //                     };
        //                     if let Err(_err) = handle_file_chunk_ack_received() {
        //                         log_error!("error handling file chunk ack recieved: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::FileTransferSucceeded {
        //                     service_id,
        //                     file_transfer_handle,
        //                 }) => {
        //                     log_info!("file transfer succeeded, peer: {service_id}, file_transfer_handle: {file_transfer_handle:?}");

        //                     let handle_file_transfer_succeeded = || -> Result<()> {
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let file_transfer_id = *user_data.file_transfer_handle_to_id.get(&file_transfer_handle).context(format!("no file transfer associated with handle {file_transfer_handle:?}"))?;
        //                         self.callback_queue.push(CallbackData::FileTransferComplete{
        //                             user_id: service_id,
        //                             file_transfer_id,
        //                             direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
        //                             result: tego_file_transfer_result::tego_file_transfer_result_success
        //                         });
        //                         Ok(())
        //                     };
        //                     if let Err(_err) = handle_file_transfer_succeeded() {
        //                         log_error!("error handling file transfer succeeded: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::FileTransferFailed {
        //                     service_id,
        //                     file_transfer_handle,
        //                 }) => {
        //                     log_info!("file transfer failed, peer: {service_id}, file_transfer_handle: {file_transfer_handle:?}");

        //                     let handle_file_transfer_failed = || -> Result<()> {
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let file_transfer_id = *user_data.file_transfer_handle_to_id.get(&file_transfer_handle).context(format!("no file transfer associated with handle {file_transfer_handle:?}"))?;
        //                         self.callback_queue.push(CallbackData::FileTransferComplete{
        //                             user_id: service_id,
        //                             file_transfer_id,
        //                             direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
        //                             result: tego_file_transfer_result::tego_file_transfer_result_failure
        //                         });
        //                         Ok(())
        //                     };
        //                     if let Err(_err) = handle_file_transfer_failed() {
        //                         log_error!("error handling file transfer failed: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::FileTransferCancelled {
        //                     service_id,
        //                     file_transfer_handle,
        //                 }) => {
        //                     log_info!("file transfer cancelled, peer: {service_id}, file_transfer_handle: {file_transfer_handle:?}");

        //                     let handle_file_transfer_cancelled = || -> Result<()> {
        //                         let user_data = self
        //                             .users
        //                             .get_mut(&service_id)
        //                             .context(format!("no user data for service id {service_id}"))?;
        //                         let file_transfer_id = *user_data.file_transfer_handle_to_id.get(&file_transfer_handle).context(format!("no file transfer associated with handle {file_transfer_handle:?}"))?;
        //                         self.callback_queue.push(CallbackData::FileTransferComplete{
        //                             user_id: service_id,
        //                             file_transfer_id,
        //                             direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
        //                             result: tego_file_transfer_result::tego_file_transfer_result_cancelled
        //                         });
        //                         Ok(())
        //                     };
        //                     if let Err(_err) = handle_file_transfer_cancelled() {
        //                         log_error!("error handling file transfer cancelled: {_err}");
        //                     }
        //                 }
        //                 Ok(Event::ChannelClosed { id: _id }) => {
        //                     log_info!("channel closed: {_id}");
        //                 }
        //                 // errors
        //                 Ok(Event::ProtocolFailure { message: _message }) => {
        //                     log_info!("non-fatal protocol failure: {_message}");
        //                 }
        //                 Ok(Event::FatalProtocolFailure { message: _message }) => {
        //                     log_error!("fatal protocol error, removing connection: {_message}");
        //                     self.to_remove.insert(handle);
        //                     break 'packet_handle;
        //                 }
        //                 Err(err) => panic!("error: {err:?}"),
        //             }
        //         }
        //     }

        //     // write packets to stream
        //     if !write_packets.is_empty() {
        //         // serialise out packets to bytes
        //         let mut write_bytes: Vec<u8> = Default::default();
        //         for packet in write_packets.drain(..) {
        //             log_packet!("write {packet:?}");
        //             packet
        //                 .write_to_vec(&mut write_bytes)
        //                 .expect("packet failed");
        //         }

        //         // send bytes
        //         let stream = &mut connection.stream;
        //         if stream.write(write_bytes.as_slice()).is_err() {
        //             self.to_remove.insert(handle);
        //             if let Some(service_id) = &connection.service_id {
        //                 to_retry.insert(service_id.clone());
        //             }
        //         }
        //     }
        // }

        // // drop our dead connections
        // for handle in self.to_remove.iter() {
        //     self.packet_handler.remove_connection(handle);
        //     let connection = self
        //         .connections
        //         .remove(handle)
        //         .expect("removed non-existing connection");
        //     // signal user offline status to frontend
        //     if let Some(service_id) = connection.service_id {
        //         log_info!("dropping connection; handle: {handle}, service_id: {service_id}");
        //         // signal user offline status if there is not another connection
        //         if !self.packet_handler.has_verified_connection(&service_id) {
        //             if let Some(user_data) = self.users.get(&service_id) {
        //                 if matches!(
        //                     user_data.user_type,
        //                     tego_user_type::tego_user_type_allowed
        //                         | tego_user_type::tego_user_type_pending
        //                 ) {
        //                     use crate::ffi::tego_user_status::tego_user_status_offline;
        //                     let status = tego_user_status_offline;
        //                     self.callback_queue
        //                         .push(CallbackData::UserStatusChanged { service_id, status });
        //                 }
        //             }
        //         }
        //     } else {
        //         log_info!("dropping connection; handle: {handle}");
        //     }
        // }
        // self.to_remove.clear();

        // // initiate connection retries for connections which had IO failures
        // for service_id in to_retry.iter() {
        //     if let Some(user_data) = self.users.get_mut(service_id) {
        //         if matches!(
        //             user_data.user_type,
        //             tego_user_type::tego_user_type_allowed | tego_user_type::tego_user_type_pending
        //         ) {
        //             user_data.connection_failures += 1usize;

        //             let command_data = CommandData::ConnectContact {
        //                 service_id: service_id.clone(),
        //                 contact_request_message: None,
        //             };
        //             let delay = Self::retry_delay(user_data.connection_failures);

        //             log_info!("retry connecting to {service_id} in {delay:?}");

        //             self.command_queue.push(command_data, delay);
        //         }
        //     }
        // }

        Ok(())
    }

    fn handle_sessions(&mut self) -> Result<()> {
        if let Some(tor_provider) = self.tor_provider.as_mut() {
            for session in self.session_map.values_mut() {
                if let Err(err) = session.update_v3(tor_provider, &mut self.command_queue) {
                    log_error!("{err}")
                }
            }
        }
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

// message enum is used for enqueueing chat and file uploads in case
// the remote user goes offline
#[derive(Debug)]
enum UnAckedMessage {
    ChatMessage {
        gui_id: tego_message_id,
        network_handle: MessageHandle,
        timestamp: std::time::Instant,
        text: rico_protocol::v3::message::chat_channel::MessageText,
    },
    FileTransferRequest {
        gui_id: tego_file_transfer_id,
        network_handle: FileTransferHandle,
        file_upload: FileUpload,
    },
}

// TODO kill this/fully move to session module
struct UserData {
    user_type: tego_user_type,
    connection_handle: Option<ConnectionHandle>,
    connection_failures: usize,
    queued_messages: VecDeque<UnAckedMessage>,
    next_message_id: u64,

    file_transfer_handle_to_id: BTreeMap<FileTransferHandle, tego_file_transfer_id>,
    file_transfer_id_to_handle: BTreeMap<tego_file_transfer_id, FileTransferHandle>,
}

impl UserData {
    fn new(user_type: tego_user_type) -> Self {
        Self {
            user_type,
            connection_handle: None,
            connection_failures: 0usize,
            queued_messages: Default::default(),
            next_message_id: 0u64,
            file_transfer_handle_to_id: Default::default(),
            file_transfer_id_to_handle: Default::default(),
        }
    }

    fn next_message_id(&mut self) -> u64 {
        let result = self.next_message_id;
        self.next_message_id += 1;
        result
    }
}

#[derive(Debug)]
struct FileDownload {
    // number of bytes written
    bytes_written: u64,
    // total expected file-size
    expected_size: u64,
    // final destination for the in-progress file transfer
    final_destination: PathBuf,
    // the temporary destination we will write the file to before renaming
    temp_destination: PathBuf,
    // destination to write the received data
    file: Option<File>,
}

impl FileDownload {
    pub fn new(expected_size: u64) -> Self {
        Self {
            bytes_written: 0u64,
            expected_size,
            final_destination: Default::default(),
            temp_destination: Default::default(),
            file: None,
        }
    }

    // start the download
    pub fn start(&mut self, final_destination: PathBuf) -> Result<()> {
        bail_if!(!final_destination.is_absolute());

        // create a temporary file location of the form ".filename.part"
        let mut temp_destination_filename = OsString::from(".");
        temp_destination_filename.push(
            final_destination
                .file_name()
                .context("final_destination has no filename")?,
        );
        temp_destination_filename.push(".part");

        let mut temp_destination = final_destination.clone();
        bail_if!(!temp_destination.pop());
        temp_destination.push(temp_destination_filename);

        // create our file
        let file = Some(File::create(&temp_destination)?);

        self.final_destination = final_destination;
        self.temp_destination = temp_destination;
        self.file = file;

        Ok(())
    }

    // returns true when we're done writing, false if we need more bytes
    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        let file = self.file.as_mut().context("file is None")?;
        file.write_all(bytes)?;
        self.bytes_written += bytes.len() as u64;

        Ok(())
    }

    pub fn finalize(&mut self) -> Result<()> {
        // close the temporary file
        self.file = None;

        // move temp file to final destination
        std::fs::rename(&self.temp_destination, &self.final_destination)?;

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct FileUpload {
    file: File,
    // the name of the file
    name: String,
    // the numebr of bytes we have uploaded
    bytes_sent: u64,
    // the size of the file
    size: u64,
    // the hash of the file
    hash: rico_protocol::v3::file_hasher::FileHash,
}

impl FileUpload {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let name: String = file_path
            .file_name()
            .context("path contains no file name")?
            .to_str()
            .context("file name not valid utf8")?
            .to_string();

        // open file for reading
        let mut file = std::fs::OpenOptions::new().read(true).open(file_path)?;

        let bytes_sent = 0u64;

        // get our file's size
        let size = file.metadata()?.len();

        // calculate the file's hash
        let mut hasher: FileHasher = Default::default();
        const HASH_BUFFER_SIZE: usize = 64usize * 1024usize;
        let mut buffer = [0u8; HASH_BUFFER_SIZE];

        let mut bytes_read = 0u64;
        while bytes_read != size {
            let n = file.read(&mut buffer)?;
            hasher.update(&buffer[..n]);
            bytes_read += n as u64;
        }
        let hash = hasher.finalize();

        // reset the file read stream to beginning
        file.rewind()?;

        Ok(Self {
            file,
            name,
            bytes_sent,
            size,
            hash,
        })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.file.read(buf)?)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn hash(&self) -> rico_protocol::v3::file_hasher::FileHash {
        self.hash
    }
}
