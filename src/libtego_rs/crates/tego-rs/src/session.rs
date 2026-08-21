// standard
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsString;
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, Write};
use std::path::PathBuf;
use std::time::Duration;

// extern
use anyhow::{Context as AnyhowContext, Result};
use rico_profile::v4::profile::{Profile, User, UserProfile, UserType};
use rico_protocol::v3::packet_handler;
use rico_protocol::v3::packet_handler::{Event, Packet};
use rico_protocol::v3::Error;
use rico_protocol::v3::RICOCHET_PORT;
use tor_interface::tor_crypto::{Ed25519PrivateKey, Ed25519PublicKey, V3OnionServiceId};
use tor_interface::tor_provider;
use tor_interface::tor_provider::{OnionListener, OnionStream, TorProvider};

// internal
use crate::callbacks::CallbackData;
use crate::command_queue::{CommandData, CommandQueue};
use crate::ffi::{
    tego_file_size, tego_file_transfer_direction, tego_file_transfer_id,
    tego_file_transfer_response, tego_file_transfer_result, tego_message_id, tego_user_status,
};
use crate::macros::*;

pub(crate) type SessionHandle = i64;
pub(crate) type UserHandle = i64;

pub(crate) struct Session {
    session_handle: SessionHandle,
    profile: Profile,

    // Users
    users: BTreeMap<UserHandle, UserData>,
    owner: UserHandle,
    service_id_to_user_handle: BTreeMap<V3OnionServiceId, UserHandle>,

    // Bookkeeping
    pending_connections: BTreeMap<tor_provider::ConnectHandle, PendingConnection>,

    // V3 Session
    v3: V3Data,
    // V4 Session (todo)
}

// per-user book-keeping data
// todo: make these members private?
pub(crate) struct UserData {
    // data from the profile,
    // todo: should we just pull these from the profile as needed rather than mirroring
    pub(crate) user_type: UserType,
    pub(crate) pet_name: String,
    pub(crate) service_id: V3OnionServiceId,

    // connection data
    connection_failures: usize,
    // message we've tried to send but have not yet recieved an ack for
    // todo:
    queued_messages: VecDeque<UnAckedMessage>,
    next_message_id: u64,

    // mapping between tego's file_transfer_id and packet handler's FileTransferHandle
    file_transfer_handle_to_id: BTreeMap<packet_handler::FileTransferHandle, tego_file_transfer_id>,
    file_transfer_id_to_handle: BTreeMap<tego_file_transfer_id, packet_handler::FileTransferHandle>,
}

