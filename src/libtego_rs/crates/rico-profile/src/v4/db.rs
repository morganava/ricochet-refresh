// std
use std::collections::BTreeSet;

// extern
use rusqlite::{params, Connection, OpenFlags, Statement, Transaction};
use tor_interface::tor_crypto::{
    Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, X25519PrivateKey, X25519PublicKey,
    ED25519_PRIVATE_KEY_SIZE, ED25519_PUBLIC_KEY_SIZE, ED25519_SIGNATURE_SIZE,
    X25519_PRIVATE_KEY_SIZE, X25519_PUBLIC_KEY_SIZE,
};

// internal
use crate::v4::error::Error;
use crate::v4::profile;

/// Implements ToSql, FromSql for a wrapper struct around a single value.
macro_rules! impl_sql_wrapper_type {
    // struct case
    ($vis:vis struct $wrapper_type:ident($inner_vis:vis $inner_type:ty)) => {
        #[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        $vis struct $wrapper_type($inner_vis $inner_type);

        impl rusqlite::ToSql for $wrapper_type {
            fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
                self.0.to_sql()
            }
        }

        impl rusqlite::types::FromSql for $wrapper_type {
            fn column_result(
                value: rusqlite::types::ValueRef<'_>,
            ) -> Result<Self, rusqlite::types::FromSqlError> {
                <$inner_type>::column_result(value).map(|v| $wrapper_type(v))
            }
        }
    };
}

//
// RowID types
//

impl_sql_wrapper_type!(pub(crate) struct DBVersionRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct UserProfileRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct AvatarRowID(pub i64));
impl_sql_wrapper_type!(pub struct UserRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct ConversationRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct ConversationMemberRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct MessageRecordRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct MessageContentRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct ModifiedMessageRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct TextMessageRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct FileShareMessageRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct SaltRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct Sha256HashRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct Ed25519PrivateKeyRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct Ed25519PublicKeyRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct Ed25519SignatureRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct X25519PrivateKeyRowID(pub i64));
impl_sql_wrapper_type!(pub(crate) struct X25519PublicKeyRowID(pub i64));

//
// DBVersion
//

pub struct DBVersionRow {
    rowid: DBVersionRowID,
    major: i64,
    minor: i64,
    patch: i64,
}

//
// UserProfile
//

pub struct UserProfileRow {
    rowid: UserProfileRowID,
    nickname: String,
    pet_name: Option<String>,
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
    value: Box<[u8; profile::Avatar::BYTES]>,
}

//
// User
//

pub(super) struct UserRow {
    rowid: UserRowID,
    user_type: crate::v4::profile::UserType,
    user_profile_rowid: Option<UserProfileRowID>,
    identity_ed25519_public_key_rowid: Ed25519PublicKeyRowID,
    identity_ed25519_private_key_rowid: Option<Ed25519PrivateKeyRowID>,
    remote_endpoint_ed25519_public_key_rowid: Option<Ed25519PublicKeyRowID>,
    remote_endpoint_x25519_private_key_rowid: Option<X25519PrivateKeyRowID>,
    local_endpoint_ed25519_private_key_rowid: Option<Ed25519PrivateKeyRowID>,
    local_endpoint_x25519_public_key_rowid: Option<X25519PublicKeyRowID>,
}

//
// Conversation
//

