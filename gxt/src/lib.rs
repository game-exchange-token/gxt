#![forbid(unsafe_code)]

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde_cbor::Value;
use std::fmt;
use std::io::{Read, Write};
use std::str::FromStr;
use thiserror::Error;

const PREFIX: &str = "gxt:";
const SIG_DOMAIN: &[u8] = b"GXT";
const MAX_RAW: usize = 64 * 1024;

type Bytes32 = [u8; 32];
type Bytes64 = [u8; 64];

#[derive(Error, Debug)]
pub enum GxtError {
    #[error("bad prefix")]
    BadPrefix,
    #[error("decode error: {0}")]
    Decode(String),
    #[error("compress/decompress error: {0}")]
    Decompress(String),
    #[error("cbor error: {0}")]
    Cbor(#[from] serde_cbor::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("too large")]
    TooLarge,
    #[error("invalid signature")]
    BadSig,
    #[error("invalid id")]
    BadId,
    #[error("invalid message type")]
    MessageType,
    #[error("bad hex")]
    BadHex(#[from] hex::FromHexError),
    #[error("payload required")]
    PayloadRequired,
    #[error("meta data required")]
    MetaRequired,
    #[error("invalid hex size. expected {expected} got {len}")]
    InvalidHexSize { expected: usize, len: usize },
    #[error("access denied")]
    AccessDenied,
    #[error("invalid record")]
    Invalid,
    #[error("invalid payload kind")]
    InvalidPayloadKind,
}

#[derive(Copy, Clone, Debug)]
pub enum PayloadKind {
    Id,
    Msg,
}

impl FromStr for PayloadKind {
    type Err = GxtError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "id" => Ok(PayloadKind::Id),
            "msg" => Ok(PayloadKind::Msg),
            _ => Err(GxtError::InvalidPayloadKind),
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

#[derive(Clone, Debug)]
pub struct Rec {
    pub v: u8,
    pub vk: String,
    pub pk: String,
    pub kind: PayloadKind,
    pub payload: Value,
    pub parent: Option<String>,
    pub id: String,
    pub sig: String,
}

impl fmt::Display for Rec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "valid   : true")?;
        writeln!(f, "version : {}", self.v)?;
        match &self.parent {
            Some(parent) => writeln!(f, "parent  : {} ({})", parent, &parent[..8])?,
            None => writeln!(f, "parent  : -")?,
        }
        writeln!(f, "id      : {} ({})", self.id, &self.id[..8])?;
        writeln!(f, "vk      : {} ({})", self.vk, &self.vk[..8])?;
        writeln!(f, "pk      : {} ({})", self.pk, &self.pk[..8])?;
        writeln!(f, "kind    : {}", self.kind)?;
        writeln!(f, "payload :")?;
        writeln!(
            f,
            "{}",
            serde_json::to_string_pretty(&payload_to_json(&self.payload).map_err(|_| fmt::Error)?)
                .map_err(|_| fmt::Error)?
        )?;
        Ok(())
    }
}

fn payload_to_json(p: &serde_cbor::Value) -> Result<serde_json::Value, GxtError> {
    use serde_cbor::Value::*;
    fn conv(v: &serde_cbor::Value) -> Result<serde_json::Value, GxtError> {
        let result = match v {
            Bool(b) => serde_json::Value::Bool(*b),
            Integer(i) => {
                if let Ok(ii64) = i64::try_from(*i) {
                    serde_json::Value::Number(serde_json::Number::from(ii64))
                } else {
                    serde_json::Value::String(i.to_string())
                }
            }
            Text(s) => serde_json::Value::String(s.clone()),
            Array(a) => {
                serde_json::Value::Array(a.iter().map(conv).collect::<Result<Vec<_>, _>>()?)
            }
            Map(m) => {
                let mut o = serde_json::Map::new();
                for (k, v) in m {
                    let key = if let Text(s) = k {
                        s.clone()
                    } else {
                        format!("{:?}", k)
                    };
                    o.insert(key, conv(v)?);
                }
                serde_json::Value::Object(o)
            }
            Tag(_, x) => conv(x)?,
            _ => serde_json::Value::Null,
        };
        Ok(result)
    }
    conv(p)
}

