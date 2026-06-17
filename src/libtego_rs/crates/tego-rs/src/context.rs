// standard
use std::collections::BTreeMap;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

// extern
use anyhow::{Context as AnyhowContext, Result};
use tor_interface::legacy_tor_client::LegacyTorClientConfig;
use tor_interface::legacy_tor_version::LegacyTorVersion;
use tor_interface::tor_crypto::{Ed25519PrivateKey, V3OnionServiceId};

// internal crates
use crate::callbacks::*;
use crate::command_queue::*;
use crate::event_loop_task::*;
use crate::ffi::*;
use crate::macros::*;
use crate::promise::Promise;

pub(crate) const RICOCHET_PORT: u16 = 9878u16;

#[derive(Default)]
pub(crate) struct Context {
    // callback struct
    pub callbacks: Arc<Mutex<Callbacks>>,
    // flags
    connect_complete: Arc<AtomicBool>,
    // command queue
    command_queue: CommandQueue,
    // event loop thread handle
    event_loop_thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl Context {
    pub fn start_event_loop(&mut self, context_handle: TegoContextHandle) {
        let callbacks = Arc::downgrade(&self.callbacks);
        let connect_complete = Arc::downgrade(&self.connect_complete);
        let command_queue = self.command_queue.downgrade();

        let task = EventLoopTask::new(
            context_handle,
            callbacks,
            connect_complete,
            command_queue,
        );

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
                }).expect("failed to start event-loop thread"),
        );
    }

    // todo: remove need for this
    pub fn connect_complete(&self) -> bool {
        log_trace!();

        self.connect_complete.load(Ordering::Relaxed)
    }

    fn push_command(&self, data: CommandData) {
        self.push_command_ex(data, Duration::ZERO);
    }

    fn push_command_ex(&self, data: CommandData, delay: Duration) {
        self.command_queue.push(data, delay);
    }

    pub fn forget_user(&mut self, service_id: V3OnionServiceId) -> Result<()> {
        log_trace!();

        // self.users.remove(&service_id);
        // let result: Promise<Result<()>> = Default::default();
        // let result_future = result.get_future();
        // self.push_command(CommandData::ForgetUser { service_id, result });

        // result_future.wait()

        Ok(())
    }

    pub fn send_contact_request(
        &self,
        service_id: V3OnionServiceId,
        message: rico_protocol::v3::message::contact_request_channel::MessageText,
    ) {
        log_trace!();

        let contact_request_message = Some(message);
        self.push_command(CommandData::ConnectContact {
            service_id,
            contact_request_message,
        });
    }

    pub fn acknowledge_contact_request(
        &self,
        service_id: V3OnionServiceId,
        response: tego_chat_acknowledge,
    ) {
        log_trace!();

        self.push_command(CommandData::AcknowledgeContactRequest {
            service_id,
            response,
        });
    }

    pub fn send_message(
        &self,
        service_id: V3OnionServiceId,
        message_text: rico_protocol::v3::message::chat_channel::MessageText,
    ) -> Result<tego_message_id> {
        log_trace!();

        let message_id: Promise<Result<tego_message_id>> = Default::default();
        let message_id_future = message_id.get_future();
        let cmd = CommandData::SendMessage {
            service_id,
            message_text,
            message_id,
        };
        self.push_command(cmd);

        message_id_future.wait()
    }

    pub fn send_file_transfer_request(
        &self,
        service_id: V3OnionServiceId,
        file_path: PathBuf,
    ) -> Result<(tego_file_transfer_id, tego_file_size)> {
        log_trace!();

        let result: Promise<Result<(tego_file_transfer_id, tego_file_size)>> = Default::default();
        let result_future = result.get_future();
        let cmd = CommandData::SendFileTransferRequest {
            service_id,
            file_path,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

    pub fn accept_file_transfer_request(
        &self,
        service_id: V3OnionServiceId,
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
            service_id,
            file_transfer_id,
            dest_path,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

    pub fn reject_file_transfer_request(
        &self,
        service_id: V3OnionServiceId,
        file_transfer_id: tego_file_transfer_id,
    ) -> Result<()> {
        log_trace!();

        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();

        let cmd = CommandData::RejectFileTransferRequest {
            service_id,
            file_transfer_id,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

    pub fn cancel_file_transfer(
        &self,
        service_id: V3OnionServiceId,
        file_transfer_id: tego_file_transfer_id,
    ) -> Result<()> {
        log_trace!();

        let result: Promise<Result<()>> = Default::default();
        let result_future = result.get_future();

        let cmd = CommandData::CancelFileTransfer {
            service_id,
            file_transfer_id,
            result,
        };
        self.push_command(cmd);

        result_future.wait()
    }

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
}

impl Drop for Context {
    fn drop(&mut self) {
        if let Some(join_handle) = std::mem::take(&mut self.event_loop_thread_handle) {
            self.push_command(CommandData::EndEventLoop);
            let _ = join_handle.join();
        }
    }
}