pub(super) struct ConversationRow {
    rowid: ConversationRowID,
    conversation_type: profile::ConversationType,
    conversation_key_rowid: Sha256HashRowID,
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

impl_sql_wrapper_type!(pub(super) struct MessageSequence(pub i64));
impl_sql_wrapper_type!(pub(super) struct RecordSequence(pub i64));
impl_sql_wrapper_type!(pub(super) struct Timestamp(pub i64));

type MessageType = rico_protocol::v4::MessageType;

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

impl_sql_wrapper_type!(pub(super) struct FileSize(pub i64));

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
// Set the table's password for decryption key
//

pub(super) fn set_password(conn: &Connection, password: &str) -> Result<(), Error> {
    conn.pragma_update(None, "key", password)
        .map_err(Error::PragmaUpdateFailure)?;
    Ok(())
}

//
// Create database tables and indexes
//

pub(super) fn create_tables(conn: &Connection) -> Result<(), Error> {
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
          nickname TEXT NOT NULL,
          pet_name TEXT,
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
          user_profile_rowid INTEGER UNIQUE REFERENCES user_profiles(rowid),
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
          conversation_type INTEGER NOT NULL CHECK(conversation_type >= 1 AND conversation_type <= 2),
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
        CREATE INDEX idx_message_records_conversation_timestamp_record ON message_records(conversation_rowid, create_timestamp DESC, record_sequence DESC);

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
    Ok(())
}

//
// Row insert methods
//

pub(crate) fn insert_db_version(
    conn: &Connection,
    major: i64,
    minor: i64,
    patch: i64,
) -> Result<DBVersionRowID, Error> {
    conn.execute(
        "INSERT INTO db_versions (
            major,
            minor,
            patch
        ) VALUES (?1, ?2, ?3)",
        params![major, minor, patch],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = conn.last_insert_rowid();
    Ok(DBVersionRowID(rowid))
}

pub fn insert_user_profile(
    tx: &Transaction<'_>,
    user_profile: &profile::UserProfile,
) -> Result<UserProfileRowID, Error> {
    let nickname = &user_profile.nickname;
    let pet_name = &user_profile.pet_name;
    let pronouns = &user_profile.pronouns;
    let avatar_rowid = if let Some(avatar) = &user_profile.avatar {
        Some(insert_avatar(tx, avatar)?)
    } else {
        None
    };
    let status = &user_profile.status;
    let description = &user_profile.description;

    tx.execute(
        "INSERT INTO user_profiles (
              nickname,
              pet_name,
              pronouns,
              avatar_rowid,
              status,
              description
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            nickname,
            pet_name,
            pronouns,
            avatar_rowid,
            status,
            description
        ],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    Ok(UserProfileRowID(rowid))
}

pub fn insert_avatar(tx: &Transaction<'_>, avatar: &profile::Avatar) -> Result<AvatarRowID, Error> {
    tx.execute(
        "INSERT INTO avatars (value) VALUES (?1)",
        params![avatar.rgba_data],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    Ok(AvatarRowID(rowid))
}

pub fn insert_user(tx: &Transaction<'_>, user: &profile::User) -> Result<UserRowID, Error> {
    let user_type = user.user_type;
    let user_profile_rowid = insert_user_profile(tx, &user.user_profile)?;
    let identity_ed25519_public_key_rowid =
        insert_ed25519_public_key(tx, &user.identity_ed25519_public_key)?;
    let identity_ed25519_private_key_rowid =
        if let Some(identity_ed25519_private_key) = &user.identity_ed25519_private_key {
            Some(insert_ed25519_private_key(
                tx,
                &identity_ed25519_private_key,
            )?)
        } else {
            None
        };
    let remote_endpoint_ed25519_public_key_rowid = if let Some(remote_endpoint_ed25519_public_key) =
        &user.remote_endpoint_ed25519_public_key
    {
        Some(insert_ed25519_public_key(
            tx,
            &remote_endpoint_ed25519_public_key,
        )?)
    } else {
        None
    };
    let remote_endpoint_x25519_private_key_rowid = if let Some(remote_endpoint_x25519_private_key) =
        &user.remote_endpoint_x25519_private_key
    {
        Some(insert_x25519_private_key(
            tx,
            &remote_endpoint_x25519_private_key,
        )?)
    } else {
        None
    };
    let local_endpoint_ed25519_private_key_rowid = if let Some(local_endpoint_ed25519_private_key) =
        &user.local_endpoint_ed25519_private_key
    {
        Some(insert_ed25519_private_key(
            tx,
            &local_endpoint_ed25519_private_key,
        )?)
    } else {
        None
    };
    let local_endpoint_x25519_public_key_rowid =
        if let Some(local_endpoint_x25519_public_key) = &user.local_endpoint_x25519_public_key {
            Some(insert_x25519_public_key(
                tx,
                &local_endpoint_x25519_public_key,
            )?)
        } else {
            None
        };

    tx.execute(
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
            local_endpoint_x25519_public_key_rowid,
        ],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(UserRowID(rowid))
}

pub fn insert_conversation(
    tx: &Transaction<'_>,
    conversation: profile::Conversation,
) -> Result<ConversationRowID, Error> {
    let conversation_type: i64 = conversation.conversation_type.into();
    let conversation_members = conversation.conversation_members;
    let conversation_key = conversation.conversation_key;
    let conversation_key_rowid = insert_sha256_hash(tx, conversation_key)?;

    tx.execute(
        "INSERT INTO conversations (conversation_type, conversation_key_rowid) VALUES (?1, ?2)",
        params![conversation_type, conversation_key_rowid],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    let conversation_rowid = ConversationRowID(rowid);

    for conversation_member in conversation_members {
        let user_rowid = UserRowID(conversation_member.0);
        let _ = insert_conversation_member(tx, conversation_rowid, user_rowid)?;
    }
    Ok(conversation_rowid)
}

pub fn insert_conversation_member(
    tx: &Connection,
    conversation_rowid: ConversationRowID,
    user_rowid: UserRowID,
) -> Result<ConversationRowID, Error> {
    tx.execute(
        "INSERT INTO conversation_members (
              conversation_rowid,
              user_rowid
            ) VALUES (?1, ?2)",
        params![conversation_rowid, user_rowid],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(ConversationRowID(rowid))
}

/*
pub(crate) fn insert_message_record(
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

pub(crate) fn insert_message_content(
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

pub(crate) fn insert_modified_message(
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

pub(crate) fn insert_text_message(
    conn: &Connection,
    text: String,
) -> Result<TextMessageRowID, Error> {
    conn.execute(
        "INSERT INTO text_messages (
                text
            ) VALUES (?1)",
        params![text],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = conn.last_insert_rowid();
    assert!(rowid > 0);
    Ok(TextMessageRowID(rowid))
}

pub(crate) fn insert_file_share_message(
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
                file_path
            ) VALUES (?1, ?2, ?3, ?4)",
        params![
            file_data_salt_rowid,
            file_size,
            file_data_hash_rowid,
            file_path
        ],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = conn.last_insert_rowid();
    assert!(rowid > 0);
    Ok(FileShareMessageRowID(rowid))
}

pub(crate) fn insert_salt(conn: &Connection, value: [u8; 32]) -> Result<SaltRowID, Error> {
    conn.execute(
        "INSERT INTO salts (
                value
            ) VALUES (?1)",
        params![value],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = conn.last_insert_rowid();
    assert!(rowid > 0);
    Ok(SaltRowID(rowid))
}
*/
pub(crate) fn insert_sha256_hash(
    tx: &Transaction<'_>,
    value: profile::Sha256Sum,
) -> Result<Sha256HashRowID, Error> {
    tx.execute(
        "INSERT INTO sha256_hashes (
                value
            ) VALUES (?1)",
        params![value.0],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(Sha256HashRowID(rowid))
}

pub(crate) fn insert_ed25519_private_key(
    tx: &Transaction<'_>,
    value: &Ed25519PrivateKey,
) -> Result<Ed25519PrivateKeyRowID, Error> {
    let value = value.to_bytes();
    tx.execute(
        "INSERT INTO ed25519_private_keys (
                value
            ) VALUES (?1)",
        params![value],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(Ed25519PrivateKeyRowID(rowid))
}

pub(crate) fn insert_ed25519_public_key(
    tx: &Transaction<'_>,
    value: &Ed25519PublicKey,
) -> Result<Ed25519PublicKeyRowID, Error> {
    let value = value.as_bytes();
    tx.execute(
        "INSERT INTO ed25519_public_keys (
                value
            ) VALUES (?1)",
        params![value],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(Ed25519PublicKeyRowID(rowid))
}

pub(crate) fn insert_ed25519_signature(
    tx: &Transaction<'_>,
    value: &Ed25519Signature,
) -> Result<Ed25519SignatureRowID, Error> {
    let value = value.to_bytes();
    tx.execute(
        "INSERT INTO ed25519_signatures (
                value
            ) VALUES (?1)",
        params![value],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(Ed25519SignatureRowID(rowid))
}

pub(crate) fn insert_x25519_private_key(
    tx: &Transaction<'_>,
    value: &X25519PrivateKey,
) -> Result<X25519PrivateKeyRowID, Error> {
    let value = value.to_bytes();
    tx.execute(
        "INSERT INTO x25519_private_keys (
                value
            ) VALUES (?1)",
        params![value],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(X25519PrivateKeyRowID(rowid))
}

pub(crate) fn insert_x25519_public_key(
    tx: &Transaction<'_>,
    value: &X25519PublicKey,
) -> Result<X25519PublicKeyRowID, Error> {
    let value = value.as_bytes();
    tx.execute(
        "INSERT INTO ed25519_public_keys (
                value,
            ) VALUES (?1)",
        params![value,],
    )
    .map_err(Error::StatementExecuteFailure)?;
    let rowid = tx.last_insert_rowid();
    assert!(rowid > 0);
    Ok(X25519PublicKeyRowID(rowid))
}

//
// Row Select Methods
//

pub fn select_newest_db_version(conn: &Connection) -> Result<profile::Version, Error> {
    let (major, minor, patch) = conn.query_one(
        "SELECT major, minor, patch FROM db_versions ORDER BY rowid DESC LIMIT 1;",
        [],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        },
    )?;

    profile::Version::new(major, minor, patch)
}

pub fn select_all_conversations(
    conn: &Connection,
) -> Result<Vec<(profile::Conversation, ConversationRowID)>, Error> {
    let mut result: Vec<(profile::Conversation, ConversationRowID)> = Default::default();

    // create our prepared statements
    let mut select_conversations_stmt =
        conn.prepare("SELECT rowid, conversation_type, conversation_key_rowid FROM conversations")?;

    let mut select_conversation_members_stmt =
        conn.prepare("SELECT user_rowid FROM conversation_members WHERE conversation_rowid = ?1")?;

    let mut select_conversation_key_stmt =
        conn.prepare("SELECT value FROM sha256_hashes WHERE rowid = ?1")?;

    // get all our conversaiton rows
    let conversation_rows = select_conversations_stmt.query_map(params![], |row| {
        Ok((
            row.get::<_, ConversationRowID>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Sha256HashRowID>(2)?,
        ))
    })?;

    // build conversation objects from rows
    for conversation in conversation_rows {
        let (rowid, conversation_type, conversation_key_rowid) = conversation?;
        let conversation_type = profile::ConversationType::try_from(conversation_type)?;

        // get conversation members
        let mut conversation_members: BTreeSet<UserRowID> = Default::default();
        let conversation_member_rows = select_conversation_members_stmt
            .query_map(params![], |row| row.get::<_, UserRowID>(0))?;
        for conversation_member in conversation_member_rows {
            conversation_members.insert(conversation_member?);
        }

        // get our conversation key
        let conversation_key = select_conversation_key_stmt
            .query_one(params![conversation_key_rowid], |row| {
                row.get::<_, [u8; 32]>(0)
            })?;
        let conversation_key = profile::Sha256Sum(conversation_key);

        // append result
        result.push((
            profile::Conversation {
                conversation_type,
                conversation_members,
                conversation_key,
            },
            rowid,
        ));
    }
    Ok(result)
}

pub fn select_all_users(conn: &Connection) -> Result<Vec<(profile::User, UserRowID)>, Error> {
    // create our prepared statement
    let mut select_users_stmt = conn.prepare("SELECT rowid, user_type, user_profile_rowid, identity_ed25519_public_key_rowid, identity_ed25519_private_key_rowid, remote_endpoint_ed25519_public_key_rowid, remote_endpoint_x25519_private_key_rowid, local_endpoint_ed25519_private_key_rowid, local_endpoint_x25519_public_key_rowid FROM users")?;

    let mut select_user_profile_stmt = conn.prepare("SELECT nickname, pet_name, pronouns, avatar_rowid, status, description FROM user_profiles WHERE rowid = ?1")?;

    let mut select_avatar_stmt = conn.prepare("SELECT value FROM avatars WHERE rowid = ?1")?;

    let mut select_ed25519_private_key_stmt =
        conn.prepare("SELECT value FROM ed25519_private_keys WHERE rowid = ?1")?;

    let mut select_ed25519_public_key_stmt =
        conn.prepare("SELECT value FROM ed25519_public_keys WHERE rowid = ?1")?;

    let mut select_x25519_private_key_stmt =
        conn.prepare("SELECT value FROM x25519_private_keys WHERE rowid = ?1")?;

    let mut select_x25519_public_key_stmt =
        conn.prepare("SELECT value FROM x25519_public_keys WHERE rowid = ?1")?;

    let user_rows = select_users_stmt.query_map(params![], |row| {
        Ok((
            row.get::<_, UserRowID>(0)?,
            row.get::<_, profile::UserType>(1)?,
            row.get::<_, UserProfileRowID>(2)?,
            // identity
            row.get::<_, Ed25519PublicKeyRowID>(3)?,
            row.get::<_, Option<Ed25519PrivateKeyRowID>>(4)?,
            // remote endpoint
            row.get::<_, Option<Ed25519PublicKeyRowID>>(5)?,
            row.get::<_, Option<X25519PrivateKeyRowID>>(6)?,
            // local endpoint
            row.get::<_, Option<Ed25519PrivateKeyRowID>>(7)?,
            row.get::<_, Option<X25519PublicKeyRowID>>(8)?,
        ))
    })?;

    // construct list of users
    let mut result: Vec<(profile::User, UserRowID)> = Default::default();
    for user in user_rows {
        let (
            user_rowid,
            user_type,
            user_profile_rowid,
            identity_ed25519_public_key_rowid,
            identity_ed25519_private_key_rowid,
            remote_endpoint_ed25519_public_key_rowid,
            remote_endpoint_x25519_private_key_rowid,
            local_endpoint_ed25519_private_key_rowid,
            local_endpoint_x25519_public_key_rowid,
        ) = user?;

        // construct user's profile
        let (nickname, pet_name, pronouns, avatar_rowid, status, description) =
            select_user_profile_stmt.query_one(params![user_profile_rowid], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<AvatarRowID>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })?;

        let avatar = match avatar_rowid {
            Some(avatar_rowid) => {
                let rgba_data =
                    Box::new(select_avatar_stmt.query_one(params![avatar_rowid], |row| {
                        row.get::<_, [u8; profile::Avatar::BYTES]>(0)
                    })?);
                let avatar = profile::Avatar { rgba_data };
                Some(avatar)
            }
            None => None,
        };

        let user_profile = profile::UserProfile {
            nickname,
            pet_name,
            pronouns,
            avatar,
            status,
            description,
        };

        // get user's keys

        // identity keys
        let identity_ed25519_public_key = {
            let raw = select_ed25519_public_key_stmt
                .query_one(params![identity_ed25519_public_key_rowid], |row| {
                    row.get::<_, [u8; ED25519_PUBLIC_KEY_SIZE]>(0)
                })?;
            Ed25519PublicKey::from_raw(&raw)?
        };
        let identity_ed25519_private_key = match identity_ed25519_private_key_rowid {
            Some(identity_ed25519_private_key_rowid) => {
                let raw = select_ed25519_private_key_stmt
                    .query_one(params![identity_ed25519_private_key_rowid], |row| {
                        row.get::<_, [u8; ED25519_PRIVATE_KEY_SIZE]>(0)
                    })?;
                Some(Ed25519PrivateKey::from_raw(&raw)?)
            }
            None => None,
        };
        // remote endpoint keys
        let remote_endpoint_ed25519_public_key = match remote_endpoint_ed25519_public_key_rowid {
            Some(remote_endpoint_ed25519_public_key_rowid) => {
                let raw = select_ed25519_public_key_stmt
                    .query_one(params![remote_endpoint_ed25519_public_key_rowid], |row| {
                        row.get::<_, [u8; ED25519_PUBLIC_KEY_SIZE]>(0)
                    })?;
                Some(Ed25519PublicKey::from_raw(&raw)?)
            }
            None => None,
        };
        let remote_endpoint_x25519_private_key = match remote_endpoint_x25519_private_key_rowid {
            Some(remote_endpoint_x25519_private_key_rowid) => {
                let raw = select_x25519_private_key_stmt
                    .query_one(params![remote_endpoint_x25519_private_key_rowid], |row| {
                        row.get::<_, [u8; X25519_PRIVATE_KEY_SIZE]>(0)
                    })?;
                Some(X25519PrivateKey::from_raw(&raw)?)
            }
            None => None,
        };

        // local endpoint keys
        let local_endpoint_ed25519_private_key = match local_endpoint_ed25519_private_key_rowid {
            Some(local_endpoint_ed25519_private_key_rowid) => {
                let raw = select_ed25519_private_key_stmt
                    .query_one(params![local_endpoint_ed25519_private_key_rowid], |row| {
                        row.get::<_, [u8; ED25519_PRIVATE_KEY_SIZE]>(0)
                    })?;
                Some(Ed25519PrivateKey::from_raw(&raw)?)
            }
            None => None,
        };
        let local_endpoint_x25519_public_key = match local_endpoint_x25519_public_key_rowid {
            Some(local_endpoint_x25519_public_key_rowid) => {
                let raw = select_x25519_public_key_stmt
                    .query_one(params![local_endpoint_x25519_public_key_rowid], |row| {
                        row.get::<_, [u8; X25519_PUBLIC_KEY_SIZE]>(0)
                    })?;
                Some(X25519PublicKey::from_raw(&raw))
            }
            None => None,
        };

        let user = profile::User {
            user_type,
            user_profile,
            identity_ed25519_public_key,
            identity_ed25519_private_key,
            remote_endpoint_ed25519_public_key,
            remote_endpoint_x25519_private_key,
            local_endpoint_ed25519_private_key,
            local_endpoint_x25519_public_key,
        };

        result.push((user, user_rowid));
    }

