// std
use std::boxed::Box;
use std::path::Path;

// extern
use rusqlite::{params, Connection, OpenFlags, Statement};
use tor_interface::tor_crypto::{
    Ed25519PrivateKey, V3OnionServiceId, X25519PrivateKey, X25519PublicKey,
};

// internal
#[cfg(feature = "v3-profile")]
use crate::v3;
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
    conn: Connection,
}

impl Profile {
    pub fn new(path: &Path, password: &str) -> Result<Profile, Error> {
        let open_flags: OpenFlags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        let conn =
            Connection::open_with_flags(path, open_flags).map_err(Error::DatabaseOpenFailure)?;

        // set the our password
        conn.pragma_update(None, "key", password)
            .map_err(Error::PragmaUpdateFailure)?;

        // build our tables
        conn.execute_batch(
            "BEGIN;

            -- db_versions
            CREATE TABLE db_versions (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              major INTEGER NOT NULL CHECK(major >= 0),
              minor INTEGER NOT NULL CHECK(minor >= 0),
              patch INTEGER NOT NULL CHECK(patch >= 0)
            );

            -- user_profiles
            CREATE TABLE user_profiles (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              display_name TEXT NOT NULL,
              pronouns TEXT CHECK(LENGTH(pronouns) <= 64),
              avatar_rowid INTEGER UNIQUE REFERENCES avatars(rowid),
              status TEXT CHECK(LENGTH(status) <= 256),
              description TEXT CHECK(LENGTH(description) <= 2048)
            );

            -- avatars
            CREATE TABLE avatars (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB CHECK(LENGTH(value) = 262144)
            );

            -- users
            CREATE TABLE users (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              user_type INTEGER NOT NULL CHECK(user_type >= 0 AND user_type <= 4),
              user_profile_rowid INTEGER NOT NULL UNIQUE REFERENCES user_profiles(rowid),
              identity_ed25519_public_key_rowid INTEGER NOT NULL UNIQUE REFERENCES ed25519_public_keys(rowid),
              identity_ed25519_private_key_rowid INTEGER UNIQUE REFERENCES ed25519_private_keys(rowid),
              remote_endpoint_ed25519_public_key_rowid INTEGER UNIQUE REFERENCES ed25519_public_keys(rowid),
              remote_endpoint_x25519_private_key_rowid INTEGER UNIQUE REFERENCES x25519_private_keys(rowid),
              local_endpoint_ed25519_private_key_rowid INTEGER UNIQUE REFERENCES ed25519_private_keys(rowid),
              local_endpoint_x25519_public_key_rowid INTEGER UNIQUE REFERENCES x25519_public_keys(rowid),
              CHECK((user_type == 0) == (identity_ed25519_private_key_rowid IS NOT NULL)),
              CHECK(remote_endpoint_ed25519_public_key_rowid IS NULL == remote_endpoint_x25519_private_key_rowid IS NULL),
              CHECK(local_endpoint_ed25519_private_key_rowid IS NULL == local_endpoint_x25519_public_key_rowid IS NULL)
            );
            CREATE INDEX idx_users_identity_ed25519_public_key_rowid ON users(identity_ed25519_public_key_rowid);

            -- conversations
            CREATE TABLE conversations (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              conversation_type INTEGER NOT NULL CHECK(conversation_type >= 0 AND conversation_type <= 2),
              conversation_key_rowid INTEGER NOT NULL UNIQUE REFERENCES sha256_hashes(rowid)
            );

            -- conversation_members
            CREATE TABLE conversation_members (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              conversation_rowid INTEGER NOT NULL REFERENCES conversations(rowid),
              user_rowid INTEGER NOT NULL REFERENCES users(rowid),
              UNIQUE(conversation_rowid, user_rowid)
            );
            CREATE INDEX idx_conversation_members_conversation_rowid ON conversation_members(conversation_rowid);

            -- message_records
            CREATE TABLE message_records (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              conversation_rowid INTEGER NOT NULL REFERENCES conversations(rowid),
              user_rowid INTEGER NOT NULL REFERENCES users(rowid),
              record_sequence INTEGER NOT NULL CHECK(record_sequence >= 0),
              message_sequence INTEGER NOT NULL CHECK(message_sequence >= 0),
              create_timestamp INTEGER NOT NULL,
              modify_timestamp INTEGER NOT NULL CHECK(modify_timestamp >= create_timestamp),
              message_content_rowid INTEGER NOT NULL UNIQUE REFERENCES message_contents(rowid),
              signature_rowid INTEGER NOT NULL UNIQUE REFERENCES ed25519_signatures(rowid),
              UNIQUE(conversation_rowid, user_rowid, record_sequence)
            );
            CREATE INDEX idx_message_records_conversation_user_sequence ON message_records(conversation_rowid, user_rowid, message_sequence, record_sequence);
            CREATE INDEX idx_message_records_conversation_user_timestamp_sequence ON message_records(conversation_rowid, user_rowid, create_timestamp, message_sequence, record_sequence);

            -- message_contents
            CREATE TABLE message_contents (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              salt_rowid INTEGER NOT NULL REFERENCES salts(rowid),
              message_type INTEGER NOT NULL CHECK(message_type >= 0 AND message_type <= 2),
              modified_message_rowid INTEGER REFERENCES modified_messages(rowid),
              text_message_rowid INTEGER REFERENCES text_messages(rowid),
              file_share_message_rowid INTEGER UNIQUE REFERENCES file_share_messages(rowid),
              CHECK((message_type = 0) = (modified_message_rowid IS NOT NULL)),
              CHECK((message_type = 1) = (text_message_rowid IS NOT NULL)),
              CHECK((message_type = 2) = (file_share_message_rowid IS NOT NULL))
            );

            -- modified_messages
            CREATE TABLE modified_messages (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              original_message_content_hash_rowid INTEGER NOT NULL REFERENCES sha256_hashes(rowid),
              original_message_record_signature_rowid INTEGER NOT NULL REFERENCES ed25519_signatures(rowid)
            );

            -- text_messages
            CREATE TABLE text_messages (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              text TEXT NOT NULL
            );

            -- file_share_messages
            CREATE TABLE file_share_messages (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              file_data_salt_rowid INTEGER NOT NULL REFERENCES salts(rowid),
              file_size INTEGER NOT NULL CHECK(file_size >= 0),
              file_data_hash_rowid INTEGER NOT NULL REFERENCES sha256_hashes(rowid),
              file_path TEXT
            );

            -- salts
            CREATE TABLE salts (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB NOT NULL UNIQUE CHECK(LENGTH(value) = 32)
            );

            -- sha256_hashes
            CREATE TABLE sha256_hashes (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB NOT NULL UNIQUE CHECK(LENGTH(value) = 32)
            );

            -- ed25519_private_keys
            CREATE TABLE ed25519_private_keys (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB NOT NULL UNIQUE CHECK(LENGTH(value) = 64)
            );

            -- ed25519_public_keys
            CREATE TABLE ed25519_public_keys (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB NOT NULL UNIQUE CHECK(LENGTH(value) = 32)
            );
            CREATE INDEX idx_ed25519_public_keys_value ON ed25519_public_keys(value);

            -- ed25519_signatures
            CREATE TABLE ed25519_signatures (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB NOT NULL UNIQUE CHECK(LENGTH(value) = 64)
            );

            -- x25519_private_keys
            CREATE TABLE x25519_private_keys (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB NOT NULL UNIQUE CHECK(LENGTH(value) = 32)
            );

            -- x25519_public_keys
            CREATE TABLE x25519_public_keys (
              rowid INTEGER PRIMARY KEY AUTOINCREMENT,
              value BLOB NOT NULL UNIQUE CHECK(LENGTH(value) = 32)
            );

            COMMIT;"
        ).map_err(Error::StatementExecuteFailure)?;
        // insert the version row
        let version = Version::LATEST;
        conn.execute(
            "INSERT INTO db_versions (major, minor, patch) VALUES (?1, ?2, ?3)",
            params![version.major, version.minor, version.patch],
        )
        .map_err(Error::StatementExecuteFailure)?;
        std::mem::drop(conn);

        Self::open(path, password)
    }

