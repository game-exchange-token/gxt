#![forbid(unsafe_code)]

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde_cbor::Value;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use thiserror::Error;

pub const PREFIX: &str = "gxt:";
pub const SIG_DOMAIN: &[u8] = b"GXT1";
pub const MAX_RAW: usize = 64 * 1024;

pub type Bytes32 = [u8; 32];
pub type Bytes64 = [u8; 64];

#[derive(Error, Debug)]
pub enum GxtError {
    #[error("bad prefix")]
    BadPrefix,
    #[error("decode error: {0}")]
    Decode(String),
    #[error("compress/decompress error: {0}")]
    Decompress(String),
    #[error("cbor error: {0}")]
    Cbor(String),
    #[error("too large")]
    TooLarge,
    #[error("invalid signature")]
    BadSig,
    #[error("invalid id")]
    BadId,
    #[error("bad hex")]
    BadHex(#[from] hex::FromHexError),
    #[error("invalid hex size. expected {expected} got {len}")]
    InvalidHexSize { expected: usize, len: usize },
    #[error("invalid record")]
    Invalid,
}

#[derive(Clone, Debug)]
pub struct Rec {
    pub v: u8, // =1
    pub vk: Bytes32,
    pub pk: Bytes32,
    pub payload: Value, // ["id", meta] OR ["msg", {parent?, body?}]
    pub id: Bytes32,
    pub sig: Bytes64,
}

/* ---------- payload helpers ---------- */

pub fn payload_id(meta: Value) -> Value {
    Value::Array(vec![Value::Text("id".into()), meta])
}

pub fn payload_msg(parent: Option<Bytes32>, body: Value) -> Value {
    let mut m: BTreeMap<serde_cbor::Value, serde_cbor::Value> = BTreeMap::new();
    if let Some(p) = parent {
        m.insert(Value::Text("parent".into()), Value::Text(hex(&p)));
    }
    if !matches!(body, Value::Null) {
        m.insert(Value::Text("body".into()), body);
    }
    Value::Array(vec![Value::Text("msg".into()), Value::Map(m)])
}

pub fn payload_kind(p: &Value) -> &'static str {
    if let Value::Array(a) = p {
        if a.len() == 2 {
            if let Value::Text(t) = &a[0] {
                return if t == "id" {
                    "id"
                } else if t == "msg" {
                    "msg"
                } else {
                    "?"
                };
            }
        }
    }
    "?"
}

/* ---------- encode/verify ---------- */

fn cbor_array(
    v: u8,
    vk: &Bytes32,
    pk: &Bytes32,
    payload: &Value,
    id: Option<&Bytes32>,
    sig: Option<&Bytes64>,
) -> Result<Vec<u8>, GxtError> {
    let arr = Value::Array(vec![
        Value::Integer(v.into()),
        Value::Text(hex(vk)),
        Value::Text(hex(pk)),
        payload.clone(),
        Value::Text(id.map(|v| hex(v)).unwrap_or_default()),
        Value::Text(sig.map(|v| hex(v)).unwrap_or_default()),
    ]);
    serde_cbor::to_vec(&arr).map_err(|e| GxtError::Cbor(e.to_string()))
}

fn bytes0(vk: &Bytes32, pk: &Bytes32, payload: &Value) -> Result<Vec<u8>, GxtError> {
    cbor_array(1, vk, pk, payload, None, None)
}

fn preimage(bytes0: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(SIG_DOMAIN.len() + bytes0.len());
    v.extend_from_slice(SIG_DOMAIN);
    v.extend_from_slice(bytes0);
    v
}

/// Create any payload token (low-level).
pub fn make(sk: &SigningKey, payload: &Value) -> Result<String, GxtError> {
    let vk = sk.verifying_key().to_bytes();
    let (_, pk) = derive_enc_from_signing(sk);
    let b0 = bytes0(&vk, &pk, payload)?;
    if b0.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }

    let mut id = [0u8; 32];
    id.copy_from_slice(blake3::hash(&b0).as_bytes());
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&sk.sign(&preimage(&b0)).to_bytes());

    encode_token(1, &vk, &pk, payload, &id, &sig)
}

/// Higher-level: identity token (payload = ["id", meta])
pub fn make_identity(sk: &SigningKey, meta: Value) -> Result<String, GxtError> {
    make(sk, &payload_id(meta))
}

/// Higher-level: message token (payload = ["msg", {parent?, body?}])
pub fn make_message(
    sk: &SigningKey,
    body: Value,
    parent: Option<Bytes32>,
) -> Result<String, GxtError> {
    make(sk, &payload_msg(parent, body))
}

