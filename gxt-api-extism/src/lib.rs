#![allow(clippy::unnecessary_wraps)]

use extism_pdk::{FnResult, Json, plugin_fn};
use gxt_extism_types::*;

#[plugin_fn]
pub fn make_key() -> FnResult<String> {
    Ok(gxt::make_key())
}

#[plugin_fn]
pub fn make_id_card(Json(IdCardRequest { key, meta }): Json<IdCardRequest>) -> FnResult<String> {
    Ok(gxt::make_id_card(&key, meta)?)
}

#[plugin_fn]
pub fn verify_message(msg: String) -> FnResult<Json<Envelope>> {
    Ok(Json(gxt::verify_message::<serde_json::Value>(&msg)?.into()))
}

#[plugin_fn]
pub fn encrypt_message(
    Json(EncryptRequest {
        key,
        id_card,
        payload,
        parent,
    }): Json<EncryptRequest>,
) -> FnResult<String> {
    Ok(gxt::encrypt_message(&key, &id_card, payload, parent)?)
}

#[plugin_fn]
pub fn decrypt_message(
    Json(DecryptRequest { message, key }): Json<DecryptRequest>,
) -> FnResult<Json<Envelope>> {
    Ok(Json(
        gxt::decrypt_message::<serde_json::Value>(&message, &key)?.into(),
    ))
}
