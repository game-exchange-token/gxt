//! # GXT (Game Exchange Token)
//!
//! Minimal, encrypted, signed and copy-pasteable tokens for manual data exchange between games.
//!
//! For details check out [`spec.md`](https://github.com/hardliner66/gxt/blob/main/spec.md).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::similar_names)]

use std::{
    fmt,
    io::{Read, Write},
    str::FromStr,
};

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Serialize;
use serde_cbor::Value;
use thiserror::Error;
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret as XSecret};

const PREFIX: &str = "gxt:";
const SIG_DOMAIN: &[u8] = b"GXT";
const MAX_RAW: usize = 64 * 1024;

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
#[derive(Serialize, Copy, Clone, Debug)]
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

#[derive(Serialize, Clone, Debug)]
/// Parsed, verified GXT record.
///
/// Represents a decoded token after signature verification and/or decryption.
pub struct Envelope {
    /// Version
    pub v: u8,
    /// Verification Key
    pub vk: String,
    /// Public Key
    pub pk: String,
    /// Payload Kind
    pub kind: PayloadKind,
    /// Opaque Payload
    pub payload: Value,
    /// Id of the Parent Message
    pub parent: Option<String>,
    /// Id of this Message
    pub id: String,
    /// Signature of this Message
    pub sig: String,
}

impl fmt::Display for Envelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "valid   : true")?;
        writeln!(f, "version : {}", self.v)?;
        writeln!(
            f,
            "parent  : {}",
            self.parent.as_ref().map_or_else(
                || "-".to_string(),
                |parent| format!("{} ({})", parent, &parent[..8])
            )
        )?;
        writeln!(f, "id      : {} ({})", self.id, &self.id[..8])?;
        writeln!(f, "vk      : {} ({})", self.vk, &self.vk[..8])?;
        writeln!(f, "pk      : {} ({})", self.pk, &self.pk[..8])?;
        writeln!(f, "kind    : {}", self.kind)?;
        writeln!(f, "payload :")?;
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
pub fn make_id_card(key: &str, meta: &str) -> Result<String, GxtError> {
    let key = parse_key(key.trim())?;
    let meta: Value = serde_json::from_str(meta.trim())?;
    make(&key, PayloadKind::Id, meta, None)
}