#[allow(clippy::too_many_arguments)]
fn cbor_array(
    v: u8,
    vk: &Bytes32,
    pk: &Bytes32,
    kind: PayloadKind,
    payload: &Value,
    parent: Option<Bytes32>,
    id: Option<&Bytes32>,
    sig: Option<&Bytes64>,
) -> Result<Vec<u8>, GxtError> {
    let arr = Value::Array(vec![
        Value::Integer(v.into()),
        Value::Text(hex(vk)),
        Value::Text(hex(pk)),
        Value::Text(kind.to_string()),
        payload.clone(),
        Value::Text(parent.map(|v| hex(&v)).unwrap_or_default()),
        Value::Text(id.map(|v| hex(v)).unwrap_or_default()),
        Value::Text(sig.map(|v| hex(v)).unwrap_or_default()),
    ]);
    Ok(serde_cbor::to_vec(&arr)?)
}

fn bytes0(
    vk: &Bytes32,
    pk: &Bytes32,
    kind: PayloadKind,
    payload: &Value,
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
    sk: &SigningKey,
    kind: PayloadKind,
    payload: &Value,
    parent: Option<Bytes32>,
) -> Result<String, GxtError> {
    let vk = sk.verifying_key().to_bytes();
    let (_, pk) = derive_enc_from_signing(sk);
    let b0 = bytes0(&vk, &pk, kind, payload)?;
    if b0.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }

    let mut id = [0u8; 32];
    id.copy_from_slice(blake3::hash(&b0).as_bytes());
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&sk.sign(&preimage(&b0)).to_bytes());

    encode_token(1, &vk, &pk, kind, payload, parent, &id, &sig)
}

pub fn make_key() -> Result<String, GxtError> {
    let sk = SigningKey::generate(&mut OsRng);
    Ok(hex::encode(sk.to_bytes()))
}

pub fn make_identity(sk: &str, meta: &str) -> Result<String, GxtError> {
    let sk = parse_sk(sk.trim())?;
    let meta = parse_json_to_cbor(meta.trim())?.ok_or(GxtError::PayloadRequired)?;
    make(&sk, PayloadKind::Id, &meta, None)
}

#[allow(clippy::too_many_arguments)]
fn encode_token(
    v: u8,
    vk: &Bytes32,
    pk: &Bytes32,
    kind: PayloadKind,
    payload: &Value,
    parent: Option<Bytes32>,
    id: &Bytes32,
    sig: &Bytes64,
) -> Result<String, GxtError> {
    let cbor = cbor_array(v, vk, pk, kind, payload, parent, Some(id), Some(sig))?;
    if cbor.len() > MAX_RAW {
        return Err(GxtError::TooLarge);
    }
    let mut comp = Vec::new();
    brotli::CompressorWriter::new(&mut comp, 4096, 5, 20)
        .write_all(&cbor)
        .map_err(|e| GxtError::Decompress(e.to_string()))?;
    Ok(format!("{}{}", PREFIX, bs58::encode(comp).into_string()))
}

fn decode_token(token: &str) -> Result<Vec<u8>, GxtError> {
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

fn parse_hex_unsized(h: &str) -> Result<Vec<u8>, GxtError> {
    Ok(hex::decode(h)?)
}

fn parse_hex<const SIZE: usize>(h: &str) -> Result<[u8; SIZE], GxtError> {
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

fn parse_sk(h: &str) -> Result<SigningKey, GxtError> {
    let v = parse_hex::<32>(h)?;
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    Ok(SigningKey::from_bytes(&a))
}

pub fn verify(token: &str) -> Result<Rec, GxtError> {
    let raw = decode_token(token.trim())?;
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

    let b0 = bytes0(&vk_bytes, &pk, kind, &payload)?;
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
        vk: hex(&vk_bytes),
        pk: hex(&pk),
        parent: parent.map(|b| hex(&b)),
        kind,
        payload,
        id: hex(&id),
        sig: hex(&sig),
    })
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::RngCore;
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret as XSecret};