    Ok(result)
}

//
// Row delete methods
//

fn delete_avatar(tx: &Transaction, avatar_rowid: AvatarRowID) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM avatars WHERE rowid = 1?",
        params![avatar_rowid],
    )?;
    Ok(())
}

fn delete_user_profile(
    tx: &Transaction<'_>,
    user_profile_rowid: UserProfileRowID,
) -> Result<(), Error> {
    let avatar_rowid = tx.query_one(
        "DELETE FROM user_profiles WHERE rowid = ?1 RETURNING avatar_rowid",
        params![user_profile_rowid],
        |row| row.get::<_, AvatarRowID>(0),
    )?;
    delete_avatar(tx, avatar_rowid)?;

    Ok(())
}

pub fn delete_user(tx: &Transaction<'_>, user_handle: UserRowID) -> Result<(), Error> {
    let user_rowid = user_handle;

    let (user_profile_rowid, identity_ed25519_public_key_rowid, identity_ed25519_private_key_rowid, remote_endpoint_ed25519_public_key_rowid, remote_endpoint_x25519_private_key_rowid, local_endpoint_ed25519_private_key_rowid, local_endpoint_x25519_public_key_rowid) = tx.query_one("DELETE FROM users WHERE rowid = ?1 RETURNING user_profile_rowid, identity_ed25519_public_key_rowid, identity_ed25519_private_key_rowid, remote_endpoint_ed25519_public_key_rowid, remote_endpoint_x25519_private_key_rowid, local_endpoint_ed25519_private_key_rowid, local_endpoint_x25519_public_key_rowid", params![user_rowid], |row| Ok((
            row.get::<_, UserProfileRowID>(0)?,
            row.get::<_, Ed25519PublicKeyRowID>(1)?,
            row.get::<_, Option<Ed25519PrivateKeyRowID>>(2)?,
            row.get::<_, Option<Ed25519PublicKeyRowID>>(3)?,
            row.get::<_, Option<X25519PrivateKeyRowID>>(4)?,
            row.get::<_, Option<Ed25519PrivateKeyRowID>>(5)?,
            row.get::<_, Option<X25519PublicKeyRowID>>(6)?,
        )))?;

    delete_user_profile(tx, user_profile_rowid)?;
    delete_ed25519_public_key(tx, identity_ed25519_public_key_rowid)?;
    if let Some(identity_ed25519_private_key_rowid) = identity_ed25519_private_key_rowid {
        delete_ed25519_private_key(tx, identity_ed25519_private_key_rowid)?;
    }
    if let Some(remote_endpoint_ed25519_public_key_rowid) = remote_endpoint_ed25519_public_key_rowid
    {
        delete_ed25519_public_key(tx, remote_endpoint_ed25519_public_key_rowid)?;
    }
    if let Some(remote_endpoint_x25519_private_key_rowid) = remote_endpoint_x25519_private_key_rowid
    {
        delete_x25519_private_key(tx, remote_endpoint_x25519_private_key_rowid)?;
    }
    if let Some(local_endpoint_ed25519_private_key_rowid) = local_endpoint_ed25519_private_key_rowid
    {
        delete_ed25519_private_key(tx, local_endpoint_ed25519_private_key_rowid)?;
    }
    if let Some(local_endpoint_x25519_public_key_rowid) = local_endpoint_x25519_public_key_rowid {
        delete_x25519_public_key(tx, local_endpoint_x25519_public_key_rowid)?;
    }

    Ok(())
}

