// std
use std::boxed::Box;
use std::path::{Path, PathBuf};

// extern
use rusqlite::{params, Connection, OpenFlags, Statement};
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
    const fn new(major: i64, minor: i64, patch: i64) -> Result<Self, Error> {
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
        let profile = Profile::new(path, password)?;
        let conn = &profile.conn;

        //
        // Add our host user
        //
        let identity_private_key = v3_profile.private_key;
        let identity_public_key = Ed25519PublicKey::from_private_key(&identity_private_key);

        // insert keys into db
        let identity_private_key_rowid =
            db::insert_ed25519_private_key(conn, &identity_private_key.to_bytes())?;
        let identity_public_key_rowid =
            db::insert_ed25519_public_key(conn, identity_public_key.as_bytes())?;

        // create owner's user profile
        let owner_user_profile_rowid = db::insert_user_profile(
            conn, nickname, None, // pet_name
            None, // pronouns
            None, // avatar
            None, //status
            None, //description
        )?;

        // insert user
        db::insert_user(
            conn,
            db::UserType::Owner,
            Some(owner_user_profile_rowid),
            identity_public_key_rowid,
            Some(identity_private_key_rowid),
            None, // remote endpoint ed25519 public key
            None, // remote endpoint x25519 private key
            None, // local endpoint ed25519 private key
            None, // local endpoint x25519 public key
        )?;

        for (service_id, user) in v3_profile.users {
            // insert public key
            let identity_public_key = Ed25519PublicKey::from_service_id(&service_id).unwrap();
            let identity_public_key_rowid =
                db::insert_ed25519_public_key(conn, identity_public_key.as_bytes())?;

            //insert user profile
            let nickname = service_id.to_string();
            let nickname = nickname.as_str();
            let pet_name = user.nickname.as_str();

            let user_profile_rowid = db::insert_user_profile(
                conn,
                nickname,
                Some(pet_name),
                None, // pronouns
                None, // avatar
                None, // status
                None, // description
            )?;

            // map legacy user types to new user types
            let user_type = user.user_type;
            let user_type = match user_type {
                v3::profile::UserType::Allowed => db::UserType::Allowed,
                v3::profile::UserType::Requesting | v3::profile::UserType::Pending => {
                    db::UserType::Requesting
                }
                v3::profile::UserType::Rejected => db::UserType::Rejected,
                v3::profile::UserType::Blocked => db::UserType::Blocked,
            };

            // insert user
            db::insert_user(
                conn,
                user_type.clone(),
                Some(user_profile_rowid),
                identity_public_key_rowid,
                None, // identity ed25519 private key
                None, // remote endpoint ed25519 public key
                None, // remote endpoint x25519 private key
                None, // local endpoint ed25519 private key
                None, // local endpoint x25519 public key
            )?;
        }

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
        let (major, minor, patch) = self
            .conn
            .query_one(
                "SELECT major, minor, patch FROM db_versions ORDER BY rowid DESC LIMIT 1;",
                [],
                |row| Ok((row.get(0), row.get(1), row.get(2))),
            )
            .map_err(Error::QueryFailure)?;
        let major = major.map_err(Error::QueryFailure)?;
        let minor = minor.map_err(Error::QueryFailure)?;
        let patch = patch.map_err(Error::QueryFailure)?;

        Version::new(major, minor, patch)
    }

    //
    // Conversation
    //

    pub fn add_conversation(
        &mut self,
        conversation: Conversation,
    ) -> Result<ConversationHandle, Error> {
        Err(Error::NotImplemented)
    }

    pub fn get_conversations(&self) -> Result<Vec<(Conversation, ConversationHandle)>, Error> {
        Err(Error::NotImplemented)
    }

    pub fn delete_conversation(
        &mut self,
        conversation_handle: ConversationHandle,
    ) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    //
    // Profile
    //

    pub fn add_user_profile(
        &mut self,
        user_handle: UserHandle,
        profile: UserProfile,
    ) -> Result<UserProfileHandle, Error> {
        Err(Error::NotImplemented)
    }

    pub fn update_user_profile(
        &mut self,
        profile_handle: UserProfileHandle,
        profile: UserProfile,
    ) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    //
    // User
    //

    pub fn add_user(&mut self, user: User) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    pub fn remove_user(
        &mut self,
        identity_ed25519_public_key: &Ed25519PublicKey,
    ) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    pub fn update_user_remote_endpoint_keys(
        &mut self,
        remote_endpoint_ed25519_public_key: Ed25519PublicKey,
        remote_endpoint_x25519_private_key: X25519PrivateKey,
    ) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    pub fn update_user_local_endpoint_keys(
        &mut self,
        local_endpoint_ed25519_private_key: Ed25519PrivateKey,
        local_endpoint_x25519_public_key: X25519PublicKey,
    ) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    //
    // Messages
    //

    pub fn add_message_record(
        &mut self,
        message_record: MessageRecord,
    ) -> Result<MessageRecordHandle, Error> {
        Err(Error::NotImplemented)
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
pub struct UserProfileHandle(i64);
pub struct UserProfile {
    nickname: String,
    pet_name: Option<String>,
    pronouns: Option<String>,
    avatar: Option<Avatar>,
    status: Option<String>,
    description: Option<String>,
}

// Avatar
pub struct AvatarHandle(i64);
pub struct Avatar {
    // 256x256 8-bit channel RGBA image in row-major order
    rgba_data: Box<[u8; Self::BYTES]>,
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
pub struct UserHandle(i64);
pub struct User {
    pub user_type: UserType,
    pub user_profile: UserProfile,
    pub identity_ed25519_public_key: Ed25519PublicKey,
    pub identity_ed25519_private_key: Option<Ed25519PrivateKey>,
    pub remote_endpoint_ed25519_public_key: Option<Ed25519PublicKey>,
    pub remote_endpoint_x25519_private_key: Option<X25519PrivateKey>,
    pub local_endpoint_ed25519_private_key: Option<Ed25519PrivateKey>,
    pub local_endpoint_x25519_public_key: Option<X25519PrivateKey>,
}

pub enum UserType {
    Owner,
    Allowed,
    Requesting,
    Rejected,
    Blocked,
}

//
// Conversation
//
pub struct ConversationHandle(i64);
pub struct Conversation {
    pub conversation_type: ConversationType,
    pub conversation_members: Vec<Ed25519PublicKey>,
    pub conversation_key: Sha256Sum,
}

pub enum ConversationType {
    LegacyV3,
    EphemeralDirectMessage,
    PersistentDirectMessage,
}

impl From<ConversationType> for i64 {
    fn from(value: ConversationType) -> i64 {
        match value {
            ConversationType::LegacyV3 => 0i64,
            ConversationType::EphemeralDirectMessage => 1i64,
            ConversationType::PersistentDirectMessage => 2i64,
        }
    }
}

// Messages

pub struct MessageRecordHandle(i64);
pub struct RecordSequence(i64);
pub struct MessageSequence(i64);
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

pub struct FileSize(i64);
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

pub struct Salt([u8; 32]);

//
// Sha256Sum
//

#[derive(PartialEq)]
pub struct Sha256Sum([u8; 32]);