    pub fn open(path: &Path, password: &str) -> Result<Profile, Error> {
        let open_flags: OpenFlags =
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        let conn =
            Connection::open_with_flags(path, open_flags).map_err(Error::DatabaseOpenFailure)?;

        // set password
        conn.pragma_update(None, "key", password)
            .map_err(Error::PragmaUpdateFailure)?;

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

// UserProfile
pub struct UserProfile {
    display_name: String,
}

// Avatar
pub struct Avatar {
    // 256x256 8-bit channel RGBA image in row-major order
    rgba_data: Box<[u8; Self::BYTES]>,
}

impl Avatar {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    const CHANNELS: usize = 4;
    const BYTES: usize = Self::WIDTH * Self::HEIGHT * Self::CHANNELS;
}

pub struct Salt {
    data: [u8; 32],
}

//
// Database Types
//

mod db {
    use crate::v4::error::Error;
    use rusqlite::{params, Connection, OpenFlags, Statement};

    // RowID types
    pub(super) struct DBVersionRowID(pub i64);
    impl rusqlite::ToSql for DBVersionRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct UserProfileRowID(pub i64);
    impl rusqlite::ToSql for UserProfileRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct AvatarRowID(pub i64);
    impl rusqlite::ToSql for AvatarRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct UserRowID(pub i64);
    impl rusqlite::ToSql for UserRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct ConversationRowID(pub i64);
    impl rusqlite::ToSql for ConversationRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct ConversationMemberRowID(pub i64);
    impl rusqlite::ToSql for ConversationMemberRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct MessageRecordRowID(pub i64);
    impl rusqlite::ToSql for MessageRecordRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct MessageContentRowID(pub i64);
    impl rusqlite::ToSql for MessageContentRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct ModifiedMessageRowID(pub i64);
    impl rusqlite::ToSql for ModifiedMessageRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct TextMessageRowID(pub i64);
    impl rusqlite::ToSql for TextMessageRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct FileShareMessageRowID(pub i64);
    impl rusqlite::ToSql for FileShareMessageRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct SaltRowID(pub i64);
    impl rusqlite::ToSql for SaltRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct Sha256HashRowID(pub i64);
    impl rusqlite::ToSql for Sha256HashRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct Ed25519PrivateKeyRowID(pub i64);
    impl rusqlite::ToSql for Ed25519PrivateKeyRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct Ed25519PublicKeyRowID(pub i64);
    impl rusqlite::ToSql for Ed25519PublicKeyRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct Ed25519SignatureRowID(pub i64);
    impl rusqlite::ToSql for Ed25519SignatureRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct X25519PrivateKeyRowID(pub i64);
    impl rusqlite::ToSql for X25519PrivateKeyRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct X25519PublicKeyRowID(pub i64);
    impl rusqlite::ToSql for X25519PublicKeyRowID {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }

    //
    // DBVersion
    //

    pub(super) struct DBVersionRow {
        rowid: DBVersionRowID,
        major: i64,
        minor: i64,
        patch: i64,
    }

    //
    // UserProfile
    //

    pub(super) struct UserProfileRow {
        rowid: UserProfileRowID,
        display_name: String,
        pronouns: Option<String>,
        avatar_rowid: Option<AvatarRowID>,
        status: Option<String>,
        description: Option<String>,
    }

    //
    // Avatar
    //

    pub(super) struct AvatarRow {
        rowid: AvatarRowID,
        // 256x256 8-bit channel RGBA image in row-major order
        value: Box<[u8; crate::v4::profile::Avatar::BYTES]>,
    }

    //
    // User
    //

    pub(super) struct UserRow {
        rowid: UserRowID,
        user_type: UserType,
        user_profile_rowid: Option<UserProfileRowID>,
        identity_ed25519_public_key_rowid: Ed25519PublicKeyRowID,
        identity_ed25519_private_key_rowid: Option<Ed25519PrivateKeyRowID>,
        remote_endpoint_ed25519_public_key_rowid: Option<Ed25519PublicKeyRowID>,
        remote_endpoint_x25519_private_key_rowid: Option<X25519PrivateKeyRowID>,
        local_endpoint_ed25519_private_key_rowid: Option<Ed25519PrivateKeyRowID>,
        local_endpoint_x25519_public_key_rowid: Option<X25519PublicKeyRowID>,
    }

    //
    // UserType
    //

    #[derive(Clone)]
    #[repr(i64)]
    pub(super) enum UserType {
        Owner = 0i64,
        Allowed = 1i64,
        Requesting = 2i64,
        Rejected = 3i64,
        Blocked = 4i64,
    }
    impl rusqlite::ToSql for UserType {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            let val = self.clone() as i64;
            Ok(val.into())
        }
    }

    //
    // Conversation
    //

    pub(super) struct ConversationRow {
        rowid: ConversationRowID,
        conversation_type: ConversationType,
    }

    //
    // ConversationType
    //

    #[derive(Clone)]
    #[repr(i64)]
    pub(super) enum ConversationType {
        LegacyV3 = 0i64,
        PersistentDirectMessage = 1i64,
        EphemeralDirectMessage = 2i64,
    }
    impl rusqlite::ToSql for ConversationType {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            let val = self.clone() as i64;
            Ok(val.into())
        }
    }