pub fn delete_conversation(
    tx: &Transaction<'_>,
    conversation_handle: ConversationRowID,
) -> Result<(), Error> {
    let conversation_rowid = conversation_handle;

    // get and delete this conversation's conversation_key and delete the conversation
    let conversation_key_rowid = tx.query_one(
        "DELETE FROM conversations WHERE conversations_rowid = ?1 RETURNING conversation_key_rowid",
        params![conversation_rowid],
        |row| row.get::<_, Sha256HashRowID>(0),
    )?;
    delete_sha256_hash(tx, conversation_key_rowid)?;

    // and delete all of the conversation members
    let _count = tx.execute(
        "DELETE FROM conversation_members WHERE conversation_rowid = 1?",
        params![conversation_rowid],
    )?;

    // delete all of the conversation's messages and return foreign keys
    let mut delete_messages_stmt = tx.prepare(
        "DELETE FROM message_records WHERE conversation_rowid = ?1 RETURNING message_content_rowid, signature_rowid")?;
    let message_record_foreign_keys_it =
        delete_messages_stmt.query_map(params![conversation_key_rowid], |row| {
            Ok((
                row.get::<_, MessageContentRowID>(0)?,
                row.get::<_, Ed25519SignatureRowID>(1)?,
            ))
        })?;

    // delete referenced message_contents and ed25519_signature
    for message_record_foreign_keys in message_record_foreign_keys_it {
        let (message_content_rowid, signature_rowid) = message_record_foreign_keys?;
        delete_message_content(tx, message_content_rowid)?;
        delete_ed25519_signature(tx, signature_rowid)?;
    }

    Err(Error::NotImplemented)
}

