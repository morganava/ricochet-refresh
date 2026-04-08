// std
use std::boxed::Box;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// extern
use rusqlite::{Connection, OpenFlags};
use time::UtcDateTime;
use tor_interface::tor_crypto::{
    Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, X25519PrivateKey, X25519PublicKey,
};

// internal
#[cfg(feature = "v3-profile")]
use crate::v3;
use crate::v4::db;
use crate::v4::error::Error;

//
// Profile: semantic version of the schema of the databases
//

#[derive(Debug, PartialEq)]
pub struct Version {
    major: i64,
    minor: i64,
    patch: i64,
}

impl Version {
    pub const LATEST: Version = Version {
        major: 0i64,
        minor: 1i64,
        patch: 0i64,
    };
}

impl Version {
    pub(crate) const fn new(major: i64, minor: i64, patch: i64) -> Result<Self, Error> {
        if major < 0 || minor < 0 || patch < 0 {
            Err(Error::InvalidSemanticVersion(major, minor, patch))
        } else {
            Ok(Self {
                major,
                minor,
                patch,
            })
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}.{1}.{2}", self.major, self.minor, self.patch)
    }
}

//
// Profile and associated types
//

pub struct Profile {
    pub(crate) conn: Connection,
}

impl Profile {
    pub fn new(path: &Path, password: &str) -> Result<Profile, Error> {
        let open_flags: OpenFlags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        let conn =
            Connection::open_with_flags(path, open_flags).map_err(Error::DatabaseOpenFailure)?;

        // set the our password
        db::set_password(&conn, password)?;

        // create tabless
        db::create_tables(&conn)?;

        // insert the version row
        let version = Version::LATEST;
        db::insert_db_version(&conn, version.major, version.minor, version.patch)?;
        std::mem::drop(conn);

        Self::open(path, password)
    }

    #[cfg(feature = "v3-profile")]
    pub fn new_from_v3_profile(
        v3_profile: v3::profile::Profile,
        nickname: &str,
        path: &Path,
        password: &str,
    ) -> Result<Profile, Error> {
        // todo, write profile to a temp file and move after successful creation
        let mut profile = Profile::new(path, password)?;
        let tx = profile
            .conn
            .transaction()
            .map_err(Error::TransactionCreateFailure)?;
        //
        // Add our host user
        //

        let host_identity_ed25519_private_key = v3_profile.private_key;
        let nickname = nickname.to_string();

        let host_identity_ed25519_public_key =
            Ed25519PublicKey::from_private_key(&host_identity_ed25519_private_key);
        let host_identity_ed25519_private_key = Some(host_identity_ed25519_private_key);

        let host_user = User {
            user_type: UserType::Owner,
            user_profile: UserProfile {
                nickname,
                pet_name: None,
                pronouns: None,
                avatar: None,
                status: None,
                description: None,
            },
            identity_ed25519_public_key: host_identity_ed25519_public_key.clone(),
            identity_ed25519_private_key: host_identity_ed25519_private_key,
            remote_endpoint_ed25519_public_key: None,
            remote_endpoint_x25519_private_key: None,
            local_endpoint_ed25519_private_key: None,
            local_endpoint_x25519_public_key: None,
        };

        let host_user_handle = db::insert_user(&tx, &host_user)?;

        for (service_id, user) in v3_profile.users {
            // map legacy UserType to v4 UserType
            let user_type = user.user_type;
            let user_type = match user_type {
                v3::profile::UserType::Allowed => UserType::Allowed,
                v3::profile::UserType::Requesting | v3::profile::UserType::Pending => {
                    UserType::Requesting
                }
                v3::profile::UserType::Rejected => UserType::Rejected,
                v3::profile::UserType::Blocked => UserType::Blocked,
            };
            let nickname = service_id.to_string();
            let pet_name = Some(user.nickname);

            // map legacy user types to new user types
            let identity_ed25519_public_key =
                Ed25519PublicKey::from_service_id(&service_id).unwrap();

            let user = User {
                user_type,
                user_profile: UserProfile {
                    nickname,
                    pet_name,
                    pronouns: None,
                    avatar: None,
                    status: None,
                    description: None,
                },
                identity_ed25519_public_key: identity_ed25519_public_key.clone(),
                identity_ed25519_private_key: None,
                remote_endpoint_ed25519_public_key: None,
                remote_endpoint_x25519_private_key: None,
                local_endpoint_ed25519_private_key: None,
                local_endpoint_x25519_public_key: None,
            };

            // insert user
            let user_handle = db::insert_user(&tx, &user)?;

            //
            // insert default conversations
            //
            let conversation_members_public_keys: BTreeSet<Ed25519PublicKey> = [
                host_identity_ed25519_public_key.clone(),
                identity_ed25519_public_key,
            ]
            .into();

            // ephemeral conversation
            let ephemeral_conversation_key = rico_protocol::v4::payload::conversation_key(
                ConversationType::EphemeralDirectMessage,
                &conversation_members_public_keys,
            );
            let ephemeral_conversation = Conversation {
                conversation_type: ConversationType::EphemeralDirectMessage,
                conversation_members: [host_user_handle, user_handle].into(),
                conversation_key: Sha256Sum(ephemeral_conversation_key),
            };
            db::insert_conversation(&tx, ephemeral_conversation)?;

            // persistent conversation
            let persistent_conversation_key = rico_protocol::v4::payload::conversation_key(
                ConversationType::PersistentDirectMessage,
                &conversation_members_public_keys,
            );
            let persistent_conversation = Conversation {
                conversation_type: ConversationType::PersistentDirectMessage,
                conversation_members: [host_user_handle, user_handle].into(),
                conversation_key: Sha256Sum(persistent_conversation_key),
            };
            db::insert_conversation(&tx, persistent_conversation)?;
        }
        tx.commit().map_err(Error::TransactionCommitFailure)?;

        Ok(profile)
    }

