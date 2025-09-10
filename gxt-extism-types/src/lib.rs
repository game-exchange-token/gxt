#![allow(clippy::unnecessary_wraps)]

use extism_convert::*;
use serde::{Deserialize, Serialize};

pub use serde_json::{json, to_value};

#[derive(Clone, Debug, FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub struct IdCardRequest {
    pub key: String,
    pub meta: serde_json::Value,
}

#[derive(Clone, Debug, FromBytes, Deserialize, Serialize, ToBytes)]
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

#[derive(Clone, Debug, FromBytes, Deserialize, Serialize, ToBytes)]
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

#[derive(Clone, Debug, FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub struct EncryptRequest {
    pub key: String,
    pub id_card: String,
    pub body: serde_json::Value,
    pub parent: Option<String>,
}

#[derive(Clone, Debug, FromBytes, Deserialize, Serialize, ToBytes)]
#[encoding(Json)]
pub struct DecryptRequest {
    pub message: String,
    pub key: String,
}

#[allow(non_camel_case_types)]
pub mod calls {
    use crate::DecryptRequest;
    use crate::EncryptRequest;
    use crate::Envelope;
    use crate::IdCardRequest;

    pub const MAKE_KEY: &str = "make_key";
    pub type MAKE_KEY_IN = ();
    pub type MAKE_KEY_OUT = String;

    pub const MAKE_ID_CARD: &str = "make_id_card";
    pub type MAKE_ID_IN = IdCardRequest;
    pub type MAKE_ID_OUT = String;

    pub const VERIFY_MESSAGE: &str = "verify_message";
    pub type VERIFY_MESSAGE_IN = String;
    pub type VERIFY_MESSAGE_OUT = Envelope;

    pub const ENCRYPT_MESSAGE: &str = "encrypt_message";
    pub type ENCRYPT_MESSAGE_IN = EncryptRequest;
    pub type ENCRYPT_MESSAGE_OUT = String;

    pub const DECRYPT_MESSAGE: &str = "decrypt_message";
    pub type DECRYPT_MESSAGE_IN = DecryptRequest;
    pub type DECRYPT_MESSAGE_OUT = Envelope;
}
