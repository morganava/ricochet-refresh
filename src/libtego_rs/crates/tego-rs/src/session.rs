// standard
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

// extern
use anyhow::{Context as AnyhowContext, Result};
use rico_profile::v4::profile::{Profile, User, UserType};
use rico_protocol::v3::packet_handler;
use rico_protocol::v3::packet_handler::{Packet, PacketHandler};
use rico_protocol::v3::RICOCHET_PORT;
use tor_interface::tor_crypto::{Ed25519PrivateKey, V3OnionServiceId};
use tor_interface::tor_provider;
use tor_interface::tor_provider::{OnionListener, OnionStream, TorProvider};

// internal
use crate::command_queue::{CommandData, CommandQueue};
use crate::macros::*;

pub(crate) type SessionHandle = i64;
pub(crate) type UserHandle = i64;

pub(crate) struct Session {
    session_handle: SessionHandle,
    profile: Profile,

    // Users
    users: BTreeMap<UserHandle, UserData>,
    owner: UserHandle,
    allowed_users: BTreeSet<UserHandle>,
    pending_users: BTreeSet<UserHandle>,
    requesting_users: BTreeSet<UserHandle>,
    rejected_users: BTreeSet<UserHandle>,
    blocked_users: BTreeSet<UserHandle>,

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
    pub(crate) user_type: UserType,
    pub(crate) pet_name: String,
    pub(crate) service_id: V3OnionServiceId,

    // connection data
    connection_failures: usize,
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
}

#[derive(Debug)]
struct Connection {
    pub service_id: Option<V3OnionServiceId>,
    pub stream: OnionStream,
    // buffer of unhandled read bytes
    pub read_bytes: Vec<u8>,
    // buffer of read Packets to handle
    pub read_packets: Vec<Packet>,
    // buffer of packets to write
    pub write_packets: Vec<Packet>,
    // todo: maybe these should also just be a single FileTransfer
    // pending and in-progress file downloads
    pub file_downloads:
        BTreeMap<rico_protocol::v3::packet_handler::FileTransferHandle, FileDownload>,
    // // pending and in-process file uploads
    pub file_uploads: BTreeMap<rico_protocol::v3::packet_handler::FileTransferHandle, FileUpload>,
}

// todo: migrate from event_loop_task
#[derive(Debug)]
pub struct FileDownload {}

// todo: migrate from event_loop_task
#[derive(Debug)]
pub struct FileUpload {}