    //
    // ConversationMember
    //

    pub(super) struct ConversationMemberRow {
        rowid: ConversationMemberRowID,
        conversation_rowid: ConversationRowID,
        user_rowid: UserRowID,
    }

    //
    // MessageRecord
    //

    pub(super) struct MessageRecordRow {
        rowid: MessageRecordRowID,
        conversation_rowid: ConversationRowID,
        user_rowid: UserRowID,
        message_sequence: MessageSequence,
        record_sequence: RecordSequence,
        initial_timestamp: Timestamp,
        edit_timestamp: Timestamp,
        message_record_salt_rowid: SaltRowID,
        signature_rowid: Ed25519SignatureRowID,
        message_type: MessageType,
        test_message_rowid: Option<TextMessageRowID>,
    }

    pub(super) struct MessageSequence(pub i64);
    impl rusqlite::ToSql for MessageSequence {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct RecordSequence(pub i64);
    impl rusqlite::ToSql for RecordSequence {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    pub(super) struct Timestamp(pub i64);
    impl rusqlite::ToSql for Timestamp {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }
    #[derive(Clone)]
    #[repr(i64)]
    pub(super) enum MessageType {
        Empty = 0i64,
        Text = 1i64,
    }
    impl rusqlite::ToSql for MessageType {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            let val = self.clone() as i64;
            Ok(val.into())
        }
    }