/// Encode to `gxt:` string from parts.
pub fn encode_token(
    v: u8,
    vk: &Bytes32,
    pk: &Bytes32,
    payload: &Value,
    id: &Bytes32,
    sig: &Bytes64,
) -> Result<String, GxtError> {
    let cbor = cbor_array(v, vk, pk, payload, Some(id), Some(sig))?;
    if cbor.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }
    let mut comp = Vec::new();
    brotli::CompressorWriter::new(&mut comp, 4096, 5, 20)
        .write_all(&cbor)
        .map_err(|e| GxtError::Decompress(e.to_string()))?;
    Ok(format!("{}{}", PREFIX, bs58::encode(comp).into_string()))
}

/// Decode a `gxt:` string to raw CBOR bytes (after Base58+Brotli).
pub fn decode_token(token: &str) -> Result<Vec<u8>, GxtError> {
    let rest = token.strip_prefix(PREFIX).ok_or(GxtError::BadPrefix)?;
    let comp = bs58::decode(rest)
        .into_vec()
        .map_err(|e| GxtError::Decode(e.to_string()))?;
    let mut raw = Vec::new();
    brotli::Decompressor::new(&comp[..], 4096)
        .read_to_end(&mut raw)
        .map_err(|e| GxtError::Decompress(e.to_string()))?;
    if raw.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }
    Ok(raw)
}

pub fn parse_hex_unsized(h: &str) -> Result<Vec<u8>, GxtError> {
    Ok(hex::decode(h)?)
}

pub fn parse_hex<const SIZE: usize>(h: &str) -> Result<[u8; SIZE], GxtError> {
    let v = hex::decode(h)?;
    if v.len() != SIZE {
        return Err(GxtError::InvalidHexSize {
            expected: SIZE,
            len: v.len(),
        });
    }
    let mut a = [0u8; SIZE];
    a.copy_from_slice(&v);
    Ok(a)
}

/// Verify and parse a token into a record.
pub fn verify(token: &str) -> Result<Rec, GxtError> {
    let raw = decode_token(token)?;
    let val: Value = serde_cbor::from_slice(&raw).map_err(|e| GxtError::Cbor(e.to_string()))?;
    let a = match val {
        Value::Array(a) if a.len() == 6 => a,
        _ => return Err(GxtError::Invalid),
    };

    let mut a = a.into_iter();

    // v
    let v = match a.next() {
        Some(Value::Integer(i)) if i == 1.into() => 1u8,
        _ => return Err(GxtError::Invalid),
    };
    // vk
    let vk_bytes = match a.next() {
        Some(Value::Text(b)) => parse_hex::<32>(&b)?,
        _ => return Err(GxtError::Invalid),
    };
    // pk
    let pk = match a.next() {
        Some(Value::Text(b)) => parse_hex::<32>(&b)?,
        _ => return Err(GxtError::Invalid),
    };
    // payload
    let payload = match a.next() {
        Some(Value::Array(two)) if two.len() == 2 => match &two[0] {
            Value::Text(t) if t == "id" || t == "msg" => {
                Value::Array(vec![two[0].clone(), two[1].clone()])
            }
            _ => return Err(GxtError::Invalid),
        },
        _ => return Err(GxtError::Invalid),
    };
    // id
    let id = match a.next() {
        Some(Value::Text(b)) => parse_hex::<32>(&b)?,
        _ => return Err(GxtError::Invalid),
    };
    // sig
    let sig = match a.next() {
        Some(Value::Text(b)) => parse_hex::<64>(&b)?,
        _ => return Err(GxtError::Invalid),
    };

    // recompute bytes0/id and verify sig
    let b0 = bytes0(&vk_bytes, &pk, &payload)?;
    let expect = blake3::hash(&b0);
    if id != *expect.as_bytes() {
        return Err(GxtError::BadId);
    }

    let vk = VerifyingKey::from_bytes(&vk_bytes).map_err(|_| GxtError::Invalid)?;
    let sigv = Signature::from_bytes(&sig);
    vk.verify_strict(&preimage(&b0), &sigv)
        .map_err(|_| GxtError::BadSig)?;

    Ok(Rec {
        v,
        vk: vk_bytes,
        pk,
        payload,
        id,
        sig,
    })
}

/* ---------- small utils for consumers ---------- */

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/* ---------- encryption (derived from signing key) ---------- */
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret as XSecret};

/// Deterministically derive an X25519 keypair (esk, epk) from an Ed25519 signing key.
/// This lets one keypair control both signing and encryption without storing two secrets.
pub fn derive_enc_from_signing(sk: &SigningKey) -> (Bytes32, Bytes32) {
    // Take the 32-byte Ed25519 secret bytes and derive a 32-byte X25519 secret using BLAKE3.
    let seed = sk.to_bytes();
    let dk = blake3::derive_key("GXT-ENC-X25519-FROM-ED25519", &seed);
    let esk = XSecret::from(dk); // X25519 clamping applied internally
    let epk = XPublicKey::from(&esk);
    (esk.to_bytes(), epk.to_bytes())
}

