//! # GXT (Game Exchange Token)
//!
//! Minimal, encrypted, signed and copy-pasteable tokens for manual data exchange between games.
//!
//! For details check out [`spec.md`](https://github.com/hardliner66/gxt/blob/main/spec.md).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::similar_names)]

use std::{fmt, str::FromStr};

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::{Serialize, de::DeserializeOwned};
use serde_cbor::Value;
use thiserror::Error;
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret as XSecret};

pub use serde_json::{from_value, json, to_value};

const PREFIX: &str = "gxt:";
const SIGNATURE_DOMAIN: &[u8] = b"GXT";
const MAX_RAW: usize = 64 * 1024;
const VERSION: u8 = 2;

type Bytes32 = [u8; 32];
type Bytes64 = [u8; 64];

#[derive(Error, Debug)]
/// Errors that can occur while encoding, decoding, compressing,
/// or verifying GXT tokens.
pub enum GxtError {
    #[error("bad prefix")]
    /// The message must start with the prefix "gxt:"
    BadPrefix,
    #[error("decode error: {0}")]
    /// Base58 decoding failed
    Decode(#[from] bs58::decode::Error),
    /// Compression or decompression failed
    #[error("decompress error: {0}")]
    Compression(#[from] std::io::Error),
    /// Encryption or decryption failed
    #[error("encrypt error: {0}")]
    Encryption(String),
    /// CBOR serialization failed
    #[error("cbor error: {0}")]
    Cbor(#[from] serde_cbor::Error),
    /// JSON serialization failed
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// The message is too large
    #[error("too large")]
    TooLarge,
    /// The signature is wrong
    #[error("invalid signature")]
    BadSig,
    /// The id is wrong
    #[error("invalid id")]
    BadId,
    /// A hex value contains invalid characters
    #[error("bad hex")]
    BadHex(#[from] hex::FromHexError),
    /// A hex has the wrong size
    #[error("invalid hex size. expected {expected} got {got}")]
    InvalidHexSize {
        /// The expected hex size
        expected: usize,
        /// The hex size we got
        got: usize,
    },
    /// The specified key can not decrypt this message
    #[error("access denied")]
    AccessDenied,
    /// The structure message is invalid
    #[error("invalid record")]
    Invalid,
    /// Received an unknown payload kind
    #[error("unknown payload kind")]
    UnknownPayloadKind,
}

/// What kind of payload was sent
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum PayloadKind {
    /// ID card
    Id,
    /// Message
    Msg,
}

impl FromStr for PayloadKind {
    type Err = GxtError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "id" => Ok(PayloadKind::Id),
            "msg" => Ok(PayloadKind::Msg),
            _ => Err(GxtError::UnknownPayloadKind),
        }
    }
}

impl fmt::Display for PayloadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id => write!(f, "id"),
            Self::Msg => write!(f, "msg"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Parsed, verified GXT record.
///
/// Represents a decoded token after signature verification and/or decryption.
#[serde(bound(serialize = "P: Serialize", deserialize = "P: Deserialize<'de>"))]
pub struct Envelope<P> {
    /// Version
    pub version: u8,
    /// Verification Key
    pub verification_key: String,
    /// Public Key
    pub encryption_key: String,
    /// Payload Kind
    pub kind: PayloadKind,
    /// Opaque Payload
    pub payload: P,
    /// Id of the Parent Message
    pub parent: Option<String>,
    /// Id of this Message
    pub id: String,
    /// Signature of this Message
    pub signature: String,
}

impl<P: Serialize + DeserializeOwned> fmt::Display for Envelope<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "valid           : true")?;
        writeln!(f, "version         : {}", self.version)?;
        writeln!(
            f,
            "parent          : {}",
            self.parent.as_ref().map_or_else(
                || "-".to_string(),
                |parent| format!("{} ({})", parent, &parent[..8])
            )
        )?;
        writeln!(f, "id              : {} ({})", self.id, &self.id[..8])?;
        writeln!(
            f,
            "verification key: {} ({})",
            self.verification_key,
            &self.verification_key[..8]
        )?;
        writeln!(
            f,
            "encryption key  : {} ({})",
            self.encryption_key,
            &self.encryption_key[..8]
        )?;
        writeln!(f, "kind            : {}", self.kind)?;
        writeln!(f, "payload:")?;
        writeln!(
            f,
            "{}",
            serde_json::to_string_pretty(&self.payload).map_err(|_| fmt::Error)?
        )?;
        Ok(())
    }
}

