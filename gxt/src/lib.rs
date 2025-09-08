#![forbid(unsafe_code)]

use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use serde_cbor::Value;
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
    #[error("invalid record")]
    Invalid,
}

#[derive(Clone, Debug)]
pub struct Rec {
    pub v: u8,          // =1
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
    let mut m = serde_cbor::value::Map::new();
    if let Some(p) = parent {
        m.insert(Value::Text("parent".into()), Value::Bytes(p.to_vec()));
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
                return if t == "id" { "id" } else if t == "msg" { "msg" } else { "?" };
            }
        }
    }
    "?"
}

/* ---------- encode/verify ---------- */

fn cbor_array(
    v: u8,
    pk: &Bytes32,
    payload: &Value,
    id: Option<&Bytes32>,
    sig: Option<&Bytes64>,
) -> Result<Vec<u8>, GxtError> {
    let arr = Value::Array(vec![
        Value::Integer(v.into()),
        Value::Bytes(pk.to_vec()),
        payload.clone(),
        Value::Bytes(id.map(|x| x.to_vec()).unwrap_or_default()),
        Value::Bytes(sig.map(|x| x.to_vec()).unwrap_or_default()),
    ]);
    serde_cbor::to_vec(&arr).map_err(|e| GxtError::Cbor(e.to_string()))
}

fn bytes0(pk: &Bytes32, payload: &Value) -> Result<Vec<u8>, GxtError> {
    cbor_array(1, pk, payload, None, None)
}

fn preimage(bytes0: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(SIG_DOMAIN.len() + bytes0.len());
    v.extend_from_slice(SIG_DOMAIN);
    v.extend_from_slice(bytes0);
    v
}

/// Create any payload token (low-level).
pub fn make(sk: &SigningKey, payload: &Value) -> Result<String, GxtError> {
    let pk = sk.verifying_key().to_bytes();
    let b0 = bytes0(&pk, payload)?;
    if b0.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }

    let mut id = [0u8; 32];
    id.copy_from_slice(blake3::hash(&b0).as_bytes());
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&sk.sign(&preimage(&b0)).to_bytes());

    encode_token(1, &pk, payload, &id, &sig)
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
    pk: &Bytes32,
    payload: &Value,
    id: &Bytes32,
    sig: &Bytes64,
) -> Result<String, GxtError> {
    let cbor = cbor_array(v, pk, payload, Some(id), Some(sig))?;
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

/// Verify and parse a token into a record.
pub fn verify(token: &str) -> Result<Rec, GxtError> {
    let raw = decode_token(token)?;
    let val: Value =
        serde_cbor::from_slice(&raw).map_err(|e| GxtError::Cbor(e.to_string()))?;
    let a = match val {
        Value::Array(a) if a.len() == 5 => a,
        _ => return Err(GxtError::Invalid),
    };

    // v
    let v = match &a[0] {
        Value::Integer(i) if *i == 1.into() => 1u8,
        _ => return Err(GxtError::Invalid),
    };
    // pk
    let pk = match &a[1] {
        Value::Bytes(b) if b.len() == 32 => {
            let mut x = [0u8; 32];
            x.copy_from_slice(b);
            x
        }
        _ => return Err(GxtError::Invalid),
    };
    // payload
    let payload = match &a[2] {
        Value::Array(two) if two.len() == 2 => match &two[0] {
            Value::Text(t) if t == "id" || t == "msg" => {
                Value::Array(vec![two[0].clone(), two[1].clone()])
            }
            _ => return Err(GxtError::Invalid),
        },
        _ => return Err(GxtError::Invalid),
    };
    // id
    let id = match &a[3] {
        Value::Bytes(b) if b.len() == 32 => {
            let mut x = [0u8; 32];
            x.copy_from_slice(b);
            x
        }
        _ => return Err(GxtError::Invalid),
    };
    // sig
    let sig = match &a[4] {
        Value::Bytes(b) if b.len() == 64 => {
            let mut x = [0u8; 64];
            x.copy_from_slice(b);
            x
        }
        _ => return Err(GxtError::Invalid),
    };

    // recompute bytes0/id and verify sig
    let b0 = bytes0(&pk, &payload)?;
    let expect = blake3::hash(&b0);
    if id != *expect.as_bytes() {
        return Err(GxtError::BadId);
    }

    let vk = VerifyingKey::from_bytes(&pk).map_err(|_| GxtError::Invalid)?;
    let sigv = Signature::from_bytes(&sig).map_err(|_| GxtError::Invalid)?;
    vk.verify_strict(&preimage(&b0), &sigv)
        .map_err(|_| GxtError::BadSig)?;

    Ok(Rec {
        v,
        pk,
        payload,
        id,
        sig,
    })
}

/* ---------- small utils for consumers ---------- */

pub fn hex32(b: &Bytes32) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}
