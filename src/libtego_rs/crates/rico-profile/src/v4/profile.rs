// std
use std::boxed::Box;
use std::path::Path;

// extern
use rusqlite::{params, Connection, OpenFlags, Statement};
use tor_interface::tor_crypto::{
    Ed25519PrivateKey, Ed25519PublicKey, V3OnionServiceId, X25519PrivateKey, X25519PublicKey,
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
    // Row select methods
    //
}

//
// UserProfile
//
pub struct UserProfile {
    nickname: String,
    pet_name: Option<String>,
    pronouns: Option<String>,
    avatar: Option<Avatar>,
    status: Option<String>,
    description: Option<String>,
}

// Avatar
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
pub struct User {
    user_type: UserType,
    user_profile: UserProfile,
    identity_ed25519_public_key: Ed25519PublicKey,
    identity_ed25519_private_key: Option<Ed25519PrivateKey>,
    remote_endpoint_ed25519_public_key: Option<Ed25519PublicKey>,
    remote_endpoint_x25519_private_key: Option<X25519PrivateKey>,
    local_endpoint_ed25519_private_key: Option<Ed25519PrivateKey>,
    local_endpoint_x25519_public_key: Option<X25519PrivateKey>,
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
pub struct Conversation {
    conversation_type: ConversationType,
    conversation_members: Vec<Ed25519PublicKey>,
}

pub enum ConversationType {
    LegacyV3,
    EphemeralDirectMessage,
    PersistentDirectMessage,
}

//
// Salt
//

pub struct Salt {
    data: [u8; 32],
}