fn enc_derive_key_from_pairs(my_esk: &Bytes32, their_epk: &Bytes32) -> Key {
    let sk = XSecret::from(*my_esk);
    let vk = XPublicKey::from(*their_epk);
    let shared = sk.diffie_hellman(&vk);
    let k = blake3::derive_key("GXT-ENC-XCHACHA20POLY1305", shared.as_bytes());
    Key::from_slice(&k).to_owned()
}

/// Create an encrypted message token using only the Ed25519 signing key for the sender.
/// The sender's X25519 keys are derived deterministically from the signing key.
pub fn make_encrypted_message(
    sk: &SigningKey,
    to_epk: &Bytes32,
    body: Value,
    parent: Option<Bytes32>,
) -> Result<String, GxtError> {
    // derive sender encryption keys
    let (my_esk, my_epk) = derive_enc_from_signing(sk);
    // Encrypt body with XChaCha20-Poly1305 using DH(shared) → key
    let key = enc_derive_key_from_pairs(&my_esk, to_epk);
    let cipher = XChaCha20Poly1305::new(&key);
    let mut n = [0u8; 24];
    OsRng.fill_bytes(&mut n);
    let nonce = XNonce::from_slice(&n);
    let pt = serde_cbor::to_vec(&payload_msg(parent, body))
        .map_err(|e| GxtError::Cbor(e.to_string()))?;
    let ct = cipher
        .encrypt(nonce, pt.as_ref())
        .map_err(|e| GxtError::Decompress(e.to_string()))?;

    // Build payload map: {to, from, enc:{alg,n24,ct}, parent?}
    let mut m = std::collections::BTreeMap::new();
    m.insert(Value::Text("to".into()), Value::Text(hex(to_epk)));
    m.insert(Value::Text("from".into()), Value::Text(hex(&my_epk)));
    let mut encm = std::collections::BTreeMap::new();
    encm.insert(
        Value::Text("alg".into()),
        Value::Text("xchacha20poly1305".into()),
    );
    encm.insert(Value::Text("n24".into()), Value::Text(hex(&n)));
    encm.insert(Value::Text("ct".into()), Value::Text(hex(&ct)));
    m.insert(Value::Text("enc".into()), Value::Map(encm));
    if let Some(p) = parent {
        m.insert(Value::Text("parent".into()), Value::Text(hex(&p)));
    }
    let payload = Value::Array(vec![Value::Text("msg".into()), Value::Map(m)]);
    make(sk, &payload)
}

/// Decrypt the encrypted `body` using the receiver's Ed25519 signing key
/// (from which we derive the X25519 secret).
pub fn decrypt_body_for_with_signing(rec: &Rec, my_sk: &SigningKey) -> Result<Value, GxtError> {
    // Extract enc fields
    let map = match &rec.payload {
        Value::Array(a) if a.len() == 2 => match (&a[0], &a[1]) {
            (Value::Text(t), Value::Map(m)) if t == "msg" => m,
            _ => return Err(GxtError::Invalid),
        },
        _ => return Err(GxtError::Invalid),
    };
    let to = match map.get(&Value::Text("to".into())) {
        Some(Value::Text(b)) => parse_hex::<32>(b)?,
        _ => return Err(GxtError::Invalid),
    };
    let from = match map.get(&Value::Text("from".into())) {
        Some(Value::Text(b)) => parse_hex::<32>(b)?,
        _ => return Err(GxtError::Invalid),
    };
    let encm = match map.get(&Value::Text("enc".into())) {
        Some(Value::Map(m)) => m,
        _ => return Err(GxtError::Invalid),
    };
    let n = match encm.get(&Value::Text("n24".into())) {
        Some(Value::Text(b)) => parse_hex::<24>(b)?,
        _ => return Err(GxtError::Invalid),
    };
    let ct = match encm.get(&Value::Text("ct".into())) {
        Some(Value::Text(b)) => parse_hex_unsized(b)?,
        _ => return Err(GxtError::Invalid),
    };

    // Check that `to` matches our derived epk
    let (_my_esk, my_epk) = derive_enc_from_signing(my_sk);
    if to != my_epk {
        return Err(GxtError::Invalid);
    }

    // Decrypt using derived esk and sender's epk
    let (my_esk, _) = derive_enc_from_signing(my_sk);
    let key = enc_derive_key_from_pairs(&my_esk, &from);
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XNonce::from_slice(&n);
    let pt = cipher
        .decrypt(nonce, ct.as_ref())
        .map_err(|e| GxtError::Decompress(e.to_string()))?;
    let val: Value = serde_cbor::from_slice(&pt).map_err(|e| GxtError::Cbor(e.to_string()))?;
    Ok(val)
}
