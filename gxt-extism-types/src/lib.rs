#![allow(clippy::unnecessary_wraps)]

use extism_convert::*;
use serde::{Deserialize, Serialize};

pub mod exports {
    pub const MAKE_KEY: &str = "make_key";
    pub const MAKE_ID_CARD: &str = "make_id_card";
    pub const VERIFY: &str = "verify";
    pub const ENCRYPT: &str = "encrypt";
    pub const DECRYPT: &str = "decrypt";
}

#[derive(FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub struct IdCardRequest {
    pub key: String,
    pub meta: serde_json::Value,
}

#[derive(FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub enum PayloadKind {
    Id,
    Msg,
}

impl From<gxt::PayloadKind> for PayloadKind {
    fn from(value: gxt::PayloadKind) -> Self {
        match value {
            gxt::PayloadKind::Id => PayloadKind::Id,
            gxt::PayloadKind::Msg => PayloadKind::Msg,
        }
    }
}

#[derive(FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub struct Envelope {
    pub version: u8,
    pub verification_key: String,
    pub encryption_key: String,
    pub kind: PayloadKind,
    pub payload: serde_json::Value,
    pub parent: Option<String>,
    pub id: String,
    pub signature: String,
}

impl From<gxt::Envelope> for Envelope {
    fn from(
        gxt::Envelope {
            version,
            verification_key,
            encryption_key,
            kind,
            payload,
            parent,
            id,
            signature,
        }: gxt::Envelope,
    ) -> Self {
        Envelope {
            version,
            verification_key,
            encryption_key,
            kind: kind.into(),
            payload: serde_cbor::value::from_value(payload)
                .expect("Could not convert payload from JSON to CBOR"),
            parent,
            id,
            signature,
        }
    }
}

#[derive(FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub struct EncryptRequest {
    pub key: String,
    pub id_card: String,
    pub body: serde_json::Value,
    pub parent: Option<String>,
}

#[derive(FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub struct DecryptRequest {
    pub message: String,
    pub key: String,
}