fn derive_enc_from_signing(sk: &SigningKey) -> (Bytes32, Bytes32) {
    let seed = sk.to_bytes();
    let dk = blake3::derive_key("GXT-ENC-X25519-FROM-ED25519", &seed);
    let esk = XSecret::from(dk);
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

pub fn make_encrypted_message(
    sk: &str,
    id_card: &str,
    body: &str,
    parent: Option<String>,
) -> Result<String, GxtError> {
    let id_card = verify(id_card.trim())?;
    let pk = parse_hex::<32>(&id_card.pk)?;
    let parent = match parent {
        Some(h) => Some(parse_hex::<32>(&h)?),
        None => None,
    };
    let sk = parse_sk(sk.trim())?;
    let body = parse_json_to_cbor(body.trim())?.ok_or(GxtError::PayloadRequired)?;
    let (my_esk, my_epk) = derive_enc_from_signing(&sk);
    let key = enc_derive_key_from_pairs(&my_esk, &pk);
    let cipher = XChaCha20Poly1305::new(&key);
    let mut n = [0u8; 24];
    OsRng.fill_bytes(&mut n);
    let nonce = XNonce::from_slice(&n);
    let pt = serde_cbor::to_vec(&body)?;
    let ct = cipher
        .encrypt(nonce, pt.as_ref())
        .map_err(|e| GxtError::Decompress(e.to_string()))?;

    let mut m = std::collections::BTreeMap::new();
    m.insert(Value::Text("to".into()), Value::Text(hex(&pk)));
    m.insert(Value::Text("from".into()), Value::Text(hex(&my_epk)));
    let mut encm = std::collections::BTreeMap::new();
    encm.insert(
        Value::Text("alg".into()),
        Value::Text("xchacha20poly1305".into()),
    );
    encm.insert(Value::Text("n24".into()), Value::Text(hex(&n)));
    encm.insert(Value::Text("ct".into()), Value::Text(hex(&ct)));
    m.insert(Value::Text("enc".into()), Value::Map(encm));
    let payload = Value::Map(m);
    make(&sk, PayloadKind::Msg, &payload, parent)
}

pub fn decrypt_message(token: &str, sk: &str) -> Result<Rec, GxtError> {
    let mut rec = match verify(token.trim()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("invalid token: {e}");
            std::process::exit(1);
        }
    };
    let sk = SigningKey::from_bytes(&parse_hex::<32>(sk.trim())?);
    let map = match &rec.payload {
        Value::Map(m) => m,
        _ => return Err(GxtError::Invalid),
    };
    let to = match map.get(&Value::Text("to".into())) {
        Some(Value::Text(t)) => parse_hex::<32>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let from = match map.get(&Value::Text("from".into())) {
        Some(Value::Text(t)) => parse_hex::<32>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let encm = match map.get(&Value::Text("enc".into())) {
        Some(Value::Map(m)) => m,
        _ => return Err(GxtError::Invalid),
    };
    let n = match encm.get(&Value::Text("n24".into())) {
        Some(Value::Text(t)) => parse_hex::<24>(t)?,
        _ => return Err(GxtError::Invalid),
    };
    let ct = match encm.get(&Value::Text("ct".into())) {
        Some(Value::Text(t)) => parse_hex_unsized(t)?,
        _ => return Err(GxtError::Invalid),
    };

    let (_my_esk, my_epk) = derive_enc_from_signing(&sk);
    if to != my_epk {
        return Err(GxtError::AccessDenied);
    }

    let (my_esk, _) = derive_enc_from_signing(&sk);
    let key = enc_derive_key_from_pairs(&my_esk, &from);
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XNonce::from_slice(&n);
    let pt = cipher
        .decrypt(nonce, ct.as_ref())
        .map_err(|e| GxtError::Decompress(e.to_string()))?;
    rec.payload = serde_cbor::from_slice(&pt)?;

    Ok(rec)
}

fn parse_json_to_cbor(s: &str) -> Result<Option<Value>, GxtError> {
    let cbor = if s.trim().is_empty() {
        None
    } else {
        let v: serde_json::Value = serde_json::from_str(s)?;
        Some(serde_cbor::value::to_value(&v)?)
    };
    Ok(cbor)
}