/// Verify the signature of a message and return a parsed [`Envelope`].
///
/// # Errors
/// - returns a corresponding [`GxtError`], depending on what went wrong.
pub fn verify_message(msg: &str) -> Result<Envelope, GxtError> {
    let raw = decode_message(msg.trim())?;
    let val: Value = serde_cbor::from_slice(&raw)?;

    let a = match val {
        Value::Array(a) if a.len() == 8 => a,
        _ => return Err(GxtError::Invalid),
    };

    let mut a = a.into_iter();

    let v = match a.next() {
        Some(Value::Integer(i)) if i == 1.into() => 1u8,
        _ => return Err(GxtError::Invalid),
    };
    let vk_bytes = match a.next() {
        Some(Value::Text(t)) => parse_hex::<32>(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let pk = match a.next() {
        Some(Value::Text(t)) => parse_hex::<32>(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let kind = match a.next() {
        Some(Value::Text(t)) => PayloadKind::from_str(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let payload = match a.next() {
        Some(payload) => payload.clone(),
        _ => return Err(GxtError::Invalid),
    };
    let parent = match a.next() {
        Some(Value::Text(t)) if !t.is_empty() => Some(parse_hex::<32>(&t)?),
        Some(Value::Text(_)) => None,
        _ => return Err(GxtError::Invalid),
    };
    let id = match a.next() {
        Some(Value::Text(t)) => parse_hex::<32>(&t)?,
        _ => return Err(GxtError::Invalid),
    };
    let sig = match a.next() {
        Some(Value::Text(t)) => parse_hex::<64>(&t)?,
        _ => return Err(GxtError::Invalid),
    };

    let b0 = bytes0(&vk_bytes, &pk, kind, payload.clone())?;
    let expect = blake3::hash(&b0);
    if id != *expect.as_bytes() {
        return Err(GxtError::BadId);
    }

    let vk = VerifyingKey::from_bytes(&vk_bytes).map_err(|_| GxtError::Invalid)?;
    let sigv = Signature::from_bytes(&sig);
    vk.verify_strict(&preimage(&b0), &sigv)
        .map_err(|_| GxtError::BadSig)?;

    Ok(Envelope {
        v,
        vk: hex::encode(vk_bytes),
        pk: hex::encode(pk),
        parent: parent.map(hex::encode),
        kind,
        payload,
        id: hex::encode(id),
        sig: hex::encode(sig),
    })
}

/// Create an **encrypted** message for the owner of the
/// ID card that was passed in.
///
/// # Errors
/// - returns a corresponding [`GxtError`], depending on what went wrong.
pub fn encrypt_message(
    key: &str,
    id_card: &str,
    body: &str,
    parent: Option<String>,
) -> Result<String, GxtError> {
    let id_card = verify_message(id_card.trim())?;
    let pk = parse_hex::<32>(&id_card.pk)?;
    let key = parse_key(key.trim())?;
    let body: Value = serde_json::from_str(body.trim())?;
    let (my_esk, my_epk) = derive_enc_from_signing(&key);
    let ekey = enc_derive_key_from_pairs(&my_esk, &pk);
    let cipher = XChaCha20Poly1305::new(&ekey);
    let mut n = [0u8; 24];
    OsRng.fill_bytes(&mut n);
    let nonce = XNonce::from_slice(&n);
    let pt = serde_cbor::to_vec(&body)?;
    let ct = cipher
        .encrypt(nonce, pt.as_ref())
        .map_err(|e| GxtError::Encryption(e.to_string()))?;

    let mut m = std::collections::BTreeMap::new();
    m.insert(Value::Text("to".into()), Value::Text(hex::encode(pk)));
    m.insert(Value::Text("from".into()), Value::Text(hex::encode(my_epk)));
    let mut encm = std::collections::BTreeMap::new();
    encm.insert(
        Value::Text("alg".into()),
        Value::Text("xchacha20poly1305".into()),
    );
    encm.insert(Value::Text("n24".into()), Value::Text(hex::encode(n)));
    encm.insert(Value::Text("ct".into()), Value::Text(hex::encode(&ct)));
    m.insert(Value::Text("enc".into()), Value::Map(encm));
    let payload = Value::Map(m);
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
pub fn decrypt_message(msg: &str, key: &str) -> Result<Envelope, GxtError> {
    let mut rec = verify_message(msg.trim())?;

    let key = SigningKey::from_bytes(&parse_hex::<32>(key.trim())?);
    let Value::Map(map) = &rec.payload else {
        return Err(GxtError::Invalid);
    };
    let to = match map.get(&Value::Text("to".into())) {
        Some(Value::Text(t)) => parse_hex::<32>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let from = match map.get(&Value::Text("from".into())) {
        Some(Value::Text(t)) => parse_hex::<32>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let Some(Value::Map(encm)) = map.get(&Value::Text("enc".into())) else {
        return Err(GxtError::Invalid);
    };
    let n = match encm.get(&Value::Text("n24".into())) {
        Some(Value::Text(t)) => parse_hex::<24>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let ct = match encm.get(&Value::Text("ct".into())) {
        Some(Value::Text(t)) => hex::decode(t)?,
        _ => return Err(GxtError::Invalid),
    };

    let (_my_esk, my_epk) = derive_enc_from_signing(&key);
    if to != my_epk {
        return Err(GxtError::AccessDenied);
    }

    let (my_esk, _) = derive_enc_from_signing(&key);
    let key = enc_derive_key_from_pairs(&my_esk, &from);
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XNonce::from_slice(&n);
    let pt = cipher
        .decrypt(nonce, ct.as_ref())
        .map_err(|e| GxtError::Encryption(e.to_string()))?;
    rec.payload = serde_cbor::from_slice(&pt)?;

    Ok(rec)
}

#[allow(clippy::too_many_arguments)]
fn cbor_array(
    v: u8,
    vk: &Bytes32,
    pk: &Bytes32,
    kind: PayloadKind,
    payload: Value,
    parent: Option<Bytes32>,
    id: Option<&Bytes32>,
    sig: Option<&Bytes64>,
) -> Result<Vec<u8>, GxtError> {
    let arr = Value::Array(vec![
        Value::Integer(v.into()),
        Value::Text(hex::encode(vk)),
        Value::Text(hex::encode(pk)),
        Value::Text(kind.to_string()),
        payload,
        Value::Text(parent.map(hex::encode).unwrap_or_default()),
        Value::Text(id.map(hex::encode).unwrap_or_default()),
        Value::Text(sig.map(hex::encode).unwrap_or_default()),
    ]);
    Ok(serde_cbor::to_vec(&arr)?)
}

fn bytes0(
    vk: &Bytes32,
    pk: &Bytes32,
    kind: PayloadKind,
    payload: Value,
) -> Result<Vec<u8>, GxtError> {
    cbor_array(1, vk, pk, kind, payload, None, None, None)
}

fn preimage(bytes0: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(SIG_DOMAIN.len() + bytes0.len());
    v.extend_from_slice(SIG_DOMAIN);
    v.extend_from_slice(bytes0);
    v
}

fn make(
    key: &SigningKey,
    kind: PayloadKind,
    payload: Value,
    parent: Option<Bytes32>,
) -> Result<String, GxtError> {
    let vk = key.verifying_key().to_bytes();
    let (_, pk) = derive_enc_from_signing(key);
    let b0 = bytes0(&vk, &pk, kind, payload.clone())?;
    if b0.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }

    let id = blake3::hash(&b0);
    let sig = key.sign(&preimage(&b0));

    encode_message(
        1,
        &vk,
        &pk,
        kind,
        payload,
        parent,
        id.as_bytes(),
        &sig.to_bytes(),
    )
}

#[allow(clippy::too_many_arguments)]
fn encode_message(
    v: u8,
    vk: &Bytes32,
    pk: &Bytes32,
    kind: PayloadKind,
    payload: Value,
    parent: Option<Bytes32>,
    id: &Bytes32,
    sig: &Bytes64,
) -> Result<String, GxtError> {
    let cbor = cbor_array(v, vk, pk, kind, payload, parent, Some(id), Some(sig))?;
    if cbor.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }
    let mut comp = Vec::new();
    brotli::CompressorWriter::new(&mut comp, 4096, 5, 20).write_all(&cbor)?;
    Ok(format!("{}{}", PREFIX, bs58::encode(comp).into_string()))
}

fn decode_message(message: &str) -> Result<Vec<u8>, GxtError> {
    let rest = message.strip_prefix(PREFIX).ok_or(GxtError::BadPrefix)?;
    let comp = bs58::decode(rest).into_vec()?;
    let mut raw = Vec::new();
    brotli::Decompressor::new(&comp[..], 4096).read_to_end(&mut raw)?;
    if raw.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }
    Ok(raw)
}

fn parse_hex<const SIZE: usize>(h: &str) -> Result<[u8; SIZE], GxtError> {
    let v = hex::decode(h)?;

    let len = v.len();
    let a: [u8; SIZE] = v.try_into().map_err(|_| GxtError::InvalidHexSize {
        expected: SIZE,
        got: len,
    })?;
    Ok(a)
}

fn parse_key(h: &str) -> Result<SigningKey, GxtError> {
    Ok(SigningKey::from_bytes(&parse_hex::<32>(h)?))
}

fn derive_enc_from_signing(key: &SigningKey) -> (Bytes32, Bytes32) {
    let seed = key.to_bytes();
    let dk = blake3::derive_key("GXT-ENC-X25519-FROM-ED25519", &seed);
    let esk = XSecret::from(dk);
    let epk = XPublicKey::from(&esk);
    (esk.to_bytes(), epk.to_bytes())
}

fn enc_derive_key_from_pairs(my_esk: &Bytes32, their_epk: &Bytes32) -> Key {
    let key = XSecret::from(*my_esk);
    let vk = XPublicKey::from(*their_epk);
    let shared = key.diffie_hellman(&vk);
    let k = blake3::derive_key("GXT-ENC-XCHACHA20POLY1305", shared.as_bytes());
    Key::from_slice(&k).to_owned()
}