impl Session {
    pub fn new(session_handle: SessionHandle, profile: Profile) -> Result<Self> {
        let mut users: BTreeMap<UserHandle, UserData> = Default::default();
        let mut owner: Option<UserHandle> = None;
        let mut allowed_users: BTreeSet<UserHandle> = Default::default();
        let mut pending_users: BTreeSet<UserHandle> = Default::default();
        let mut requesting_users: BTreeSet<UserHandle> = Default::default();
        let mut rejected_users: BTreeSet<UserHandle> = Default::default();
        let mut blocked_users: BTreeSet<UserHandle> = Default::default();
        let mut owner_private_key: Option<Ed25519PrivateKey> = None;

        // get all our user info out of the profile
        for (user, user_handle) in profile.get_users()? {
            let user_handle = user_handle.0;
            let user_type = user.user_type;
            let service_id = user.identity_v3_onion_service_id();
            let pet_name = if let Some(pet_name) = user.user_profile.pet_name {
                pet_name
            } else {
                user.user_profile.nickname
            };

            let user_data = UserData {
                user_type,
                pet_name,
                service_id,
                connection_failures: 0usize,
            };
            if let Some(_) = users.insert(user_handle, user_data) {
                unreachable!("Profile should not have duplicate UserHandles in users table");
            }

            let _ = match user_type {
                UserType::Owner => {
                    assert!(owner.is_none(), "Profile should only have one Owner");
                    owner = Some(user_handle);
                    owner_private_key = user.identity_ed25519_private_key;
                    false
                }
                UserType::Allowed => allowed_users.insert(user_handle),
                UserType::Pending => pending_users.insert(user_handle),
                UserType::Requesting => requesting_users.insert(user_handle),
                UserType::Rejected => rejected_users.insert(user_handle),
                UserType::Blocked => blocked_users.insert(user_handle),
            };
        }

        let owner = owner.context("Profile must have an owner")?;

        let pending_connections = Default::default();

        // v3 data
        let v3 = {
            // get 'known' contacts from pending and allowed lists
            let mut known_contacts: BTreeSet<V3OnionServiceId> = Default::default();
            for user_handle in allowed_users.iter().chain(pending_users.iter()) {
                let service_id = users.get(&user_handle).unwrap().service_id.clone();
                known_contacts.insert(service_id);
            }

            // get 'blocked' contacts from blocked list
            let mut blocked_contacts: BTreeSet<V3OnionServiceId> = Default::default();
            for user_handle in &blocked_users {
                let service_id = users.get(&user_handle).unwrap().service_id.clone();
                blocked_contacts.insert(service_id);
            }

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

            V3Data {
                listener,
                open_connections,
                packet_handler,
            }
        };

        Ok(Session {
            session_handle,
            profile,
            users,
            owner,
            allowed_users,
            pending_users,
            requesting_users,
            rejected_users,
            blocked_users,
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

    pub fn get_user_count(&self) -> usize {
        self.users.len()
    }

    pub fn get_user_handles(&self) -> Vec<UserHandle> {
        self.users.keys().cloned().collect()
    }

    pub fn get_allowed_user_handles(&self) -> Vec<UserHandle> {
        self.allowed_users.iter().cloned().collect()
    }

    pub fn get_pending_user_handles(&self) -> Vec<UserHandle> {
        self.pending_users.iter().cloned().collect()
    }

    pub fn get_user_data(&self, user_handle: UserHandle) -> Result<&UserData> {
        self.users
            .get(&user_handle)
            .context("No user with UserHandle '{user_handle}'")
    }

    pub fn get_user_data_mut(&mut self, user_handle: UserHandle) -> Result<&mut UserData> {
        self.users
            .get_mut(&user_handle)
            .context("No user with UserHandle '{user_handle}'")
    }

    pub fn get_user_type(&self, user_handle: UserHandle) -> Result<UserType> {
        Ok(self.get_user_data(user_handle)?.user_type)
    }

    pub fn get_user_identity_v3_onion_service_id(
        &self,
        user_handle: UserHandle,
    ) -> Result<V3OnionServiceId> {
        Ok(self.get_user_data(user_handle)?.service_id.clone())
    }

    // pub fn get_user_nickname(&self, user_handle: UserHandle) -> Result<String> {
    //     Ok(self.get_user_data(user_handle)?.user_profile.nickname.clone())
    // }

    pub fn get_user_pet_name(&self, user_handle: UserHandle) -> Result<String> {
        Ok(self.get_user_data(user_handle)?.pet_name.clone())
    }

    //
    // Protocol Stuffs
    //

    // Called each frame to update our internal stuff
    pub fn update_v3(
        &mut self,
        tor_provider: &mut Box<dyn TorProvider>,
        command_queue: &mut CommandQueue,
    ) -> Result<()> {
        // handle new incoming connections
        if let Some(listener) = &mut self.v3.listener {
            loop {
                match listener.accept() {
                    Ok(Some(stream)) => {
                        log_info!("New incoming connection (not implemented)");
                        // command_queue
                        //     .push(CommandData::BeginServerHandshake { stream }, Duration::ZERO);
                    }
                    Ok(None) => break,
                    Err(err) => {
                        log_error!("Error listening for new connections: {err}");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn start_onion_listener(&mut self, tor_provider: &mut Box<dyn TorProvider>) -> Result<()> {
        let mut listener = tor_provider.listener(
            self.v3.packet_handler.get_private_key(),
            RICOCHET_PORT,
            None,
        )?;
        listener.set_nonblocking(true)?;
        self.v3.listener = Some(listener);
        Ok(())
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
        if !self.v3.packet_handler.has_verified_connection(service_id) {
            log_info!("Conecting to {service_id}");
            let target_addr: tor_interface::tor_provider::TargetAddr =
                (service_id.clone(), RICOCHET_PORT).into();

            if let Ok(connect_handle) = tor_provider.connect_async(target_addr, None) {
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

        if !packet_handler.has_verified_connection(&service_id) {
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
