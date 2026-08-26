// standard
use std::collections::BTreeMap;
use std::sync::{Mutex, Weak};
use std::time::Instant;

// extern
use anyhow::{anyhow, Context as AnyhowContext, Result};
use rico_profile::v4::profile::UserType;
use tor_interface::legacy_tor_client::*;
use tor_interface::tor_provider::{TorEvent, TorProvider};

// internal crates
use crate::callbacks::*;
use crate::command_queue::*;
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

    fn unwrap_tor_provider(
        tor_provider: &mut Option<Box<dyn TorProvider>>,
    ) -> Result<&mut Box<dyn TorProvider>> {
        tor_provider.as_mut().context("Missing TorProvider")
    }

    fn handle_tor_events(&mut self) -> Result<()> {
        if let Some(tor_client) = &mut self.tor_provider {
            // handle tor events
            for event in tor_client.update()? {
                if let Err(_err) = self.handle_tor_event(event) {
                    log_error!("{_err}");
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

            if let Err(_err) = self.handle_command(cmd.data()) {
                log_error!("{_err}");
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
            CommandData::EndSession {
                session_handle,
                result: result_promise,
            } => {
                let result = if self.session_map.remove(&session_handle).is_some() {
                    self.connect_handles
                        .retain(|&_key, &mut val| val != session_handle);
                    Ok(())
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };

                result_promise.resolve(result);
            }
            CommandData::AddPendingContact {
                session_handle,
                service_id,
                pet_name,
                result: result_promise,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    match session.add_pending_contact(service_id, pet_name) {
                        Ok(user_handle) => {
                            let user_type = UserType::Pending;
                            self.callback_queue.push(CallbackData::UserAdded {
                                session_handle,
                                user_handle,
                                user_type,
                            });
                            Ok(user_handle)
                        }
                        err => err,
                    }
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result_promise.resolve(result);
            }
            CommandData::ConnectContact {
                session_handle,
                user_handle,
                contact_request_message,
            } => {
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
            CommandData::ForgetUser {
                session_handle,
                user_handle,
                result: result_promise,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    match session.forget_user(user_handle) {
                        Ok(()) => {
                            self.callback_queue.push(CallbackData::UserRemoved {
                                session_handle,
                                user_handle,
                            });
                            Ok(())
                        }
                        err => err,
                    }
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result_promise.resolve(result);
            }
            CommandData::SendMessage {
                session_handle,
                user_handle,
                message_text,
                // todo: rename result for consistency
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
                result: result_promise,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    session.accept_file_transfer_request(user_handle, file_transfer_id, dest_path)
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result_promise.resolve(result);
            }
            CommandData::SendFileTransferRequest {
                session_handle,
                user_handle,
                file_path,
                result: result_promise,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    session.send_file_transfer_request(user_handle, file_path)
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result_promise.resolve(result);
            }
            CommandData::RejectFileTransferRequest {
                session_handle,
                user_handle,
                file_transfer_id,
                result: result_promise,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    match session.reject_file_transfer_request(user_handle, file_transfer_id) {
                        Ok(()) => {
                            let direction = tego_file_transfer_direction::tego_file_transfer_direction_receiving;

                            self.callback_queue
                                .push(CallbackData::FileTransferComplete {
                                session_handle,
                                user_handle,
                                file_transfer_id,
                                direction,
                                result:
                                    tego_file_transfer_result::tego_file_transfer_result_rejected,
                            });
                            Ok(())
                        }
                        Err(err) => Err(err),
                    }
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result_promise.resolve(result);
            }
            CommandData::CancelFileTransfer {
                session_handle,
                user_handle,
                file_transfer_id,
                result: result_promise,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    match session.cancel_file_transfer_request(user_handle, file_transfer_id) {
                        Ok(direction) => {
                            self.callback_queue
                                .push(CallbackData::FileTransferComplete {
                                session_handle,
                                user_handle,
                                file_transfer_id,
                                direction,
                                result:
                                    tego_file_transfer_result::tego_file_transfer_result_cancelled,
                            });
                            Ok(())
                        }
                        Err(err) => Err(err),
                    }
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result_promise.resolve(result);
            }
            CommandData::AcknowledgeContactRequest {
                session_handle,
                user_handle,
                response,
                result: result_promise,
            } => {
                let result = if let Some(session) = self.session_map.get_mut(&session_handle) {
                    // todo: these need to trigger a "user type changed" callback
                    match response {
                        tego_chat_acknowledge::tego_chat_acknowledge_accept => {
                            session.accept_contact_request(user_handle)
                        }
                        tego_chat_acknowledge::tego_chat_acknowledge_reject => {
                            session.reject_contact_request(user_handle)
                        }
                        tego_chat_acknowledge::tego_chat_acknowledge_block => todo!(),
                    }
                } else {
                    Err(anyhow!("unknown SessionHandle {session_handle}"))
                };
                result_promise.resolve(result);
            }
        }
        Ok(())
    }

    fn handle_sessions(&mut self) -> Result<()> {
        // if let Some(tor_provider) = self.tor_provider.as_mut() {
        for session in self.session_map.values_mut() {
            if let Err(_err) = session.update_v3(&mut self.command_queue, &mut self.callback_queue) {
                log_error!("{_err}")
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