fn delete_message_content(
    tx: &Transaction<'_>,
    message_content_rowid: MessageContentRowID,
) -> Result<(), Error> {
    let (salt_rowid, modified_message_rowid, text_message_rowid, file_share_message_rowid) = tx.query_one("DELETE FROM message_contents WHERE rowid = 1? RETURNING salt_rowid, modified_message_rowid, text_message_rowid, file_share_message_rowid", params![message_content_rowid], |row| Ok((
            row.get::<_, SaltRowID>(0)?,
            row.get::<_, Option<ModifiedMessageRowID>>(1)?,
            row.get::<_, Option<TextMessageRowID>>(2)?,
            row.get::<_, Option<FileShareMessageRowID>>(3)?,
        )))?;

    delete_salt(tx, salt_rowid)?;
    if let Some(modified_message_rowid) = modified_message_rowid {
        delete_modified_message(tx, modified_message_rowid)?;
    }

    if let Some(text_message_rowid) = text_message_rowid {
        delete_text_message(tx, text_message_rowid)?;
    }

    if let Some(file_share_message_rowid) = file_share_message_rowid {
        delete_file_share_message(tx, file_share_message_rowid)?;
    }

    Ok(())
}

fn delete_modified_message(
    tx: &Transaction<'_>,
    modified_message_rowid: ModifiedMessageRowID,
) -> Result<(), Error> {
    let (original_message_content_hash_rowid, original_message_record_signature_rowid) = tx.query_one("DELETE FROM modified_messages WHERE rowid = 1? RETURNING original_message_content_hash_rowid, original_message_record_signature_rowid", params![modified_message_rowid], |row| Ok((
            row.get::<_, Sha256HashRowID>(0)?,
            row.get::<_, Ed25519SignatureRowID>(1)?,
        )))?;

    delete_sha256_hash(tx, original_message_content_hash_rowid)?;
    delete_ed25519_signature(tx, original_message_record_signature_rowid)?;

    Ok(())
}

