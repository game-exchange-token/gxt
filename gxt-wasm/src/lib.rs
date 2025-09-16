#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn make_key() -> String {
    gxt::make_key()
}

#[wasm_bindgen]
pub fn make_id_card(key: &str, meta: JsValue) -> Result<String, JsValue> {
    let meta: serde_json::Value = serde_wasm_bindgen::from_value(meta)?;
    Ok(gxt::make_id_card(key, meta).map_err(|e| e.to_string())?)
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum WasmPayloadKind {
    /// ID card
    Id,
    /// Message
    Msg,
    /// A key packaged into a gxt token
    Key,
}

impl From<gxt::PayloadKind> for WasmPayloadKind {
    fn from(value: gxt::PayloadKind) -> Self {
        match value {
            gxt::PayloadKind::Id => Self::Id,
            gxt::PayloadKind::Msg => Self::Msg,
            gxt::PayloadKind::Key => Self::Key,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WasmEnvelope {
    /// Version
    pub version: u8,
    /// Verification Key
    pub verification_key: String,
    /// Public Key
    pub encryption_key: String,
    /// Payload Kind
    pub kind: WasmPayloadKind,
    /// Opaque Payload
    pub payload: String,
    /// Id of the Parent Message
    pub parent: Option<String>,
    /// Id of this Message
    pub id: String,
    /// Signature of this Message
    pub signature: String,
}

impl From<gxt::Envelope<serde_json::Value>> for WasmEnvelope {
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
        }: gxt::Envelope<serde_json::Value>,
    ) -> Self {
        Self {
            version,
            verification_key,
            encryption_key,
            kind: kind.into(),
            payload: serde_json::to_string(&payload).unwrap(),
            parent,
            id,
            signature,
        }
    }
}

#[wasm_bindgen]
pub fn verify_message(msg: &str) -> Result<JsValue, JsValue> {
    let envelope = gxt::verify_message::<serde_json::Value>(msg).map_err(|e| e.to_string())?;
    let wasm_envelope: WasmEnvelope = envelope.into();
    Ok(serde_wasm_bindgen::to_value(&wasm_envelope)?)
}

#[wasm_bindgen]
pub fn encrypt_message(
    key: &str,
    id_card: &str,
    payload: JsValue,
    parent: Option<String>,
) -> Result<String, JsValue> {
    let payload: serde_json::Value = serde_wasm_bindgen::from_value(payload)?;
    Ok(gxt::encrypt_message(key, id_card, &payload, parent).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn decrypt_message(message: &str, key: &str) -> Result<JsValue, JsValue> {
    let envelope =
        gxt::decrypt_message::<serde_json::Value>(message, key).map_err(|e| e.to_string())?;
    let wasm_envelope: WasmEnvelope = envelope.into();
    Ok(serde_wasm_bindgen::to_value(&wasm_envelope)?)
}