impl UserData {
    fn new(user_type: UserType, pet_name: String, service_id: V3OnionServiceId) -> Self {
        Self {
            user_type,
            pet_name,
            service_id,
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
struct PendingConnection {
    user_handle: UserHandle,
    contact_request_message:
        Option<rico_protocol::v3::message::contact_request_channel::MessageText>, // optional intro message
}

struct V3Data {
    listener: Option<OnionListener>,
    open_connections: BTreeMap<packet_handler::ConnectionHandle, Connection>,
    packet_handler: rico_protocol::v3::packet_handler::PacketHandler,
    // buffer to stream data into
    read_buffer: [u8; Self::READ_BUFFER_SIZE],
    // buffer to read file chunks into for uploads and downloads
    file_read_buffer: [u8; Self::FILE_READ_BUFFER_SIZE],

    // connections to be removed and cleaned up
    to_remove: BTreeSet<packet_handler::ConnectionHandle>,
    // users to retry connecting to
    // todo: convert to UserHandles
    to_retry: BTreeSet<UserHandle>,
}

impl V3Data {
    const READ_BUFFER_SIZE: usize = 64 * 1024;
    const FILE_READ_BUFFER_SIZE: usize = rico_protocol::v3::MAX_FILE_CHUNK_SIZE;
}

#[derive(Debug)]
struct Connection {
    // todo: is this needed still?
    pub service_id: Option<V3OnionServiceId>,
    pub user_handle: Option<UserHandle>,
    pub stream: OnionStream,
    // buffer of unhandled read bytes
    pub read_bytes: Vec<u8>,
    // buffer of read Packets to handle
    pub read_packets: Vec<Packet>,
    // buffer of packets to write
    pub write_packets: Vec<Packet>,
    // todo: maybe these should also just be a single FileTransfer
    // todo: FileDownload and FileUpload are not getting removed reliably!
    // pending and in-progress file downloads
    pub file_downloads:
        BTreeMap<rico_protocol::v3::packet_handler::FileTransferHandle, FileDownload>,
    // // pending and in-process file uploads
    pub file_uploads: BTreeMap<rico_protocol::v3::packet_handler::FileTransferHandle, FileUpload>,
}

// message enum is used for enqueueing chat and file uploads in case
// the remote user goes offline
#[derive(Debug)]
enum UnAckedMessage {
    ChatMessage {
        message_id: tego_message_id,
        message_handle: packet_handler::MessageHandle,
        timestamp: std::time::Instant,
        message_text: rico_protocol::v3::message::chat_channel::MessageText,
    },
    FileTransferRequest {
        file_transfer_id: tego_file_transfer_id,
        file_transfer_handle: packet_handler::FileTransferHandle,
        file_upload: FileUpload,
    },
}

// todo: migrate from event_loop_task
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
            .context("Path contains no file name")?
            .to_str()
            .context("File name not valid utf8")?
            .to_string();

        // open file for reading
        let mut file = std::fs::OpenOptions::new().read(true).open(file_path)?;

        let bytes_sent = 0u64;

        // get our file's size
        let size = file.metadata()?.len();

        // calculate the file's hash
        let mut hasher: rico_protocol::v3::file_hasher::FileHasher = Default::default();
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
}

impl Session {
    pub fn new(session_handle: SessionHandle, profile: Profile) -> Result<Self> {
        let mut users: BTreeMap<UserHandle, UserData> = Default::default();
        let mut owner: Option<UserHandle> = None;
        let mut service_id_to_user_handle: BTreeMap<V3OnionServiceId, UserHandle> =
            Default::default();

        let mut owner_private_key: Option<Ed25519PrivateKey> = None;

        // contact lists for packet handler
        let mut known_contacts: BTreeSet<V3OnionServiceId> = Default::default();
        let mut blocked_contacts: BTreeSet<V3OnionServiceId> = Default::default();

        // get all our user info out of the profile
        for (user, user_handle) in profile.get_users()? {
            let user_handle = user_handle.0;
            let user_type = user.user_type;
            let service_id = user.identity_v3_onion_service_id();

            if service_id_to_user_handle.insert(service_id.clone(), user_handle).is_some() {
                unreachable!("Profile should not have multiple users with identical service_ids");
            }

            let pet_name = if let Some(pet_name) = user.user_profile.pet_name {
                pet_name
            } else {
                user.user_profile.nickname
            };

            let user_data = UserData::new(user_type, pet_name, service_id.clone());
            if users.insert(user_handle, user_data).is_some() {
                unreachable!("Profile should not have duplicate UserHandles in users table");
            }

            log_info!("Adding user; ServiceId: {service_id}, UserType: {user_type:?}");

            match user_type {
                UserType::Owner => {
                    assert!(owner.is_none(), "Profile should only have one Owner");
                    owner = Some(user_handle);
                    owner_private_key = user.identity_ed25519_private_key;
                }
                UserType::Allowed | UserType::Pending => {
                    assert!(known_contacts.insert(service_id))
                }
                UserType::Blocked => assert!(blocked_contacts.insert(service_id)),
                _ => (),
            }
        }

        let owner = owner.context("Profile must have an owner")?;

        let pending_connections = Default::default();

        // v3 data
        let v3 = {
            // onion listener None by default
            let listener = None;

            // our open connections to contacts
            let open_connections = Default::default();

            // construct our legacy packet handler
            let packet_handler = rico_protocol::v3::packet_handler::PacketHandler::new(
                owner_private_key.unwrap(),
                known_contacts,
                blocked_contacts,
            );

            // buffer to read stream data to for parsing
            let read_buffer = [0u8; V3Data::READ_BUFFER_SIZE];

            // buffer to write file chunks into for file transfers
            let file_read_buffer = [0u8; V3Data::FILE_READ_BUFFER_SIZE];

            // set of dead connections
            let to_remove = Default::default();

            // users to retry connecting to
            let to_retry = Default::default();

            V3Data {
                listener,
                open_connections,
                packet_handler,
                read_buffer,
                file_read_buffer,
                to_remove,
                to_retry,
            }
        };

        Ok(Session {
            session_handle,
            profile,
            users,
            owner,
            // allowed_users,
            // pending_users,
            // requesting_users,
            // rejected_users,
            // blocked_users,
            service_id_to_user_handle,
            pending_connections,
            v3,
        })
    }

    //
    // User Getters
    //

    pub fn get_users(&self) -> &BTreeMap<UserHandle, UserData> {
        &self.users
    }

    //
    // Protocol Stuffs
    //

    // Called each frame to update our internal stuff
    pub fn update_v3(
        &mut self,
        command_queue: &mut CommandQueue,
        callback_queue: &mut Vec<CallbackData>,
    ) -> Result<()> {
        // handle new incoming connections
        if let Some(listener) = &mut self.v3.listener {
            loop {
                match listener.accept() {
                    Ok(Some(stream)) => {
                        log_info!("New incoming connection");
                        if let Ok(()) = stream.set_nonblocking(true) {
                            let handle = self.v3.packet_handler.new_incoming_connection()?;
                            let connection = Connection {
                                service_id: None,
                                user_handle: None,
                                stream,
                                read_bytes: Default::default(),
                                read_packets: Default::default(),
                                write_packets: Default::default(),
                                file_downloads: Default::default(),
                                file_uploads: Default::default(),
                            };

                            self.v3.open_connections.insert(handle, connection);
                        } else {
                            log_error!("Unable to set incoming stream to nonblocking");
                        }
                    }
                    Ok(None) => break,
                    Err(err) => {
                        log_error!("Error listening for new connections: {err}");
                        break;
                    }
                }
            }
        }

        //
        // Handle Connections
        //

        self.handle_connections_v3(command_queue, callback_queue);

        Ok(())
    }

    fn handle_connections_v3(
        &mut self,
        command_queue: &mut CommandQueue,
        callback_queue: &mut Vec<CallbackData>,
    ) {
        let session_handle = self.session_handle;
        let packet_handler = &mut self.v3.packet_handler;
        for (&connection_handle, connection) in self.v3.open_connections.iter_mut() {
            // TODO: we need some kind of exponential backoff for repeated failures to connect+authenticate
            // blocked users will currently spam over and over again

            // read bytes from each connection
            let stream = &mut connection.stream;

            // handle reading
            match stream.read(&mut self.v3.read_buffer) {
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => (),
                    _ => {
                        // some error
                        log_error!("Stream read err: {err:?}");
                        self.v3.to_remove.insert(connection_handle);
                        if let Some(user_handle) = connection.user_handle {
                            self.v3.to_retry.insert(user_handle);
                        }
                    }
                },
                Ok(0) => {
                    // end of stream
                    log_error!("Stream read err: end of stream");
                    self.v3.to_remove.insert(connection_handle);
                    if let Some(user_handle) = connection.user_handle {
                        self.v3.to_retry.insert(user_handle);
                    }
                }
                Ok(size) => {
                    let read_buffer = &self.v3.read_buffer[..size];
                    connection
                        .read_bytes
                        .write_all(read_buffer)
                        .expect("read_bytes write failed");
                }
            }

            // handle reading packets
            let mut read_bytes = connection.read_bytes.as_slice();
            let read_packets = &mut connection.read_packets;
            let write_packets = &mut connection.write_packets;

            if !read_bytes.is_empty() {
                // total handled bytes
                let mut trim_count = 0usize;

                // parse read bytes into packets
                'packet_parse: loop {
                    match packet_handler.try_parse_packet(connection_handle, read_bytes) {
                        Ok((packet, size)) => {
                            log_packet!("read {packet:?}");
                            // move slice up by number of handled bytes
                            read_bytes = &read_bytes[size..];
                            trim_count += size;
                            // save off read bytes for handling
                            read_packets.push(packet);
                        }
                        Err(Error::NeedMoreBytes) => {
                            break 'packet_parse;
                        }
                        Err(_err) => {
                            log_error!("parse packet error: {_err:?}, read_bytes: {read_bytes:?}");
                            // drop connection
                            self.v3.to_remove.insert(connection_handle);
                            break 'packet_parse;
                        }
                    }
                }

                // drop handled bytes off the front
                connection.read_bytes.drain(0..trim_count);

                // handle packets and queue responses
                'packet_handle: for packet in read_packets.drain(..) {
                    match packet_handler.handle_packet(connection_handle, packet, write_packets) {
                        Ok(Event::IntroductionReceived) => {
                            log_info!("Introduction received");
                        }
                        Ok(Event::IntroductionResponseReceived) => {
                            log_info!("Introduction response received");
                        }
                        Ok(Event::OpenChannelAuthHiddenServiceReceived) => {
                            log_info!("Open auth hidden service received");
                        }
                        Ok(Event::ClientAuthenticated {
                            service_id,
                            duplicate_connection,
                        }) => {
                            // todo: handle closed connection
                            log_info!("Client authenticated: peer: {service_id:?}, duplicate_connection: {duplicate_connection:?}");
                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                connection.user_handle = Some(user_handle);
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    user_data.connection_failures = 0usize;
                                }
                            }

                            connection.service_id = Some(service_id);

                            if let Some(connection_handle) = duplicate_connection {
                                self.v3.to_remove.insert(connection_handle);
                            }
                        }
                        Ok(Event::BlockedClientAuthenticationAttempted {
                            service_id: _service_id,
                        }) => {
                            log_info!(
                                "Blocked client attempted authentication, peer: {_service_id}"
                            );
                            self.v3.to_remove.insert(connection_handle);
                        }
                        Ok(Event::HostAuthenticated {
                            service_id,
                            is_known_contact,
                            duplicate_connection,
                        }) => {
                            // when we are connecting to a client we only
                            // want to clear our connection failure count
                            // if the remote host has short-cut the contact request
                            // machinery
                            // the remote host may still reject the user in the second
                            // case which we want to treat as a conneciton failure
                            // for the purposes of reconnect attempt delays to reduce
                            // cconnection spamming
                            // that is to say, if we were to *always* clear this counter
                            // just on authentication success, a blocked user would repeatdly connect over and over and over again which we
                            // do not want
                            if is_known_contact {
                                if let Some(user_handle) =
                                    self.service_id_to_user_handle.get(&service_id)
                                {
                                    if let Some(user_data) = self.users.get_mut(user_handle) {
                                        user_data.connection_failures = 0usize;
                                    }
                                }
                            }
                            // todo: handle closed connection
                            log_info!("Host authenticated: peer: {service_id:?}, duplicate_connection: {duplicate_connection:?}");
                            if let Some(connection_handle) = duplicate_connection {
                                self.v3.to_remove.insert(connection_handle);
                            }
                        }
                        Ok(Event::DuplicateConnectionDropped {
                            duplicate_connection,
                        }) => {
                            // todo: handle closed connection
                            log_info!("Duplicate connection dropped: {duplicate_connection}");
                            self.v3.to_remove.insert(duplicate_connection);
                        }
                        Ok(Event::ContactRequestReceived {
                            service_id,
                            nickname: _,
                            message_text,
                        }) => {
                            log_info!("Contact request received, peer: {service_id:?}, message_text: \"{message_text}\"");

                            let session_handle = self.session_handle;
                            // check to see if this user has been previously rejected, in which case we migrate them back to requesting
                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                let user_data = self.users.get_mut(&user_handle).unwrap();
                                match user_data.user_type {
                                    // if user is already Requesting, we can ignore subsequent request
                                    UserType::Requesting => (),
                                    // if user is already Allowed or Pending, we can ignore
                                    UserType::Allowed | UserType::Pending => (),
                                    // migrate user over to Requesting and trigger callback
                                    UserType::Rejected => {
                                        user_data.user_type = UserType::Requesting;
                                        self.profile.set_user_type(user_handle.into(), UserType::Requesting).expect("Converting Rejected user to Requesting user should not fail");
                                        callback_queue.push(CallbackData::ChatRequestReceived {
                                            session_handle,
                                            user_handle,
                                            message: message_text,
                                        });
                                    },
                                    // todo: can we get here as an allowed user or does that early out elsewhere?
                                    user_type => unreachable!("Impossible to handle Event::ContactRequestReceived from a user of type {user_type:?}"),
                                }
                            } else {
                                // Add new user to profile as Requesting user
                                let user =
                                    User {
                                        user_type: UserType::Requesting,
                                        user_profile: UserProfile {
                                            nickname: service_id.to_string(),
                                            pet_name: None,
                                            pronouns: None,
                                            avatar: None,
                                            status: None,
                                            description: None,
                                        },
                                        identity_ed25519_public_key:
                                            Ed25519PublicKey::from_service_id(&service_id).unwrap(),
                                        identity_ed25519_private_key: None,
                                        remote_endpoint_ed25519_public_key: None,
                                        remote_endpoint_x25519_private_key: None,
                                        local_endpoint_ed25519_private_key: None,
                                        local_endpoint_x25519_public_key: None,
                                    };

                                match self.profile.add_user(&user) {
                                    Ok(user_handle) => {
                                        self.service_id_to_user_handle
                                            .insert(service_id.clone(), user_handle.into());
                                        self.users.insert(
                                            user_handle.into(),
                                            UserData::new(
                                                UserType::Requesting,
                                                service_id.to_string(),
                                                service_id,
                                            ),
                                        );
                                        // notify frontend
                                        let user_handle = user_handle.0;
                                        let user_type = UserType::Requesting;
                                        callback_queue.push(CallbackData::UserAdded {
                                            session_handle,
                                            user_handle,
                                            user_type,
                                        });
                                        callback_queue.push(CallbackData::ChatRequestReceived {
                                            session_handle,
                                            user_handle,
                                            message: message_text,
                                        });
                                    }
                                    // error handling
                                    Err(err) => log_error!("Failed to add user {user:?}; {err}"),
                                }
                            }
                        }
                        Ok(Event::ContactRequestResultPending {
                            service_id: _service_id,
                        }) => {
                            log_info!("Contact request result pending, peer: {_service_id:?}");
                        }
                        Ok(Event::ContactRequestResultAccepted { service_id }) => {
                            log_info!("Contact request result accepted, peer: {service_id:?}");
                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    user_data.user_type = UserType::Allowed;
                                    self.profile
                                        .set_user_type(user_handle.into(), UserType::Allowed)
                                        .expect(
                                        "Converting Pending user to Allowed user should not fail",
                                    );
                                    callback_queue.push(
                                        CallbackData::ChatRequestResponseReceived {
                                            session_handle,
                                            user_handle,
                                            accepted_request: true,
                                        },
                                    );
                                    callback_queue.push(CallbackData::UserStatusChanged {
                                        session_handle,
                                        user_handle,
                                        status: tego_user_status::tego_user_status_online,
                                    });
                                }
                            }
                        }
                        Ok(Event::ContactRequestResultRejected { service_id }) => {
                            log_info!("Contact request result rejected, peer: {service_id:?}");
                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    user_data.user_type = UserType::Rejected;
                                    self.profile
                                        .set_user_type(user_handle.into(), UserType::Rejected)
                                        .expect(
                                        "Converting Pending user to Rejected user should not fail",
                                    );

                                    callback_queue.push(
                                        CallbackData::ChatRequestResponseReceived {
                                            session_handle,
                                            user_handle,
                                            accepted_request: false,
                                        },
                                    );
                                }
                            }
                        }
                        Ok(Event::IncomingChatChannelOpened {
                            service_id: _service_id,
                        }) => {
                            log_info!("Incoming chat channel opened, peer: {_service_id:?}");
                        }
                        Ok(Event::IncomingFileTransferChannelOpened {
                            service_id: _service_id,
                        }) => {
                            log_info!(
                                "Incoming file transfer channel opened, peer: {_service_id:?}"
                            );
                        }
                        Ok(Event::OutgoingAuthHiddenServiceChannelOpened {
                            service_id: _service_id,
                        }) => {
                            log_info!(
                                "Outgoing auth hidden service channel opened, peer: {_service_id:?}"
                            );
                        }
                        Ok(Event::OutgoingChatChannelOpened { service_id }) => {
                            log_info!("Outgoing chat channel opened, peer: {service_id:?}");

                            if let Some(user_handle) =
                                self.service_id_to_user_handle.get(&service_id).cloned()
                            {
                                // send queued messages
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    user_data.connection_failures = 0usize;

                                    log_info!("Re-sending un-acked messages");
                                    if !user_data.queued_messages.is_empty() {
                                        for message in user_data.queued_messages.iter_mut() {
                                            if let UnAckedMessage::ChatMessage {
                                                message_id: _,
                                                message_handle,
                                                timestamp,
                                                message_text,
                                            } = message
                                            {
                                                match packet_handler.send_message(
                                                    service_id.clone(),
                                                    message_text.clone(),
                                                    Some(
                                                        std::time::Instant::now()
                                                            .duration_since(*timestamp),
                                                    ),
                                                    write_packets,
                                                ) {
                                                    Ok((
                                                        _connection_handle,
                                                        new_message_handle,
                                                    )) => {
                                                        // update the queued message with new message handle
                                                        *message_handle = new_message_handle;
                                                    }
                                                    Err(_err) => {
                                                        log_error!(
                                                            "Error re-sending queued message: {_err}"
                                                        )
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let status = tego_user_status::tego_user_status_online;
                                    callback_queue.push(CallbackData::UserStatusChanged {
                                        session_handle,
                                        user_handle,
                                        status,
                                    });
                                } else {
                                    log_error!("No UserData for service id: {service_id}");
                                }
                            } else {
                                log_error!("No UserHandle for service id: {service_id}");
                            }
                        }
                        Ok(Event::OutgoingFileTransferChannelOpened { service_id }) => {
                            if let Some(user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                log_info!(
                                    "Outgoing file transfer channel opened, peer: {service_id:?}"
                                );

                                // send queued messages
                                if let Some(user_data) = self.users.get_mut(user_handle) {
                                    log_info!("Re-sending un-acked file transfer requests");
                                    if !user_data.queued_messages.is_empty() {
                                        for message in user_data.queued_messages.iter_mut() {
                                            if let UnAckedMessage::FileTransferRequest {
                                                file_transfer_id,
                                                file_transfer_handle,
                                                file_upload,
                                            } = message
                                            {
                                                match packet_handler.send_file_transfer_request(
                                                    service_id.clone(),
                                                    file_upload.name.clone(),
                                                    file_upload.size,
                                                    file_upload.hash,
                                                    write_packets,
                                                ) {
                                                    Ok((
                                                        _connection_handle,
                                                        new_file_transfer_handle,
                                                    )) => {
                                                        // update the queued upload request with new file transfer handle handle
                                                        *file_transfer_handle =
                                                            new_file_transfer_handle;

                                                        // update our handle <-> id mappings
                                                        if let Some(old_file_transfer_handle) =
                                                            user_data
                                                                .file_transfer_id_to_handle
                                                                .insert(
                                                                    *file_transfer_id,
                                                                    *file_transfer_handle,
                                                                )
                                                        {
                                                            // remap ids and handles
                                                            user_data
                                                                .file_transfer_handle_to_id
                                                                .remove(&old_file_transfer_handle);
                                                        }
                                                        user_data
                                                            .file_transfer_handle_to_id
                                                            .insert(
                                                                *file_transfer_handle,
                                                                *file_transfer_id,
                                                            );
                                                    }
                                                    Err(_err) => {
                                                        log_error!(
                                                            "Error re-sending queued file transfer request: {_err}"
                                                        )
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Event::ChatMessageReceived {
                            service_id,
                            message_text,
                            message_handle: _message_handle,
                            time_delta,
                        }) => {
                            log_info!("Chat message receved, peer: {service_id:?}, message: \"{message_text}, message_handle: {_message_handle:?}, time_delta: {time_delta:?}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    let now = std::time::SystemTime::now();
                                    let timestamp = now.checked_sub(time_delta).unwrap_or(now);
                                    let message_id = user_data.next_message_id();
                                    let message = message_text;
                                    log_info!("MessageReceived enqueued");
                                    callback_queue.push(CallbackData::MessageReceived {
                                        session_handle,
                                        user_handle,
                                        timestamp,
                                        message_id,
                                        message,
                                    });
                                }
                            }
                        }
                        Ok(Event::ChatAcknowledgeReceived {
                            service_id,
                            message_handle,
                            accepted,
                        }) => {
                            log_info!("Chat ack received, peer: {service_id:?}, message_handle: {message_handle:?}, accepted: {accepted}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                // find message in queue and remove as it has been acked
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    let queued_messages = &mut user_data.queued_messages;
                                    for i in 0..queued_messages.len() {
                                        if let UnAckedMessage::ChatMessage {
                                            message_id: unacked_message_id,
                                            message_handle: unacked_message_handle,
                                            timestamp: _,
                                            message_text: _,
                                        } = &mut queued_messages[i]
                                        {
                                            if *unacked_message_handle == message_handle {
                                                let message_id = *unacked_message_id;
                                                let _ = queued_messages.remove(i);
                                                callback_queue.push(
                                                    CallbackData::MessageAcknowledged {
                                                        session_handle,
                                                        user_handle,
                                                        message_id,
                                                        accepted,
                                                    },
                                                );
                                                break;
                                            }
                                        }
                                    }
                                } else {
                                    log_error!("Received chat ack for unknown user {service_id}");
                                }
                            }
                        }
                        Ok(Event::FileTransferRequestReceived {
                            service_id,
                            file_transfer_handle,
                            file_name,
                            file_size,
                        }) => {
                            log_info!("File transfer request received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, file_name: {file_name}, file_size: {file_size}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    let file_transfer_id = user_data.next_message_id();
                                    user_data
                                        .file_transfer_id_to_handle
                                        .insert(file_transfer_id, file_transfer_handle);
                                    user_data
                                        .file_transfer_handle_to_id
                                        .insert(file_transfer_handle, file_transfer_id);

                                    // the protocol handler *shouldn't* be returning duplicate handles but we get them
                                    // from the other party so really we have no control here :(
                                    if connection
                                        .file_downloads
                                        .contains_key(&file_transfer_handle)
                                    {
                                        // if we have a collision, just cancel the old one
                                        callback_queue.push(CallbackData::FileTransferComplete{
                                            session_handle,
                                            user_handle,
                                            file_transfer_id,
                                            direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
                                            result: tego_file_transfer_result::tego_file_transfer_result_cancelled
                                        });
                                    }

                                    let file_download: FileDownload = FileDownload::new(file_size);
                                    connection
                                        .file_downloads
                                        .insert(file_transfer_handle, file_download);

                                    callback_queue.push(
                                        CallbackData::FileTransferRequestReceived {
                                            session_handle,
                                            user_handle,
                                            file_transfer_id,
                                            file_name,
                                            file_size,
                                        },
                                    );
                                }
                            }
                        }
                        Ok(Event::FileTransferRequestAcknowledgeReceived {
                            service_id,
                            file_transfer_handle,
                            accepted,
                        }) => {
                            log_info!("File transfer request ack received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, accepted: {accepted}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    // find file transfer request in queue and remove as it has been acked
                                    // todo: switch this to use queued_messages.iter().position rather than
                                    // manual for-looping (same in message ack recieved)
                                    let queued_messages = &mut user_data.queued_messages;
                                    for i in 0..queued_messages.len() {
                                        if let UnAckedMessage::FileTransferRequest {
                                            file_transfer_id: _,
                                            file_transfer_handle: unacked_file_transfer_handle,
                                            file_upload: _,
                                        } = &mut queued_messages[i]
                                        {
                                            if *unacked_file_transfer_handle == file_transfer_handle
                                            {
                                                if let Some(UnAckedMessage::FileTransferRequest {
                                                    file_transfer_id,
                                                    file_transfer_handle: _,
                                                    file_upload,
                                                }) = queued_messages.remove(i)
                                                {
                                                    // save off file upload record
                                                    connection
                                                        .file_uploads
                                                        .insert(file_transfer_handle, file_upload);

                                                    callback_queue.push(
                                                        CallbackData::FileTransferRequestAcknowledged {
                                                            session_handle,
                                                            user_handle,
                                                            file_transfer_id,
                                                            accepted,
                                                        },
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Event::FileTransferRequestAccepted {
                            service_id,
                            file_transfer_handle,
                        }) => {
                            log_info!("File transfer request accepted, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    if let Some(&file_transfer_id) = user_data
                                        .file_transfer_handle_to_id
                                        .get(&file_transfer_handle)
                                    {
                                        if let Some(file_upload) =
                                            connection.file_uploads.get_mut(&file_transfer_handle)
                                        {
                                            // begin sending chunks
                                            let bytes_read = match file_upload
                                                .read(&mut self.v3.file_read_buffer)
                                            {
                                                Ok(bytes_read) => bytes_read,
                                                // todo: this should cleanup and trigger an error callback
                                                Err(_err) => todo!(),
                                            };
                                            let chunk_data: Vec<u8> =
                                                self.v3.file_read_buffer[..bytes_read].to_vec();

                                            match packet_handler
                                                .send_file_chunk(
                                                    &service_id,
                                                    file_transfer_handle,
                                                    chunk_data,
                                                    write_packets,
                                                ) {
                                                Ok(_connection_handle) => {
                                                    file_upload.bytes_sent += bytes_read as u64;
                                                }
                                                // todo: again this should error callback and cleanup
                                                Err(_err) => todo!(),
                                            };

                                            // trigger callbacks
                                            callback_queue
                                                .push(CallbackData::FileTransferRequestResponseReceived {
                                                session_handle,
                                                user_handle,
                                                file_transfer_id,
                                                response:
                                                    tego_file_transfer_response::tego_file_transfer_response_accept,
                                            });
                                            callback_queue.push(CallbackData::FileTransferProgress{
                                                session_handle,
                                                user_handle,
                                                file_transfer_id,
                                                direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
                                                bytes_complete: file_upload.bytes_sent,
                                                bytes_total: file_upload.size,
                                            });

                                            file_upload.bytes_sent += bytes_read as u64;
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Event::FileTransferRequestRejected {
                            service_id,
                            file_transfer_handle,
                        }) => {
                            log_info!("File transfer request rejected, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    if let Some(&file_transfer_id) = user_data
                                        .file_transfer_handle_to_id
                                        .get(&file_transfer_handle)
                                    {
                                        let response = tego_file_transfer_response::tego_file_transfer_response_reject;
                                        callback_queue.push(
                                            CallbackData::FileTransferRequestResponseReceived {
                                                session_handle,
                                                user_handle,
                                                file_transfer_id,
                                                response,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                        Ok(Event::FileChunkReceived {
                            service_id,
                            file_transfer_handle,
                            data,
                            last_chunk,
                            hash_matches,
                        }) => {
                            log_info!("File chunk received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, data: [u8; {}], last_chunk: {last_chunk}, hash_matches: {hash_matches:?}", data.len());

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    if let Some(&file_transfer_id) = user_data
                                        .file_transfer_handle_to_id
                                        .get(&file_transfer_handle)
                                    {
                                        // these two last_chunk checks get us a Option<FileDownload&>
                                        // in both cases where we need to remove it and where we need to modify
                                        // it in-place
                                        if let Some(file_download) =
                                            connection.file_downloads.get_mut(&file_transfer_handle)
                                        {
                                            // write chunk to disk
                                            match file_download.write(&data) {
                                                Ok(()) => {
                                                    callback_queue.push(CallbackData::FileTransferProgress{
                                                        session_handle,
                                                        user_handle,
                                                        file_transfer_id,
                                                        direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
                                                        bytes_complete: file_download.bytes_written,
                                                        bytes_total: file_download.expected_size,
                                                    });
                                                    // handle completed donwload
                                                    if last_chunk {
                                                        match hash_matches {
                                                            Some(true) => {
                                                                let result = match file_download.finalize() {
                                                                    Ok(()) => tego_file_transfer_result::tego_file_transfer_result_success,
                                                                    Err(_) => tego_file_transfer_result::tego_file_transfer_result_filesystem_error,
                                                                };
                                                                callback_queue.push(CallbackData::FileTransferComplete{
                                                                    session_handle,
                                                                    user_handle,
                                                                    file_transfer_id,
                                                                    direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
                                                                    result,
                                                                });
                                                            }
                                                            Some(false) => {
                                                                callback_queue.push(CallbackData::FileTransferComplete{
                                                                    session_handle,
                                                                    user_handle,
                                                                    file_transfer_id,
                                                                    direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
                                                                    result: tego_file_transfer_result::tego_file_transfer_result_bad_hash,
                                                                });
                                                            }
                                                            None => unreachable!(),
                                                        }
                                                        connection
                                                            .file_downloads
                                                            .remove(&file_transfer_handle);
                                                    }
                                                }
                                                Err(err) => {
                                                    callback_queue.push(CallbackData::FileTransferComplete{
                                                        session_handle,
                                                        user_handle,
                                                        file_transfer_id,
                                                        direction: tego_file_transfer_direction::tego_file_transfer_direction_receiving,
                                                        result: tego_file_transfer_result::tego_file_transfer_result_filesystem_error,
                                                    });
                                                    log_error!("{err}")
                                                }
                                            }
                                        }
                                    }
                                }
                            };
                        }
                        Ok(Event::FileChunkAckReceived {
                            service_id,
                            file_transfer_handle,
                            offset,
                        }) => {
                            log_info!("File chunk ack received, peer: {service_id:?}, file_transfer_handle: {file_transfer_handle:?}, offset: {offset}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    if let Some(&file_transfer_id) = user_data
                                        .file_transfer_handle_to_id
                                        .get(&file_transfer_handle)
                                    {
                                        if let Some(file_upload) =
                                            connection.file_uploads.get_mut(&file_transfer_handle)
                                        {
                                            callback_queue.push(CallbackData::FileTransferProgress{
                                                session_handle,
                                                user_handle,
                                                file_transfer_id,
                                                direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
                                                bytes_complete: file_upload.bytes_sent,
                                                bytes_total: file_upload.size,
                                            });

                                            // todo: better error handling
                                            assert_eq!(file_upload.bytes_sent, offset);

                                            // send next chunk if there is more data to sesnd
                                            if file_upload.bytes_sent < file_upload.size {
                                                match file_upload
                                                    .read(&mut self.v3.file_read_buffer)
                                                {
                                                    Ok(bytes_read) => {
                                                        let chunk_data: Vec<u8> =
                                                            self.v3.file_read_buffer[..bytes_read]
                                                                .to_vec();

                                                        match packet_handler
                                                            .send_file_chunk(
                                                                &service_id,
                                                                file_transfer_handle,
                                                                chunk_data,
                                                                write_packets,
                                                            ) {
                                                            Ok(_) => file_upload.bytes_sent +=
                                                                bytes_read as u64,
                                                            Err(_err) => {
                                                                log_error!("{_err:?}");
                                                                todo!();
                                                            }
                                                        }
                                                    }
                                                    Err(_err) => {
                                                        log_error!("failed to read next file chunk for fille transfer {file_transfer_handle:?}: {_err}");
                                                        // disk error, terminate this file transfer
                                                        if let Err(_err) = packet_handler
                                                            .cancel_file_transfer(
                                                                &service_id,
                                                                file_transfer_handle,
                                                                true,
                                                                write_packets,
                                                            )
                                                        {
                                                            // error handling
                                                            log_error!("{_err:?}");
                                                            todo!();
                                                        }
                                                    }
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Event::FileTransferSucceeded {
                            service_id,
                            file_transfer_handle,
                        }) => {
                            log_info!("File transfer succeeded, peer: {service_id}, file_transfer_handle: {file_transfer_handle:?}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    if let Some(&file_transfer_id) = user_data
                                        .file_transfer_handle_to_id
                                        .get(&file_transfer_handle)
                                    {
                                        let direction = tego_file_transfer_direction::tego_file_transfer_direction_sending;
                                        let result = tego_file_transfer_result::tego_file_transfer_result_success;
                                        callback_queue.push(CallbackData::FileTransferComplete {
                                            session_handle,
                                            user_handle,
                                            file_transfer_id,
                                            direction,
                                            result,
                                        });
                                    }
                                }
                            };
                        }
                        Ok(Event::FileTransferFailed {
                            service_id,
                            file_transfer_handle,
                        }) => {
                            log_info!("File transfer failed, peer: {service_id}, file_transfer_handle: {file_transfer_handle:?}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    if let Some(&file_transfer_id) = user_data
                                        .file_transfer_handle_to_id
                                        .get(&file_transfer_handle)
                                    {
                                        let direction = tego_file_transfer_direction::tego_file_transfer_direction_sending;
                                        let result = tego_file_transfer_result::tego_file_transfer_result_failure;

                                        callback_queue.push(CallbackData::FileTransferComplete {
                                            session_handle,
                                            user_handle,
                                            file_transfer_id,
                                            direction,
                                            result,
                                        });
                                    }
                                }
                            }
                        }
                        Ok(Event::FileTransferCancelled {
                            service_id,
                            file_transfer_handle,
                        }) => {
                            log_info!("File transfer cancelled, peer: {service_id}, file_transfer_handle: {file_transfer_handle:?}");

                            if let Some(&user_handle) =
                                self.service_id_to_user_handle.get(&service_id)
                            {
                                if let Some(user_data) = self.users.get_mut(&user_handle) {
                                    if let Some(&file_transfer_id) = user_data
                                        .file_transfer_handle_to_id
                                        .get(&file_transfer_handle)
                                    {
                                        // todo: the file transfer should be remoed from our Connection!
                                        callback_queue.push(CallbackData::FileTransferComplete{
                                            session_handle,
                                            user_handle,
                                            file_transfer_id,
                                            // todo: direction should be deterined by whether this is an upload or download
                                            direction: tego_file_transfer_direction::tego_file_transfer_direction_sending,
                                            result: tego_file_transfer_result::tego_file_transfer_result_cancelled
                                        });
                                    }
                                }
                            }
                        }
                        Ok(Event::ChannelClosed { id: _id }) => {
                            log_info!("Channel closed: {_id}");
                        }
                        // // errors
                        Ok(Event::ProtocolFailure { message: _message }) => {
                            log_info!("Non-fatal protocol failure: {_message}");
                        }
                        Ok(Event::FatalProtocolFailure { message: _message }) => {
                            log_error!("Fatal protocol error, removing connection: {_message}");
                            self.v3.to_remove.insert(connection_handle);
                            break 'packet_handle;
                        }
                        Err(err) => panic!("error: {err:?}"),
                    }
                }
            }

            // write packets to stream
            if !write_packets.is_empty() {
                // serialise out packets to bytes
                let mut write_bytes: Vec<u8> = Default::default();
                for packet in write_packets.drain(..) {
                    log_packet!("write {packet:?}");
                    packet
                        .write_to_vec(&mut write_bytes)
                        .expect("packet failed");
                }

                // send bytes
                let stream = &mut connection.stream;
                if stream.write(write_bytes.as_slice()).is_err() {
                    self.v3.to_remove.insert(connection_handle);
                    if let Some(user_handle) = connection.user_handle {
                        self.v3.to_retry.insert(user_handle);
                    }
                }
            }
        }

        //
        // Drop our dead connections
        //
        for connection_handle in self.v3.to_remove.iter() {
            self.v3.packet_handler.remove_connection(connection_handle);
            let connection = self
                .v3
                .open_connections
                .remove(connection_handle)
                .expect("Removed non-existing connection");
            // signal user offline status to frontend
            if let Some(service_id) = connection.service_id {
                log_info!(
                    "Dropping connection; handle: {connection_handle}, service_id: {service_id}"
                );
                // signal user offline status if there is not another connection
                if !self.v3.packet_handler.has_verified_connection(&service_id) {
                    if let Some(&user_handle) = self.service_id_to_user_handle.get(&service_id) {
                        if let Some(user_data) = self.users.get(&user_handle) {
                            if let UserType::Allowed = user_data.user_type {
                                use crate::ffi::tego_user_status::tego_user_status_offline;
                                let status = tego_user_status_offline;
                                callback_queue.push(CallbackData::UserStatusChanged {
                                    session_handle,
                                    user_handle,
                                    status,
                                });
                            }
                        }
                    }
                }
            } else {
                // connect dropped for unkown remote user (i.e. didn't make it past the handshake)
                log_info!("Dropping connection; handle: {connection_handle}");
            }
        }
        self.v3.to_remove.clear();

        //
        // Initiate connection retries for connections which had IO failures
        //
        for &user_handle in self.v3.to_retry.iter() {
            if let Some(user_data) = self.users.get_mut(&user_handle) {
                if matches!(user_data.user_type, UserType::Allowed | UserType::Pending) {
                    user_data.connection_failures += 1usize;

                    let command_data = CommandData::ConnectContact {
                        session_handle,
                        user_handle,
                        contact_request_message: None,
                    };
                    let delay = Self::retry_delay(user_data.connection_failures);

                    let service_id = &user_data.service_id;
                    log_info!("Retry connecting to {service_id} in {delay:?}");

                    command_queue.push(command_data, delay);
                }
            }
        }
        self.v3.to_retry.clear();
    }

    //
    // Our direct methods for user-directed actions
    //

    pub fn add_pending_contact(
        &mut self,
        service_id: V3OnionServiceId,
        pet_name: String,
    ) -> Result<UserHandle> {
        log_trace!();
        bail_if!(self.service_id_to_user_handle.contains_key(&service_id));

        let user = User {
            user_type: UserType::Pending,
            user_profile: UserProfile {
                nickname: service_id.to_string(),
                pet_name: Some(pet_name.clone()),
                pronouns: None,
                avatar: None,
                status: None,
                description: None,
            },
            identity_ed25519_public_key: Ed25519PublicKey::from_service_id(&service_id).unwrap(),
            identity_ed25519_private_key: None,
            remote_endpoint_ed25519_public_key: None,
            remote_endpoint_x25519_private_key: None,
            local_endpoint_ed25519_private_key: None,
            local_endpoint_x25519_public_key: None,
        };
        let user_handle: UserHandle = self.profile.add_user(&user)?.into();
        self.service_id_to_user_handle
            .insert(service_id.clone(), user_handle);
        self.users.insert(
            user_handle,
            UserData::new(UserType::Pending, pet_name, service_id),
        );

        Ok(user_handle)
    }

    pub fn start_onion_listener(&mut self, tor_provider: &mut Box<dyn TorProvider>) -> Result<()> {
        let listener = tor_provider.listener(
            self.v3.packet_handler.get_private_key(),
            RICOCHET_PORT,
            None,
        )?;
        listener.set_nonblocking(true)?;
        self.v3.listener = Some(listener);
        Ok(())
    }

    // todo: we should just provide a way to get a list of all our known contacts and then call connect_contact individually
    // as it is this function is called from event loop task, and just enqueues future work for the event loop task to do
    // via Self::connect_contact()
    pub fn connect_all_known_contacts(&self, command_queue: &mut CommandQueue) {
        for (&user_handle, user_data) in self.users.iter() {
            match user_data.user_type {
                UserType::Allowed | UserType::Pending => {
                    let session_handle = self.session_handle;
                    let contact_request_message = None;
                    command_queue.push(
                        CommandData::ConnectContact {
                            session_handle,
                            user_handle,
                            contact_request_message,
                        },
                        Duration::ZERO,
                    )
                }
                _ => (),
            }
        }
    }

    pub fn connect_contact(
        &mut self,
        user_handle: UserHandle,
        tor_provider: &mut Box<dyn TorProvider>,
        contact_request_message: Option<
            rico_protocol::v3::message::contact_request_channel::MessageText,
        >,
        command_queue: &mut CommandQueue,
    ) -> Result<Option<tor_provider::ConnectHandle>> {
        // only open new connection if there is no existing verified
        // connection already

        let user_data = self
            .users
            .get_mut(&user_handle)
            .context("User does not exist")?;

        let service_id = &user_data.service_id;
        log_info!("Try to connect to contact: {service_id}");
        if !self.v3.packet_handler.has_verified_connection(service_id) {
            let target_addr: tor_interface::tor_provider::TargetAddr =
                (service_id.clone(), RICOCHET_PORT).into();

            if let Ok(connect_handle) = tor_provider.connect_async(target_addr, None) {
                log_info!("Conecting to {service_id}");
                self.pending_connections.insert(
                    connect_handle,
                    PendingConnection {
                        user_handle,
                        contact_request_message,
                    },
                );
                Ok(Some(connect_handle))
            } else {
                user_data.connection_failures += 1;

                let failure_count = user_data.connection_failures;
                let service_id = &user_data.service_id;
                // delay before trying to connect in seconds
                let delay = Self::retry_delay(failure_count);

                log_info!(
                    "Connect attempt {failure_count} to {service_id:?} failed; try again in {delay:?}"
                );

                let command_data = CommandData::ConnectContact {
                    session_handle: self.session_handle,
                    user_handle,
                    contact_request_message,
                };

                command_queue.push(command_data, delay);
                Ok(None)
            }
        } else {
            log_info!(
                "Skipping connection attempt, verified connection already exists to {service_id}"
            );
            Ok(None)
        }
    }

    pub fn forget_user(&mut self, user_handle: UserHandle) -> Result<()> {
        assert!(user_handle != self.owner);
        match self.profile.remove_user(user_handle.into()) {
            Ok(()) => {
                if let Some(user_data) = self.users.remove(&user_handle) {
                    let _ = self.service_id_to_user_handle.remove(&user_data.service_id);
                    if let Some(connection_handle) =
                        self.v3.packet_handler.forget_user(&user_data.service_id)
                    {
                        self.v3.to_remove.insert(connection_handle);
                    }
                    let _ = self.v3.to_retry.remove(&user_handle);
                }
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn accept_contact_request(&mut self, user_handle: UserHandle) -> Result<()> {
        let user_data = self
            .users
            .get_mut(&user_handle)
            .context(format!("Unknown user {user_handle}"))?;
        self.profile
            .set_user_type(user_handle.into(), UserType::Allowed)
            .expect("Converting Requesting user to Allowed user should not fail");
        user_data.user_type = UserType::Allowed;

        let service_id = &user_data.service_id;

        let mut replies: Vec<Packet> = Default::default();
        let connection_handle = self
            .v3
            .packet_handler
            .accept_contact_request(service_id.clone(), &mut replies)?;

        // todo: these need to trigger a user online/offline callback
        if let Some(connection) = self.v3.open_connections.get_mut(&connection_handle) {
            connection.write_packets.append(&mut replies);
        }

        Ok(())
    }

    pub fn reject_contact_request(&mut self, user_handle: UserHandle) -> Result<()> {
        let user_data = self
            .users
            .get_mut(&user_handle)
            .context(format!("Unknown user {user_handle}"))?;
        self.profile
            .set_user_type(user_handle.into(), UserType::Allowed)
            .expect("Converting Requesting user to Rejected user should not fail");
        user_data.user_type = UserType::Rejected;

        let service_id = &user_data.service_id;

        let mut replies: Vec<Packet> = Default::default();
        let connection_handle = self
            .v3
            .packet_handler
            .reject_contact_request(service_id.clone(), &mut replies)?;

        if let Some(connection) = self.v3.open_connections.get_mut(&connection_handle) {
            connection.write_packets.append(&mut replies);
            self.v3.to_remove.insert(connection_handle);
        }

        Ok(())
    }

    pub fn send_message(
        &mut self,
        user_handle: UserHandle,
        message_text: rico_protocol::v3::message::chat_channel::MessageText,
    ) -> Result<tego_message_id> {
        let user_data = self
            .users
            .get_mut(&user_handle)
            .context(format!("Unknown user {user_handle}"))?;

        let service_id = &user_data.service_id;

        let mut replies: Vec<Packet> = Default::default();
        let (connection_handle, message_handle) = self.v3.packet_handler.send_message(
            service_id.clone(),
            message_text.clone(),
            None,
            &mut replies,
        )?;

        let connection = self
            .v3
            .open_connections
            .get_mut(&connection_handle)
            .context(format!(
                "No connection for Connectionhandle {connection_handle}"
            ))?;
        connection.write_packets.append(&mut replies);

        let message_id = user_data.next_message_id();
        user_data
            .queued_messages
            .push_back(UnAckedMessage::ChatMessage {
                message_id,
                message_handle,
                timestamp: std::time::Instant::now(),
                message_text,
            });
        Ok(message_id)
    }

    pub fn send_file_transfer_request(
        &mut self,
        user_handle: UserHandle,
        file_path: PathBuf,
    ) -> Result<(tego_file_transfer_id, tego_file_size)> {
        // we only deal in absolute paths
        bail_if!(!file_path.is_absolute());

        bail_if!(file_path.is_dir());

        let user_data = self
            .users
            .get_mut(&user_handle)
            .context(format!("Unknown user {user_handle}"))?;

        let service_id = &user_data.service_id;

        let file_upload = FileUpload::new(file_path)?;
        let file_name = &file_upload.name;
        let file_size = file_upload.size;
        let file_hash = &file_upload.hash;

        //construct reply packets
        let mut replies: Vec<Packet> = Vec::with_capacity(1);
        let (connection_handle, file_transfer_handle) =
            self.v3.packet_handler.send_file_transfer_request(
                service_id.clone(),
                file_name.clone(),
                file_size,
                *file_hash,
                &mut replies,
            )?;
        let connection = self
            .v3
            .open_connections
            .get_mut(&connection_handle)
            .context("missing Connection struct")?;

        // queue packets for writing
        connection.write_packets.append(&mut replies);

        // queue copies of requests to resend in event of reconnect
        let file_transfer_id = user_data.next_message_id();
        user_data
            .file_transfer_id_to_handle
            .insert(file_transfer_id, file_transfer_handle);
        user_data
            .file_transfer_handle_to_id
            .insert(file_transfer_handle, file_transfer_id);
        user_data
            .queued_messages
            .push_back(UnAckedMessage::FileTransferRequest {
                file_transfer_id,
                file_transfer_handle,
                file_upload,
            });
        Ok((file_transfer_id, file_size))
    }

    pub fn accept_file_transfer_request(
        &mut self,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
        dest_path: PathBuf,
    ) -> Result<()> {
        let user_data = self
            .users
            .get_mut(&user_handle)
            .context(format!("Unknown user {user_handle}"))?;

        let service_id = &user_data.service_id;
        let file_transfer_handle = *user_data
            .file_transfer_id_to_handle
            .get(&file_transfer_id)
            .context(format!(
                "no file transfer associated with id {file_transfer_id}"
            ))?;

        // construct reply packets
        let mut replies: Vec<Packet> = Vec::with_capacity(1);
        let connection_handle = self.v3.packet_handler.accept_file_transfer_request(
            service_id,
            file_transfer_handle,
            &mut replies,
        )?;

        // setup file download
        let connection = self
            .v3
            .open_connections
            .get_mut(&connection_handle)
            .context("missing Connection struct")?;

        let file_download = connection
            .file_downloads
            .get_mut(&file_transfer_handle)
            .context("missing FileDownload struct")?;
        file_download.start(dest_path)?;

        // queue packets for writing
        connection.write_packets.append(&mut replies);

        Ok(())
    }

    pub fn reject_file_transfer_request(
        &mut self,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
    ) -> Result<()> {
        let user_data = self
            .users
            .get_mut(&user_handle)
            .context(format!("Unknown user {user_handle}"))?;

        let service_id = &user_data.service_id;
        let file_transfer_handle = *user_data
            .file_transfer_id_to_handle
            .get(&file_transfer_id)
            .context(format!(
                "no file transfer associated with id {file_transfer_id}"
            ))?;

        // construct reply packets
        let mut replies: Vec<Packet> = Vec::with_capacity(1);
        let connection_handle = self.v3.packet_handler.reject_file_transfer_request(
            service_id,
            file_transfer_handle,
            &mut replies,
        )?;

        // remove our file download struct
        let connection = self
            .v3
            .open_connections
            .get_mut(&connection_handle)
            .context("missing Connection struct")?;
        connection
            .file_downloads
            .remove(&file_transfer_handle)
            .context("missing FileDownload struct")?;

        // queue packets for writing
        connection.write_packets.append(&mut replies);
        Ok(())
    }

    pub fn cancel_file_transfer_request(
        &mut self,
        user_handle: UserHandle,
        file_transfer_id: tego_file_transfer_id,
    ) -> Result<tego_file_transfer_direction> {
        let user_data = self
            .users
            .get_mut(&user_handle)
            .context(format!("Unknown user {user_handle}"))?;

        let service_id = &user_data.service_id;
        let file_transfer_handle = *user_data
            .file_transfer_id_to_handle
            .get(&file_transfer_id)
            .context(format!(
                "no file transfer associated with id {file_transfer_id}"
            ))?;

        // construct reply packets
        let mut replies: Vec<Packet> = Vec::with_capacity(1);
        let connection_handle = self.v3.packet_handler.cancel_file_transfer(
            service_id,
            file_transfer_handle,
            false,
            &mut replies,
        )?;

        // remove our file download/upload struct
        let connection = self
            .v3
            .open_connections
            .get_mut(&connection_handle)
            .context("missing Connection struct")?;

        // remove our handle <-> id mappings
        let _ = user_data
            .file_transfer_handle_to_id
            .remove(&file_transfer_handle);
        let _ = user_data
            .file_transfer_id_to_handle
            .remove(&file_transfer_id);

        // remove un'ackd request if present
        // todo: user queued_messagse.iter().position
        for i in 0..user_data.queued_messages.len() {
            if let UnAckedMessage::FileTransferRequest {
                file_transfer_id: candidate_file_transfer_id,
                ..
            } = user_data.queued_messages[i]
            {
                if candidate_file_transfer_id == file_transfer_id {
                    let _ = user_data.queued_messages.remove(i);
                    break;
                }
            }
        }

        let direction = if connection
            .file_downloads
            .remove(&file_transfer_handle)
            .is_some()
        {
            tego_file_transfer_direction::tego_file_transfer_direction_receiving
        } else {
            // it's possible an upload never made it to the file_uploads list
            // if local user cancels before remote user accepts, so missing
            // file_upload is not an error
            let _ = connection.file_uploads.remove(&file_transfer_handle);
            tego_file_transfer_direction::tego_file_transfer_direction_sending
        };

        // queue packets for writing
        connection.write_packets.append(&mut replies);

        Ok(direction)
    }

    //
    // Event Handlers
    //

    pub fn on_connect_complete(
        &mut self,
        connect_handle: tor_provider::ConnectHandle,
        stream: OnionStream,
    ) -> Result<()> {
        let PendingConnection {
            user_handle,
            contact_request_message,
        } = self
            .pending_connections
            .remove(&connect_handle)
            .context("Received TorEvent::ConnectComplete for unknown ConnectHandle")?;

        let user_data = self
            .users
            .get(&user_handle)
            .context("User does not exist")?;
        let service_id = &user_data.service_id;

        let packet_handler = &mut self.v3.packet_handler;

        if !packet_handler.has_verified_connection(service_id) {
            log_info!("Connected to {service_id:?}");
            let mut write_packets: Vec<Packet> = Default::default();
            let connection_handle = packet_handler.new_outgoing_connection(
                service_id.clone(),
                contact_request_message,
                &mut write_packets,
            )?;
            stream.set_nonblocking(true)?;
            let connection = Connection {
                service_id: Some(service_id.clone()),
                user_handle: Some(user_handle),
                stream,
                read_bytes: Default::default(),
                read_packets: Default::default(),
                write_packets,
                file_downloads: Default::default(),
                file_uploads: Default::default(),
            };

            self.v3
                .open_connections
                .insert(connection_handle, connection);
        } else {
            log_info!(
                "Connected to {service_id:?} but verified connection already exists, dropping new connection"
            );
        }
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

    pub fn on_connect_failed(
        &mut self,
        connect_handle: tor_provider::ConnectHandle,
        command_queue: &mut CommandQueue,
    ) -> Result<()> {
        let PendingConnection {
            user_handle,
            contact_request_message,
        } = self
            .pending_connections
            .remove(&connect_handle)
            .context("Received TorEvent::ConnectFailed for unknown ConnectHandle")?;

        if let Some(user_data) = self.users.get_mut(&user_handle) {
            user_data.connection_failures += 1;

            let failure_count = user_data.connection_failures;
            let service_id = &user_data.service_id;
            // delay before trying to connect in seconds
            let delay = Self::retry_delay(failure_count);

            log_info!(
                "Connect attempt {failure_count} to {service_id:?} failed; try again in {delay:?}"
            );

            let command_data = CommandData::ConnectContact {
                session_handle: self.session_handle,
                user_handle,
                contact_request_message,
            };

            command_queue.push(command_data, delay);
        }
        Ok(())
    }
}