/// Creates a private key for a peer.
pub fn make_key() -> String {
    let key = SigningKey::generate(&mut OsRng);
    hex::encode(key.to_bytes())
}

/// Creates an ID card containing the necessary data for
/// the encrypted communication and some opaque meta data.
///
/// # Errors
/// - returns a corresponding [`GxtError`], depending on what went wrong.
pub fn make_id_card<M: Serialize + DeserializeOwned>(
    key: &str,
    meta: M,
) -> Result<String, GxtError> {
    let key = parse_key(key.trim())?;
    make(
        &key,
        PayloadKind::Id,
        serde_cbor::value::to_value(meta)?,
        None,
    )
}

/// Verify the signature of a message and return a parsed [`Envelope`].
///
/// # Errors
/// - returns a corresponding [`GxtError`], depending on what went wrong.
pub fn verify_message<P: Serialize + DeserializeOwned>(msg: &str) -> Result<Envelope<P>, GxtError> {
    let raw = decode_message(msg.trim())?;
    let envelope_cbor: Value = serde_cbor::from_slice(&raw)?;

    let arr = match envelope_cbor {
        Value::Array(a) if a.len() == 8 => a,
        _ => return Err(GxtError::Invalid),
    };

    let mut values = arr.into_iter();

    let version = match values.next() {
        Some(Value::Integer(i)) if i == VERSION.into() => 1u8,
        _ => return Err(GxtError::Invalid),
    };
    let verification_key_bytes = match values.next() {
        Some(Value::Text(t)) => parse_hex::<32>(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let encryption_key = match values.next() {
        Some(Value::Text(t)) => parse_hex::<32>(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let kind = match values.next() {
        Some(Value::Text(t)) => PayloadKind::from_str(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let payload = match values.next() {
        Some(payload) => payload.clone(),
        _ => return Err(GxtError::Invalid),
    };
    let parent = match values.next() {
        Some(Value::Text(t)) if !t.is_empty() => Some(parse_hex::<32>(&t)?),
        Some(Value::Text(_)) => None,
        _ => return Err(GxtError::Invalid),
    };
    let id = match values.next() {
        Some(Value::Text(t)) => parse_hex::<32>(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let signature_bytes = match values.next() {
        Some(Value::Text(t)) => parse_hex::<64>(&t)?,
        _ => return Err(GxtError::Invalid),
    };

    let canonical = get_canonical_representation(
        &verification_key_bytes,
        &encryption_key,
        kind,
        payload.clone(),
    )?;
    let expect = blake3::hash(&canonical);
    if id != *expect.as_bytes() {
        return Err(GxtError::BadId);
    }

    let verification_key =
        VerifyingKey::from_bytes(&verification_key_bytes).map_err(|_| GxtError::Invalid)?;
    let signature = Signature::from_bytes(&signature_bytes);
    verification_key
        .verify_strict(&preimage(&canonical), &signature)
        .map_err(|_| GxtError::BadSig)?;

    Ok(Envelope {
        version,
        verification_key: hex::encode(verification_key_bytes),
        encryption_key: hex::encode(encryption_key),
        parent: parent.map(hex::encode),
        kind,
        payload: serde_cbor::value::from_value(payload)?,
        id: hex::encode(id),
        signature: hex::encode(signature_bytes),
    })
}

/// Create an **encrypted** message for the owner of the
/// ID card that was passed in.
///
/// # Errors
/// - returns a corresponding [`GxtError`], depending on what went wrong.
pub fn encrypt_message<P: Serialize + DeserializeOwned>(
    key: &str,
    id_card: &str,
    payload: &P,
    parent: Option<String>,
) -> Result<String, GxtError> {
    let id_card = verify_message::<Value>(id_card.trim())?;
    let their_encryption_key = parse_hex::<32>(&id_card.encryption_key)?;
    let key = parse_key(key.trim())?;
    let (my_secret_key, _my_encryption_key) = derive_enc_from_signing(&key);
    let encryption_key = enc_derive_key_from_pairs(&my_secret_key, &their_encryption_key);
    let cipher = XChaCha20Poly1305::new(&encryption_key);
    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let plaintext = serde_cbor::to_vec(&payload)?;
    let cipher_text = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| GxtError::Encryption(e.to_string()))?;

    let mut message = std::collections::BTreeMap::new();
    message.insert(
        Value::Text("to".into()),
        Value::Text(hex::encode(their_encryption_key)),
    );
    let mut encrypted_message = std::collections::BTreeMap::new();
    encrypted_message.insert(
        Value::Text("alg".into()),
        Value::Text("xchacha20poly1305".into()),
    );
    encrypted_message.insert(
        Value::Text("n24".into()),
        Value::Text(hex::encode(nonce_bytes)),
    );
    encrypted_message.insert(
        Value::Text("ct".into()),
        Value::Text(hex::encode(&cipher_text)),
    );
    message.insert(Value::Text("enc".into()), Value::Map(encrypted_message));
    let payload = Value::Map(message);
    make(
        &key,
        PayloadKind::Msg,
        payload,
        parent.map(|parent| parse_hex::<32>(&parent)).transpose()?,
    )
}

/// Verify the signature of a message, decrypt its payload and return a parsed [`Envelope`].
///
/// # Errors
/// - returns a corresponding [`GxtError`], depending on what went wrong.
pub fn decrypt_message<P: Serialize + DeserializeOwned>(
    message: &str,
    key: &str,
) -> Result<Envelope<P>, GxtError> {
    let mut envelope = verify_message::<Value>(message.trim())?;

    let key = SigningKey::from_bytes(&parse_hex::<32>(key.trim())?);
    let Value::Map(map) = &envelope.payload else {
        return Err(GxtError::Invalid);
    };
    let to = match map.get(&Value::Text("to".into())) {
        Some(Value::Text(t)) => parse_hex::<32>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let Some(Value::Map(encm)) = map.get(&Value::Text("enc".into())) else {
        return Err(GxtError::Invalid);
    };
    let nonce = match encm.get(&Value::Text("n24".into())) {
        Some(Value::Text(t)) => parse_hex::<24>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let cipher_text = match encm.get(&Value::Text("ct".into())) {
        Some(Value::Text(t)) => hex::decode(t)?,
        _ => return Err(GxtError::Invalid),
    };

    let (my_secret_key, my_encryption_key) = derive_enc_from_signing(&key);
    if to != my_encryption_key {
        return Err(GxtError::AccessDenied);
    }

    let key = enc_derive_key_from_pairs(&my_secret_key, &parse_hex(&envelope.encryption_key)?);
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XNonce::from_slice(&nonce);
    let plaintext = cipher
        .decrypt(nonce, cipher_text.as_ref())
        .map_err(|e| GxtError::Encryption(e.to_string()))?;
    envelope.payload = serde_cbor::from_slice(&plaintext)?;

    Ok(Envelope {
        version: envelope.version,
        verification_key: envelope.verification_key,
        encryption_key: envelope.encryption_key,
        kind: envelope.kind,
        payload: serde_cbor::value::from_value(envelope.payload)?,
        parent: envelope.parent,
        id: envelope.id,
        signature: envelope.signature,
    })
}

#[allow(clippy::too_many_arguments)]
fn cbor_array(
    verification_key: &Bytes32,
    encryption_key: &Bytes32,
    kind: PayloadKind,
    payload: Value,
    parent: Option<Bytes32>,
    id: Option<&Bytes32>,
    signature: Option<&Bytes64>,
) -> Result<Vec<u8>, GxtError> {
    let envelope_values = Value::Array(vec![
        Value::Integer(VERSION.into()),
        Value::Text(hex::encode(verification_key)),
        Value::Text(hex::encode(encryption_key)),
        Value::Text(kind.to_string()),
        payload,
        Value::Text(parent.map(hex::encode).unwrap_or_default()),
        Value::Text(id.map(hex::encode).unwrap_or_default()),
        Value::Text(signature.map(hex::encode).unwrap_or_default()),
    ]);
    Ok(serde_cbor::to_vec(&envelope_values)?)
}

fn get_canonical_representation(
    verification_key: &Bytes32,
    encryption_key: &Bytes32,
    kind: PayloadKind,
    payload: Value,
) -> Result<Vec<u8>, GxtError> {
    cbor_array(
        verification_key,
        encryption_key,
        kind,
        payload,
        None,
        None,
        None,
    )
}

fn preimage(canonical: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(SIGNATURE_DOMAIN.len() + canonical.len());
    v.extend_from_slice(SIGNATURE_DOMAIN);
    v.extend_from_slice(canonical);
    v
}

fn make(
    key: &SigningKey,
    kind: PayloadKind,
    payload: Value,
    parent: Option<Bytes32>,
) -> Result<String, GxtError> {
    let verification_key = key.verifying_key().to_bytes();
    let (_, encryption_key) = derive_enc_from_signing(key);
    let canonical =
        get_canonical_representation(&verification_key, &encryption_key, kind, payload.clone())?;
    if canonical.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }

    let id = blake3::hash(&canonical);
    let signature = key.sign(&preimage(&canonical));

    encode_message(
        &verification_key,
        &encryption_key,
        kind,
        payload,
        parent,
        id.as_bytes(),
        &signature.to_bytes(),
    )
}

#[allow(clippy::too_many_arguments)]
fn encode_message(
    verification_key: &Bytes32,
    encryption_key: &Bytes32,
    kind: PayloadKind,
    payload: Value,
    parent: Option<Bytes32>,
    id: &Bytes32,
    signature: &Bytes64,
) -> Result<String, GxtError> {
    let envelope_cbor = cbor_array(
        verification_key,
        encryption_key,
        kind,
        payload,
        parent,
        Some(id),
        Some(signature),
    )?;
    if envelope_cbor.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }
    let compressed_message = zstd::encode_all(&envelope_cbor[..], 3)?;
    Ok(format!(
        "{}{}",
        PREFIX,
        bs58::encode(compressed_message).into_string()
    ))
}

fn decode_message(message: &str) -> Result<Vec<u8>, GxtError> {
    let rest = message.strip_prefix(PREFIX).ok_or(GxtError::BadPrefix)?;
    let compressed_message = bs58::decode(rest).into_vec()?;
    let raw = zstd::encode_all(&compressed_message[..], 3)?;
    if raw.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }
    Ok(raw)
}

fn parse_hex<const SIZE: usize>(hex_string: &str) -> Result<[u8; SIZE], GxtError> {
    let unsized_hex = hex::decode(hex_string)?;

    let got = unsized_hex.len();
    let hex: [u8; SIZE] = unsized_hex
        .try_into()
        .map_err(|_| GxtError::InvalidHexSize {
            expected: SIZE,
            got,
        })?;
    Ok(hex)
}

fn parse_key(hex_string: &str) -> Result<SigningKey, GxtError> {
    Ok(SigningKey::from_bytes(&parse_hex::<32>(hex_string)?))
}

fn derive_enc_from_signing(key: &SigningKey) -> (Bytes32, Bytes32) {
    let seed = key.to_bytes();
    let derived_key = blake3::derive_key("GXT-ENC-X25519-FROM-ED25519", &seed);
    let secret_key = XSecret::from(derived_key);
    let encryption_key = XPublicKey::from(&secret_key);
    (secret_key.to_bytes(), encryption_key.to_bytes())
}

fn enc_derive_key_from_pairs(my_secret_key: &Bytes32, their_encryption_key: &Bytes32) -> Key {
    let key = XSecret::from(*my_secret_key);
    let verification_key = XPublicKey::from(*their_encryption_key);
    let shared = key.diffie_hellman(&verification_key);
    let derived_key = blake3::derive_key("GXT-ENC-XCHACHA20POLY1305", shared.as_bytes());
    Key::from_slice(&derived_key).to_owned()
}