    //
    // ModifiedMessage
    //

    pub(super) struct ModifiedMessageRow {}

    //
    // TextMessage
    //

    pub(super) struct TextMessageRow {
        rowid: TextMessageRowID,
        text: String,
    }

    //
    // FileShareMessage
    //

    pub(super) struct FileSize(pub i64);
    impl rusqlite::ToSql for FileSize {
        fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
            self.0.to_sql()
        }
    }

    pub(super) struct FileShareMessageRow {
        rowid: FileShareMessageRowID,
        file_size: FileSize,
        file_data_hash_rowid: Sha256HashRowID,
        file_path: Option<String>,
    }

    //
    // Salt
    //

    pub(super) struct SaltRow {
        rowid: SaltRowID,
        value: [u8; 32],
    }

    //
    // Ed25519PublicKey
    //

    pub(super) struct Ed25519PublicKeyRow {
        rowid: Ed25519PublicKeyRowID,
        value: String,
    }

    //
    // Ed25519PrivateKey
    //

    pub(super) struct Ed25519PrivateKeyRow {
        rowid: Ed25519PrivateKeyRowID,
        value: [u8; 64],
    }

    //
    // Ed25519Signature
    //

    pub(super) struct Ed25519SignatureRow {
        rowid: Ed25519SignatureRowID,
        value: [u8; 64],
    }

    //
    // X25519PrivateKey
    //

    pub(super) struct X25519PrivateKeyRow {
        rowid: X25519PrivateKeyRowID,
        value: [u8; 32],
    }

    //
    // X25519PublicKey
    //

    pub(super) struct X25519PublicKeyRow {
        rowid: X25519PublicKeyRowID,
        value: [u8; 32],
    }

    //
    // Row insert methods
    //

