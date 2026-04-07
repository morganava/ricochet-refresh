// std
use std::collections::BTreeSet;
use std::io::{Read, Write};

// extern
use sha3::{Digest, Sha3_256};
use tor_interface::tor_crypto::{Ed25519PublicKey, Ed25519Signature, ED25519_SIGNATURE_SIZE};

// internal
use crate::v4::{
    self, ConversationType, FileSize, MessageContentData, MessageSequence, RecordSequence, Salt,
    Sha256Sum, Timestamp, TombstoneData,
};

// sha2
pub fn conversation_key(
    conversation_type: ConversationType,
    conversation_member_public_keys: &BTreeSet<Ed25519PublicKey>,
) -> Sha256Sum {
    let mut hasher = Sha3_256::new();
    // domain separator
    hasher.update(b"ricochet-refresh-conversation");

    // conversaiton type
    let conversation_type: i64 = conversation_type.into();
    hasher.update(conversation_type.to_be_bytes());

    //  number of members
    let conversation_member_count: i64 = conversation_member_public_keys.len() as i64;
    hasher.update(conversation_member_count.to_be_bytes());

    // keys in order
    for public_key in conversation_member_public_keys {
        hasher.update(public_key.as_bytes());
    }

    Sha256Sum(hasher.finalize().into())
}

pub fn message_record_payload(
    previous_signature: Option<&Ed25519Signature>,
    conversation_key: &Sha256Sum,
    user_identity_ed25519_public_key: &Ed25519PublicKey,
    record_sequence: RecordSequence,
    message_sequence: MessageSequence,
    create_timestamp: Timestamp,
    modify_timestamp: Timestamp,
    message_content_hash: &Sha256Sum,
) -> Result<Vec<u8>, v4::Error> {
    let mut payload: Vec<u8> = Default::default();

    if let Some(previous_signature) = previous_signature {
        payload.write(&previous_signature.to_bytes())?;
    } else {
        payload.write(&[0u8; ED25519_SIGNATURE_SIZE])?;
    }

    payload.write(b"ricochet-refresh-message-record")?;
    payload.write(&conversation_key.0)?;
    payload.write(user_identity_ed25519_public_key.as_bytes())?;
    payload.write(&record_sequence.0.to_be_bytes())?;
    payload.write(&message_sequence.0.to_be_bytes())?;
    payload.write(&create_timestamp.millis_since_unix_epoch.to_be_bytes())?;
    payload.write(&modify_timestamp.millis_since_unix_epoch.to_be_bytes())?;
    payload.write(&message_content_hash.0)?;

    Ok(payload)
}

pub fn message_content_hash(
    message_content_salt: &Salt,
    message_content_data: &MessageContentData,
) -> Sha256Sum {
    let mut hasher = Sha3_256::new();
    match message_content_data {
        MessageContentData::Tombstone(TombstoneData {
            original_message_content_hash,
            original_message_record_signature,
        }) => {
            hasher.update(b"ricochet-refresh-tombstone-message");
            hasher.update(&message_content_salt.0);
            hasher.update(&original_message_content_hash.0);
            hasher.update(&original_message_record_signature.to_bytes());
        }
        MessageContentData::Text { text } => {
            hasher.update(b"ricochet-refresh-text-message");
            hasher.update(&message_content_salt.0);
            hasher.update(text.as_bytes());
        }
        MessageContentData::FileShare {
            file_data_salt: _,
            file_size: _,
            file_data_hash,
            file_path: _,
        } => {
            hasher.update(b"ricochet-refresh-file-share-message");
            hasher.update(&message_content_salt.0);
            hasher.update(&file_data_hash.0);
        }
    }
    Sha256Sum(hasher.finalize().into())
}

pub fn file_data_hash(
    file_data_salt: &Salt,
    file_size: FileSize,
    file_contents: &mut impl Read,
) -> Result<Sha256Sum, v4::Error> {
    let mut hasher = Sha3_256::new();

    hasher.update("ricochet-refresh-file-data".as_bytes());
    hasher.update(file_data_salt.0);
    hasher.update(&file_size.0.to_be_bytes());

    // hash file contents
    let mut buffer = [0u8; 8192];
    loop {
        let n = file_contents.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(Sha256Sum(hasher.finalize().into()))
}
