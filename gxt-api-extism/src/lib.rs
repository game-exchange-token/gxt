use extism_pdk::*;
use serde::{Deserialize, Serialize};

// #[derive(ToBytes, Serialize, PartialEq, Debug)]
// #[encoding(Json)]
// pub enum PayloadKind {
//     Id,
//     Msg,
// }

// impl From<gxt::PayloadKind> for PayloadKind {
//     fn from(value: gxt::PayloadKind) -> Self {
//         match value {
//             gxt::PayloadKind::Id => Self::Id,
//             gxt::PayloadKind::Msg => Self::Msg,
//         }
//     }
// }

// #[derive(ToBytes, Serialize, PartialEq, Debug)]
// #[encoding(Json)]
// pub struct Envelope {
//     pub v: u8,
//     pub vk: String,
//     pub pk: String,
//     pub kind: PayloadKind,
//     pub payload: serde_cbor::Value,
//     pub parent: Option<String>,
//     pub id: String,
//     pub sig: String,
// }

// impl From<gxt::Envelope> for Envelope {
//     fn from(
//         gxt::Envelope {
//             v,
//             vk,
//             pk,
//             kind,
//             payload,
//             parent,
//             id,
//             sig,
//         }: gxt::Envelope,
//     ) -> Self {
//         Self {
//             v,
//             vk,
//             pk,
//             kind: kind.into(),
//             payload,
//             parent,
//             id,
//             sig,
//         }
//     }
// }

#[plugin_fn]
pub fn make_key() -> FnResult<String> {
    Ok(gxt::make_key())
}

// #[derive(FromBytes, Deserialize, PartialEq, Debug)]
// #[encoding(Json)]
// struct IdCardRequest {
//     key: String,
//     meta: serde_json::Value,
// }

// #[plugin_fn]
// pub fn make_id_card(req: IdCardRequest) -> FnResult<String> {
//     Ok(gxt::make_id_card(
//         &req.key,
//         &serde_json::to_string(&req.meta)?,
//     )?)
// }

// #[plugin_fn]
// pub fn verify(msg: String) -> FnResult<Envelope> {
//     Ok(gxt::verify_message(&msg).map(Into::into)?)
// }

// #[derive(FromBytes, Deserialize, PartialEq, Debug)]
// #[encoding(Json)]
// pub struct EncryptRequest {
//     key: String,
//     id_card: String,
//     body: String,
//     parent: Option<String>,
// }

// #[plugin_fn]
// pub fn encrypt_message(req: EncryptRequest) -> FnResult<String> {
//     Ok(gxt::encrypt_message(
//         &req.key,
//         &req.id_card,
//         &req.body,
//         req.parent,
//     )?)
// }

// #[derive(FromBytes, Deserialize, PartialEq, Debug)]
// #[encoding(Json)]
// pub struct DecryptRequest {
//     msg: String,
//     key: String,
// }

// #[plugin_fn]
// pub fn decrypt_message(req: DecryptRequest) -> FnResult<Envelope> {
//     Ok(gxt::decrypt_message(&req.msg, &req.key).map(Into::into)?)
// }