    pub(super) fn insert_user_profile(
        conn: &Connection,
        display_name: String,
        pronouns: Option<String>,
        avatar_rowid: Option<AvatarRowID>,
        status: Option<String>,
        description: Option<String>,
    ) -> Result<UserProfileRowID, Error> {
        conn.execute(
            "INSERT INTO user_profiles (
                  display_name,
                  pronouns,
                  avatar_rowid,
                  status,
                  description
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![display_name, pronouns, avatar_rowid, status, description],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        Ok(UserProfileRowID(rowid))
    }

    pub(super) fn insert_avatar(
        conn: &Connection,
        avatar_rgba_data: &[u8; crate::v4::profile::Avatar::BYTES],
    ) -> Result<AvatarRowID, Error> {
        conn.execute(
            "INSERT INTO avatars (value) VALUES (?1)",
            params![avatar_rgba_data],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        Ok(AvatarRowID(rowid))
    }

    pub(super) fn insert_user(
        conn: &Connection,
        user_type: UserType,
        user_profile_rowid: UserProfileRowID,
        identity_ed25519_public_key_rowid: Ed25519PublicKeyRowID,
        identity_ed25519_private_key_rowid: Option<Ed25519PrivateKeyRowID>,
        remote_endpoint_ed25519_public_key_rowid: Option<Ed25519PublicKeyRowID>,
        remote_endpoint_x25519_private_key_rowid: Option<X25519PrivateKeyRowID>,
        local_endpoint_ed25519_private_key_rowid: Option<Ed25519PrivateKeyRowID>,
        local_endpoint_x25519_public_key_rowid: Option<X25519PublicKeyRowID>,
    ) -> Result<UserRowID, Error> {
        conn.execute(
            "INSERT INTO users (
                  user_type,
                  user_profile_rowid,
                  identity_ed25519_public_key_rowid,
                  identity_ed25519_private_key_rowid,
                  remote_endpoint_ed25519_public_key_rowid,
                  remote_endpoint_x25519_private_key_rowid,
                  local_endpoint_ed25519_private_key_rowid,
                  local_endpoint_x25519_public_key_rowid
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                user_type,
                user_profile_rowid,
                identity_ed25519_public_key_rowid,
                identity_ed25519_private_key_rowid,
                remote_endpoint_ed25519_public_key_rowid,
                remote_endpoint_x25519_private_key_rowid,
                local_endpoint_ed25519_private_key_rowid,
                local_endpoint_x25519_public_key_rowid
            ],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(UserRowID(rowid))
    }

    pub(super) fn insert_conversation(
        conn: &Connection,
        conversation_type: ConversationType,
    ) -> Result<ConversationRowID, Error> {
        conn.execute(
            "INSERT INTO conversations (conversation_type) VALUES (?1)",
            params![conversation_type],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(ConversationRowID(rowid))
    }

    pub(super) fn insert_conversation_member(
        conn: &Connection,
        conversation_rowid: ConversationRowID,
        user_rowid: UserRowID,
    ) -> Result<ConversationRowID, Error> {
        conn.execute(
            "INSERT INTO conversation_members (
                  conversation_rowid,
                  user_rowid
                ) VALUES (?1, ?2)",
            params![conversation_rowid, user_rowid],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(ConversationRowID(rowid))
    }

    pub(super) fn insert_message_record(
        conn: &Connection,
        conversation_rowid: ConversationRowID,
        user_rowid: UserRowID,
        record_sequence: RecordSequence,
        message_sequence: MessageSequence,
        create_timestamp: Timestamp,
        modify_timestamp: Timestamp,
        message_content_rowid: MessageContentRowID,
        signature_rowid: Ed25519SignatureRowID,
    ) -> Result<MessageRecordRowID, Error> {
        conn.execute(
            "INSERT INTO message_records (
                   conversation_rowid,
                   user_rowid,
                   record_sequence,
                   message_sequence,
                   create_timestamp,
                   modify_timestamp,
                   message_content_rowid,
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                conversation_rowid,
                user_rowid,
                record_sequence,
                message_sequence,
                create_timestamp,
                modify_timestamp,
                message_content_rowid,
            ],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(MessageRecordRowID(rowid))
    }

    pub(super) fn insert_message_content(
        conn: &Connection,
        salt_rowid: SaltRowID,
        message_type: MessageType,
        modified_message_rowid: Option<ModifiedMessageRowID>,
        text_message_rowid: Option<TextMessageRowID>,
        file_share_message_rowid: Option<FileShareMessageRowID>,
    ) -> Result<MessageContentRowID, Error> {
        conn.execute(
            "INSERT INTO message_contents (
                    salt_rowid,
                    message_type,
                    modified_message_rowid,
                    text_message_rowid,
                    file_share_message_rowid,
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                salt_rowid,
                message_type,
                modified_message_rowid,
                text_message_rowid,
                file_share_message_rowid,
            ],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(MessageContentRowID(rowid))
    }

    pub(super) fn insert_modified_message(
        conn: &Connection,
        original_message_content_hash_rowid: Sha256HashRowID,
        original_message_record_signature_rowid: Ed25519SignatureRowID,
    ) -> Result<ModifiedMessageRowID, Error> {
        conn.execute(
            "INSERT INTO modified_messages (
                    original_message_content_hash_rowid,
                    original_message_record_signature_rowid,
                ) VALUES (?1, ?2)",
            params![
                original_message_content_hash_rowid,
                original_message_record_signature_rowid,
            ],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(ModifiedMessageRowID(rowid))
    }

    pub(super) fn insert_text_message(
        conn: &Connection,
        text: String,
    ) -> Result<TextMessageRowID, Error> {
        conn.execute(
            "INSERT INTO text_messages (
                    text,
                ) VALUES (?1)",
            params![text],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(TextMessageRowID(rowid))
    }

    pub(super) fn insert_file_share_message(
        conn: &Connection,
        file_data_salt_rowid: SaltRowID,
        file_size: FileSize,
        file_data_hash_rowid: Sha256HashRowID,
        file_path: Option<String>,
    ) -> Result<FileShareMessageRowID, Error> {
        conn.execute(
            "INSERT INTO file_share_messages (
                    file_data_salt_rowid,
                    file_size,
                    file_data_hash_rowid,
                    file_path,
                ) VALUES (?1, ?2, ?3, ?4)",
            params![
                file_data_salt_rowid,
                file_size,
                file_data_hash_rowid,
                file_path,
            ],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(FileShareMessageRowID(rowid))
    }

    pub(super) fn insert_salt(conn: &Connection, value: [u8; 32]) -> Result<SaltRowID, Error> {
        conn.execute(
            "INSERT INTO salts (
                    value,
                ) VALUES (?1)",
            params![value,],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(SaltRowID(rowid))
    }

    pub(super) fn insert_sha256_hash(
        conn: &Connection,
        value: [u8; 32],
    ) -> Result<Sha256HashRowID, Error> {
        conn.execute(
            "INSERT INTO sha256_hashes (
                    value,
                ) VALUES (?1)",
            params![value,],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(Sha256HashRowID(rowid))
    }

    pub(super) fn insert_ed25519_private_key(
        conn: &Connection,
        value: [u8; 64],
    ) -> Result<Ed25519PrivateKeyRowID, Error> {
        conn.execute(
            "INSERT INTO ed25519_private_keys (
                    value,
                ) VALUES (?1)",
            params![value,],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(Ed25519PrivateKeyRowID(rowid))
    }

    pub(super) fn insert_ed25519_public_key(
        conn: &Connection,
        value: [u8; 32],
    ) -> Result<Ed25519PublicKeyRowID, Error> {
        conn.execute(
            "INSERT INTO ed25519_public_keys (
                    value,
                ) VALUES (?1)",
            params![value,],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(Ed25519PublicKeyRowID(rowid))
    }

    pub(super) fn insert_ed25519_signature(
        conn: &Connection,
        value: [u8; 64],
    ) -> Result<Ed25519SignatureRowID, Error> {
        conn.execute(
            "INSERT INTO ed25519_signatures (
                    value,
                ) VALUES (?1)",
            params![value,],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(Ed25519SignatureRowID(rowid))
    }

    pub(super) fn insert_x25519_private_key(
        conn: &Connection,
        value: [u8; 32],
    ) -> Result<X25519PrivateKeyRowID, Error> {
        conn.execute(
            "INSERT INTO x25519_private_keys (
                    value,
                ) VALUES (?1)",
            params![value,],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(X25519PrivateKeyRowID(rowid))
    }

    pub(super) fn insert_x25519_public_key(
        conn: &Connection,
        value: [u8; 32],
    ) -> Result<X25519PublicKeyRowID, Error> {
        conn.execute(
            "INSERT INTO ed25519_public_keys (
                    value,
                ) VALUES (?1)",
            params![value,],
        )
        .map_err(Error::StatementExecuteFailure)?;
        let rowid = conn.last_insert_rowid();
        assert!(rowid > 0);
        Ok(X25519PublicKeyRowID(rowid))
    }
}

/*
#[cfg(feature = "v3-profile")]
impl TryFrom<v3::profile::Profile> for Profile {
    type Error = String;

    fn try_from(value: v3::profile::Profile) -> Result<Self, Self::Error> {
        let identity_key = value.private_key;

        Ok(Self {
            identity_key
        })
    }
}
*/

//
// Tests
//

mod tests {
    use crate::v4::profile::*;

    #[test]
    fn test_database_round_trips() -> anyhow::Result<()> {
        let mut path = std::env::temp_dir();
        path.push("test_database_round_trips.ricochet-profile");
        if Path::exists(&path) {
            std::fs::remove_file(&path)?;
        }

        println!("path: {path:?}");

        let mut profile = Profile::new(&path, "hunter42")?;
        let avatar_rowid = db::insert_avatar(&profile.conn, &[0u8; 256 * 256 * 4])?;
        let user_profile_rowid = db::insert_user_profile(
            &profile.conn,
            "Alice".to_string(),
            None,
            Some(avatar_rowid),
            None,
            None,
        )?;

        Ok(())
    }
}