fn delete_text_message(
    tx: &Transaction<'_>,
    text_message_rowid: TextMessageRowID,
) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM text_messages WHERE rowid = 1?",
        params![text_message_rowid],
    )?;

    Ok(())
}

fn delete_file_share_message(
    tx: &Transaction<'_>,
    file_share_message_rowid: FileShareMessageRowID,
) -> Result<(), Error> {
    let (file_data_salt_rowid, file_data_hash_rowid) = tx.query_one("DELETE FROM file_share_messages WHERE rowid = 1? RETURNING file_data_salt_rowid, file_data_hash_rowid", params![file_share_message_rowid], |row| Ok((
            row.get::<_, SaltRowID>(0)?,
            row.get::<_, Sha256HashRowID>(1)?,
        )))?;

    delete_salt(tx, file_data_salt_rowid)?;
    delete_sha256_hash(tx, file_data_hash_rowid)?;

    Ok(())
}

fn delete_salt(tx: &Transaction<'_>, salt_rowid: SaltRowID) -> Result<(), Error> {
    let _count = tx.execute("DELETE FROM salts WHERE rowid = 1?", params![salt_rowid])?;
    Ok(())
}

fn delete_sha256_hash(
    tx: &Transaction<'_>,
    sha256_hash_rowid: Sha256HashRowID,
) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM sha256_hashes WHERE rowid = 1?",
        params![sha256_hash_rowid],
    )?;
    Ok(())
}

