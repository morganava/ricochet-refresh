// standard
use std::path::PathBuf;
#[cfg(feature = "pluggable-transports")]
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// extern
use anyhow::{Context as AnyhowContext, Result};
#[cfg(feature = "pluggable-transports")]
use pt_config::pt_config::*;
use rico_profile::v4::profile::Profile;
#[cfg(feature = "pluggable-transports")]
use rico_settings::common::{BridgeConfig, BuiltInBridge};
use rico_settings::v4::settings::TorConfig;
#[cfg(feature = "pluggable-transports")]
use tor_interface::censorship_circumvention::{BridgeLine, PluggableTransportConfig};
use tor_interface::legacy_tor_client::LegacyTorClientConfig;
use tor_interface::tor_crypto::V3OnionServiceId;

// internal crates
use crate::callbacks::*;
use crate::command_queue::*;
use crate::event_loop_task::*;
use crate::ffi::*;
use crate::macros::*;
use crate::promise::Promise;
use crate::session::{SessionHandle, UserHandle};

#[derive(Default)]
pub(crate) struct Context {
    // callback struct
    pub callbacks: Arc<Mutex<Callbacks>>,
    // command queue
    command_queue: CommandQueue,
    // event loop thread handle
    event_loop_thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl Context {
    pub fn start_event_loop(&mut self, context_handle: TegoContextHandle) {
        let callbacks = Arc::downgrade(&self.callbacks);
        let command_queue = self.command_queue.downgrade();

        let task = EventLoopTask::new(context_handle, callbacks, command_queue);

        self.event_loop_thread_handle = Some(
            std::thread::Builder::new()
                .name("event-loop".to_string())
                .spawn(move || {
                    // start event loop
                    log_trace!();
                    if let Err(_err) = task.run() {
                        log_error!("{_err:?}");
                        log_flush!();
                        panic!();
                    }
                })
                .expect("failed to start event-loop thread"),
        );
    }

    pub fn begin_bootstrap(&mut self, tor_config: TorConfig) -> Result<()> {
        match tor_config {
            #[cfg(feature = "bundled-tor")]
            TorConfig::BundledTor {
                bridge_config : _bridge_config,
                proxy_config,
                firewall_config,
            } => {
                let tor_bin_path = Self::tor_bin_path()?;
                let data_directory = Self::data_directory();

                let proxy_settings = proxy_config;
                let allowed_ports = firewall_config.map(|conf| conf.allowed_ports().clone());
                #[cfg(feature = "pluggable-transports")]
                let (pluggable_transports, bridge_lines) = (
                    Some(Self::pluggable_transports()?),
                    match _bridge_config {
                        Some(BridgeConfig::BuiltIn(builtin)) => {
                            let bridge_lines: &[&str] = match builtin {
                                BuiltInBridge::Obfs4 => &BUILTIN_OBFS4_BRIDGE_LINES,
                                BuiltInBridge::Meek => &BUILTIN_MEEK_BRIDGE_LINES,
                                BuiltInBridge::Snowflake => &BUILTIN_SNOWFLAKE_BRIDGE_LINES,
                            };
                            Some(
                                bridge_lines
                                    .iter()
                                    .map(|bridge| BridgeLine::from_str(bridge).unwrap())
                                    .collect(),
                            )
                        }
                        Some(BridgeConfig::Custom(first, bridge_lines)) => {
                            let mut bridge_lines = bridge_lines;
                            bridge_lines.insert(0, first);
                            Some(bridge_lines)
                        }
                        None => None,
                    }
                );
                #[cfg(not(feature = "pluggable-transports"))]
                let (pluggable_transports, bridge_lines) = (None, None);

                let legacy_tor_client_config = LegacyTorClientConfig::BundledTor {
                    tor_bin_path,
                    data_directory,
                    proxy_settings,
                    allowed_ports,
                    pluggable_transports,
                    bridge_lines,
                };
                self.push_command(CommandData::BeginLegacyTorBootstrap {
                    legacy_tor_client_config,
                });
            }
        }

        Ok(())
    }

    pub fn cancel_bootstrap(&mut self) -> Result<()> {
        self.push_command(CommandData::CancelTorBootstrap);
        Ok(())
    }

    fn push_command(&self, data: CommandData) {
        self.push_command_ex(data, Duration::ZERO);
    }

    fn push_command_ex(&self, data: CommandData, delay: Duration) {
        self.command_queue.push(data, delay);
    }

    pub fn begin_session(&mut self, profile: Profile) -> Result<SessionHandle> {
        let result: Promise<Result<SessionHandle>> = Default::default();
        let result_future = result.get_future();

        self.push_command(CommandData::BeginSession { profile, result });

        result_future.wait()
    }

    pub fn end_session(&mut self, session_handle: SessionHandle) -> Result<()> {
        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();

        self.push_command(CommandData::EndSession {
            session_handle,
            result,
        });

        result_future.wait()
    }

    pub fn forget_user(
        &mut self,
        session_handle: SessionHandle,
        user_handle: UserHandle,
    ) -> Result<()> {
        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();
        self.push_command(CommandData::ForgetUser {
            session_handle,
            user_handle,
            result,
        });

        result_future.wait()
    }

    pub fn send_contact_request(
        &self,
        session_handle: SessionHandle,
        service_id: V3OnionServiceId,
        pet_name: String,
        message: rico_protocol::v3::message::contact_request_channel::MessageText,
    ) -> Result<UserHandle> {
        log_trace!();

        let result: Promise<Result<UserHandle>> = Default::default();
        let result_future = result.get_future();

        self.push_command(CommandData::AddPendingContact {
            session_handle,
            service_id,
            pet_name,
            result,
        });

        // todo: maybe this is weird and should be handled in the event loop task
        let result = result_future.wait();
        if let Ok(user_handle) = result {
            let contact_request_message = Some(message);
            self.push_command(CommandData::ConnectContact {
                session_handle,
                user_handle,
                contact_request_message,
            });
        }
        result
    }

    pub fn acknowledge_contact_request(
        &self,
        session_handle: SessionHandle,
        user_handle: UserHandle,
        response: tego_chat_acknowledge,
    ) -> Result<()> {
        log_trace!();

        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();

        self.push_command(CommandData::AcknowledgeContactRequest {
            session_handle,
            user_handle,
            response,
            result,
        });

        result_future.wait()
    }

    pub fn send_message(
        &self,
        session_handle: SessionHandle,
        user_handle: UserHandle,
        message_text: rico_protocol::v3::message::chat_channel::MessageText,
    ) -> Result<tego_message_id> {
        log_trace!();

        let message_id: Promise<Result<tego_message_id>> = Default::default();
        let message_id_future = message_id.get_future();
        let cmd = CommandData::SendMessage {
            session_handle,
            user_handle,
            message_text,
            message_id,
        };
        self.push_command(cmd);

        message_id_future.wait()
    }

    pub fn send_file_transfer_request(
        &self,
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_path: PathBuf,
    ) -> Result<(tego_file_transfer_id, tego_file_size)> {
        log_trace!();

        // verify absolute path
        bail_if!(!file_path.is_absolute());

        // verify path is NOT a directory
        bail_if!(file_path.is_dir());

        let result: Promise<Result<(tego_file_transfer_id, tego_file_size)>> = Default::default();
        let result_future = result.get_future();
        let cmd = CommandData::SendFileTransferRequest {
            session_handle,
            user_handle,
            file_path,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

    pub fn accept_file_transfer_request(
        &self,
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        dest_path: PathBuf,
    ) -> Result<()> {
        log_trace!();

        // verify absolute path
        bail_if!(!dest_path.is_absolute());

        // verify dest_path is NOT a directory
        bail_if!(dest_path.is_dir());

        // verify the parent directory exists
        let parent = dest_path.parent().context("dest_path has no parent")?;
        bail_if!(!parent.exists());

        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();

        let cmd = CommandData::AcceptFileTransferRequest {
            session_handle,
            user_handle,
            file_transfer_id,
            dest_path,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

    pub fn reject_file_transfer_request(
        &self,
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
    ) -> Result<()> {
        log_trace!();

        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();

        let cmd = CommandData::RejectFileTransferRequest {
            session_handle,
            user_handle,
            file_transfer_id,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

    pub fn cancel_file_transfer(
        &self,
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
    ) -> Result<()> {
        log_trace!();

        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();

        let cmd = CommandData::CancelFileTransfer {
            session_handle,
            user_handle,
            file_transfer_id,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

    #[cfg(feature = "bundled-tor")]
    pub fn tor_bin_path() -> Result<PathBuf> {
        log_trace!();

        let bin_name = format!("tor{}", std::env::consts::EXE_SUFFIX);

        // get the path of the current running exe
        let mut path = std::env::current_exe()?;
        // tor should live in the same directory
        path.pop();
        path.push(bin_name.as_str());

        if path.exists() {
            Ok(path)
        } else {
            path = which::which(bin_name)?;
            Ok(path)
        }
    }

    #[cfg(feature = "bundled-tor")]
    pub fn data_directory() -> PathBuf {
        // TODO: implement a smarter way of doing this
        let mut path = std::env::temp_dir();
        let subdirectory = format!("ricochet-refresh.{}", std::process::id());
        path.push(subdirectory.as_str());
        path.push("tor");

        path
    }

    #[cfg(all(feature = "bundled-tor", feature = "pluggable-transports"))]
    pub fn pluggable_transports() -> Result<Vec<PluggableTransportConfig>> {
        let mut pluggable_transports: Vec<PluggableTransportConfig> =
            Vec::with_capacity(PLUGGABLE_TRANSPORTS.len());

        let mut path = std::env::current_exe()?;
        path.pop();
        path.push("pluggable_transports");

        for pluggable_transport in PLUGGABLE_TRANSPORTS {
            let transports: Vec<String> = pluggable_transport
                .transports
                .iter()
                .map(|transport| transport.to_string())
                .collect();

            let mut path_to_binary = path.clone();
            path_to_binary.push(pluggable_transport.binary_name);

            let mut pluggable_transport_config =
                PluggableTransportConfig::new(transports, path_to_binary)?;

            for option in pluggable_transport.options {
                pluggable_transport_config.add_option(option.to_string());
            }

            pluggable_transports.push(pluggable_transport_config);
        }

        Ok(pluggable_transports)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if let Some(join_handle) = std::mem::take(&mut self.event_loop_thread_handle) {
            self.push_command(CommandData::EndEventLoop);
            let _ = join_handle.join();
        }
    }
}