    pub fn open(path: &Path, password: &str) -> Result<Profile, Error> {
        let open_flags: OpenFlags =
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        let conn =
            Connection::open_with_flags(path, open_flags).map_err(Error::DatabaseOpenFailure)?;

        // set password
        db::set_password(&conn, password)?;

        let profile = Profile { conn };

        match profile.get_version()? {
            Version::LATEST => Ok(profile),
            // todo: we can add migration functions here when the version number needs to be bumped
            version => Err(Error::UnknownProfileVersion(version)),
        }
    }

    //
    // Public read/write methods
    //

    //
    // Version
    //
    pub fn get_version(&self) -> Result<Version, Error> {
        db::select_newest_db_version(&self.conn)
    }

    //
    // Conversation
    //

    pub fn add_conversation(
        &mut self,
        conversation: Conversation,
    ) -> Result<ConversationHandle, Error> {
        let tx = self.conn.transaction()?;
        let conversation_handle = db::insert_conversation(&tx, conversation)?;
        tx.commit()?;
        Ok(conversation_handle)
    }

    pub fn get_conversations(&self) -> Result<Vec<(Conversation, ConversationHandle)>, Error> {
        db::select_all_conversations(&self.conn)
    }

    pub fn remove_conversation(
        &mut self,
        conversation_handle: ConversationHandle,
    ) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        db::delete_conversation(&tx, conversation_handle)?;
        tx.commit()?;
        Ok(())
    }

    //
    // Profile
    //

    pub fn get_user_profile(&self, user_handle: UserHandle) -> Result<UserProfile, Error> {
        db::select_user_profile_by_user_handle(&self.conn, user_handle)
    }

    pub fn set_user_profile(
        &mut self,
        user_handle: UserHandle,
        user_profile: UserProfile,
    ) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        db::update_user_profile(&tx, user_handle, user_profile)?;
        tx.commit()?;
        Ok(())
    }

    //
    // User
    //

    pub fn add_user(&mut self, user: User) -> Result<UserHandle, Error> {
        let tx = self.conn.transaction()?;
        let user_handle = db::insert_user(&tx, &user)?;
        tx.commit()?;
        Ok(user_handle)
    }

    pub fn get_users(&self) -> Result<Vec<(User, UserHandle)>, Error> {
        db::select_all_users(&self.conn)
    }

    pub fn remove_user(&mut self, user_handle: UserHandle) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        db::delete_user(&tx, user_handle)?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_user_remote_endpoint_keys(
        &mut self,
        user_handle: UserHandle,
        remote_endpoint_ed25519_public_key: Ed25519PublicKey,
        remote_endpoint_x25519_private_key: X25519PrivateKey,
    ) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        db::update_remote_endpoint_keys(
            &tx,
            user_handle,
            remote_endpoint_ed25519_public_key,
            remote_endpoint_x25519_private_key,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_user_local_endpoint_keys(
        &mut self,
        user_handle: UserHandle,
        local_endpoint_ed25519_private_key: Ed25519PrivateKey,
        local_endpoint_x25519_public_key: X25519PublicKey,
    ) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        db::update_local_endpoint_keys(
            &tx,
            user_handle,
            local_endpoint_ed25519_private_key,
            local_endpoint_x25519_public_key,
        )?;
        tx.commit()?;
        Ok(())
    }

    //
    // Messages
    //

    pub fn add_message_record(
        &mut self,
        message_record: MessageRecord,
    ) -> Result<MessageRecordHandle, Error> {
        let tx = self.conn.transaction()?;
        let message_record_handle = db::insert_message_record(&tx, &message_record)?;
        tx.commit()?;
        Ok(message_record_handle)
    }

    // get all the message records in conversation sorted by created_timestamp optionally:
    // - author'd by a particular user
    // - older than a particular creation timestampp AND
    // - older than a particular MessageRecordHandle AND
    // - limit number of returned records
    //
    // returns messages and the oldest MessageRecordHandle in the set
    // in creation order
    pub fn get_message_records(
        &self,
        conversation_handle: ConversationHandle,
        author: Option<UserHandle>,
        older_than_creation_timestamp: Option<UtcDateTime>,
        older_than_message_record_handle: Option<MessageRecordHandle>,
        limit: Option<usize>,
    ) -> Result<(Vec<MessageRecord>, MessageRecordHandle), Error> {
        Err(Error::NotImplemented)
    }
}