fn delete_ed25519_private_key(
    tx: &Transaction<'_>,
    ed25519_private_key_rowid: Ed25519PrivateKeyRowID,
) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM ed25519_private_keys WHERE rowid = 1?",
        params![ed25519_private_key_rowid],
    )?;
    Ok(())
}

fn delete_ed25519_public_key(
    tx: &Transaction<'_>,
    ed25519_public_key_rowid: Ed25519PublicKeyRowID,
) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM ed25519_public_keys WHERE rowid = 1?",
        params![ed25519_public_key_rowid],
    )?;
    Ok(())
}

fn delete_ed25519_signature(
    tx: &Transaction<'_>,
    ed25519_signature_rowid: Ed25519SignatureRowID,
) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM ed25519_signatures WHERE rowid = 1?",
        params![ed25519_signature_rowid],
    )?;
    Ok(())
}

fn delete_x25519_private_key(
    tx: &Transaction<'_>,
    x25519_private_key_rowid: X25519PrivateKeyRowID,
) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM x25519_private_keys WHERE rowid = 1?",
        params![x25519_private_key_rowid],
    )?;
    Ok(())
}

fn delete_x25519_public_key(
    tx: &Transaction<'_>,
    x25519_public_key_rowid: X25519PublicKeyRowID,
) -> Result<(), Error> {
    let _count = tx.execute(
        "DELETE FROM x25519_public_keys WHERE rowid = 1?",
        params![x25519_public_key_rowid],
    )?;
    Ok(())
}

//
// Tests
//

mod tests {
    use crate::v4::db;
    use crate::v4::profile::*;

    #[test]
    fn test_database_round_trips() -> anyhow::Result<()> {
        let mut path = std::env::temp_dir();
        path.push("test_database_round_trips.ricochet-profile");
        if std::path::Path::exists(&path) {
            std::fs::remove_file(&path)?;
        }

        println!("path: {path:?}");

        let mut profile = Profile::new(&path, "hunter42")?;
        let tx = profile.conn.transaction()?;

        let avatar = Avatar {
            rgba_data: Box::new([0u8; 256 * 256 * 4]),
        };

        let avatar_rowid = db::insert_avatar(&tx, &avatar)?;

        tx.commit()?;

        Ok(())
    }
}
