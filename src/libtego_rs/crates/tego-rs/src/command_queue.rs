// standard
use std::cmp::Ord;
use std::collections::BinaryHeap;
use std::ops::{Add, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

// extern
use anyhow::Result;
use tor_interface::tor_crypto::V3OnionServiceId;
use tor_interface::tor_provider::{OnionStream, TorProvider};

// internal
use crate::ffi::*;
use crate::promise::Promise;
use crate::session::{Session, SessionHandle, UserHandle};

pub(crate) struct Command {
    start_time: Instant,
    data: CommandData,
}

impl Command {
    pub fn new(data: CommandData, delay: Duration) -> Self {
        let start_time = Instant::now().add(delay);
        Self { start_time, data }
    }

    pub fn start_time(&self) -> &Instant {
        &self.start_time
    }

    pub fn data(self) -> CommandData {
        self.data
    }
}

impl Ord for Command {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.start_time.cmp(&self.start_time)
    }
}

impl PartialOrd for Command {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Command {}

pub(crate) enum CommandData {
    // library is going away we need to cleanup
    EndEventLoop,
    // begin bootstrapping a legacy tor client
    BeginLegacyTorBootstrap {
        legacy_tor_client_config: tor_interface::legacy_tor_client::LegacyTorClientConfig,
    },
    // cancel bootstrapping (if it is in progres)
    CancelTorBootstrap,
    BeginSession {
        profile: rico_profile::v4::profile::Profile,
        result: Promise<Result<SessionHandle>>,
    },
    EndSession {
        session_handle: SessionHandle,
        result: Promise<Result<()>>,
    },
    //connect to a peer and optionally request to be an allowed contact
    AddPendingContact {
        session_handle: SessionHandle,
        service_id: V3OnionServiceId,
        pet_name: String,
        result: Promise<Result<UserHandle>>,
    },
    ConnectContact {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        contact_request_message:
            Option<rico_protocol::v3::message::contact_request_channel::MessageText>,
    },
    ForgetUser {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        result: Promise<Result<()>>,
    },
    AcknowledgeContactRequest {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        response: tego_chat_acknowledge,
        result: Promise<Result<()>>,
    },
    SendMessage {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        message_text: rico_protocol::v3::message::chat_channel::MessageText,
        message_id: Promise<Result<tego_message_id>>,
    },
    // accept an incoming file transfer request
    AcceptFileTransferRequest {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        dest_path: PathBuf,
        result: Promise<Result<()>>,
    },
    SendFileTransferRequest {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_path: PathBuf,
        result: Promise<Result<(tego_file_transfer_id, tego_file_size)>>,
    },
    // reject an incoming file transfer request
    RejectFileTransferRequest {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        result: Promise<Result<()>>,
    },
    // cancel an in-progress file transfer
    CancelFileTransfer {
        session_handle: SessionHandle,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        result: Promise<Result<()>>,
    },
    //
    // Not Used Yet
    //

    // remove a user from our internal lists
    // TODO: maybe add an enqueue callback data command?
}

pub(crate) struct CommandQueue {
    queue: CommandQueueInner,
}

enum CommandQueueInner {
    Arc(Arc<Mutex<BinaryHeap<Command>>>),
    Weak(Weak<Mutex<BinaryHeap<Command>>>),
}

impl CommandQueue {
    fn queue(&self) -> Option<Arc<Mutex<BinaryHeap<Command>>>> {
        match &self.queue {
            CommandQueueInner::Arc(queue) => Some(queue.clone()),
            CommandQueueInner::Weak(queue) => queue.upgrade(),
        }
    }

    pub fn downgrade(&self) -> Self {
        match &self.queue {
            CommandQueueInner::Arc(queue) => Self {
                queue: CommandQueueInner::Weak(Arc::downgrade(queue)),
            },
            CommandQueueInner::Weak(queue) => Self {
                queue: CommandQueueInner::Weak(queue.clone()),
            },
        }
    }

    pub fn push(&self, data: CommandData, delay: Duration) {
        if let Some(queue) = self.queue() {
            let mut queue = queue.lock().expect("queue mutex poisoned");
            queue.push(Command::new(data, delay));
        }
    }

    pub fn append(&self, mut commands: BinaryHeap<Command>) {
        if let Some(queue) = self.queue() {
            let mut queue = queue.lock().expect("queue mutex poisoned");
            queue.append(&mut commands);
        }
    }

    pub fn take(&self) -> BinaryHeap<Command> {
        if let Some(queue) = self.queue() {
            let mut queue = queue.lock().expect("queue mutex poisoned");
            std::mem::take(queue.deref_mut())
        } else {
            Default::default()
        }
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self {
            queue: CommandQueueInner::Arc(Default::default()),
        }
    }
}