//
// UserProfile
//
pub type UserProfileHandle = db::UserProfileRowID;
#[derive(Debug)]
pub struct UserProfile {
    pub nickname: String,
    pub pet_name: Option<String>,
    pub pronouns: Option<String>,
    pub avatar: Option<Avatar>,
    pub status: Option<String>,
    pub description: Option<String>,
}

// Avatar
pub type AvatarHandle = db::AvatarRowID;
#[derive(Debug)]
pub struct Avatar {
    // 256x256 8-bit channel RGBA image in row-major order
    pub rgba_data: Box<[u8; Self::BYTES]>,
}

impl Avatar {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 256;
    pub const CHANNELS: usize = 4;
    pub const BYTES: usize = Self::WIDTH * Self::HEIGHT * Self::CHANNELS;
}

//
// User
//
pub type UserHandle = db::UserRowID;
#[derive(Debug)]
pub struct User {
    pub user_type: UserType,
    pub user_profile: UserProfile,
    pub identity_ed25519_public_key: Ed25519PublicKey,
    pub identity_ed25519_private_key: Option<Ed25519PrivateKey>,
    pub remote_endpoint_ed25519_public_key: Option<Ed25519PublicKey>,
    pub remote_endpoint_x25519_private_key: Option<X25519PrivateKey>,
    pub local_endpoint_ed25519_private_key: Option<Ed25519PrivateKey>,
    pub local_endpoint_x25519_public_key: Option<X25519PublicKey>,
}

#[derive(Clone, Copy, Debug)]
pub enum UserType {
    Owner,
    Allowed,
    Requesting,
    Rejected,
    Blocked,
}

impl From<UserType> for i64 {
    fn from(value: UserType) -> i64 {
        match value {
            UserType::Owner => 0i64,
            UserType::Allowed => 1i64,
            UserType::Requesting => 2i64,
            UserType::Rejected => 3i64,
            UserType::Blocked => 4i64,
        }
    }
}

impl TryFrom<i64> for UserType {
    type Error = Error;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0i64 => Ok(UserType::Owner),
            1i64 => Ok(UserType::Allowed),
            2i64 => Ok(UserType::Requesting),
            3i64 => Ok(UserType::Rejected),
            4i64 => Ok(UserType::Blocked),
            _ => Err(Error::TypeConversionFailed(format!("{value}"), "UserType")),
        }
    }
}

impl rusqlite::ToSql for UserType {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
        let value: i64 = (*self).into();
        Ok(value.into())
    }
}

impl rusqlite::types::FromSql for UserType {
    fn column_result(
        value: rusqlite::types::ValueRef<'_>,
    ) -> Result<Self, rusqlite::types::FromSqlError> {
        let value = i64::column_result(value)?;
        match UserType::try_from(value) {
            Ok(value) => Ok(value),
            Err(_) => Err(rusqlite::types::FromSqlError::OutOfRange(value)),
        }
    }
}

//
// Conversation
//
pub type ConversationHandle = db::ConversationRowID;
pub struct Conversation {
    pub conversation_type: ConversationType,
    pub conversation_members: BTreeSet<UserHandle>,
    pub conversation_key: Sha256Sum,
}

pub type ConversationType = rico_protocol::v4::ConversationType;

// Messages

pub type MessageRecordHandle = db::MessageRecordRowID;
pub type RecordSequence = db::RecordSequence;
pub type MessageSequence = db::MessageSequence;
pub struct MessageRecord {
    pub conversation_handle: ConversationHandle,
    pub user_handle: UserHandle,
    pub record_sequence: RecordSequence,
    pub message_sequence: MessageSequence,
    pub create_timestamp: UtcDateTime,
    pub modify_timestamp: UtcDateTime,
    pub message_content_salt: Salt,
    pub message_content: MessageContent,
    pub signature: Ed25519Signature,
}

pub type FileSize = db::FileSize;
pub enum MessageContent {
    Modified {
        original_message_content_hash: Sha256Sum,
        original_message_record_signature: Ed25519Signature,
    },
    Text {
        text: String,
    },
    FileShare {
        file_data_salt: Salt,
        file_size: FileSize,
        file_data_hash: Sha256Sum,
        file_path: Option<PathBuf>,
    },
}

//
// Salt
//

pub struct Salt(pub [u8; 32]);

//
// Sha256Sum
//

#[derive(PartialEq)]
pub struct Sha256Sum(pub [u8; 32]);
