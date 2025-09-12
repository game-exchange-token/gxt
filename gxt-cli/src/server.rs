use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use thiserror::Error;

use ed25519_dalek::{SigningKey as Ed25519Secret, VerifyingKey as Ed25519Public};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use x25519_dalek::StaticSecret as X25519Secret;

const SALT_TIMEL0CK: &[u8] = b"gxt-timelock-salt:v1";
const INFO_TIMEL0CK: &[u8] = b"gxt-timelock|kdf:v1";
const INFO_X25519_SK: &[u8] = b"gxt-x25519-sk:v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicTimelock {
    pub timestamp: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateTimelock {
    pub timestamp: String,
    pub label: String,
    pub secret_key: String,
}

pub fn derive_timelock_x25519(master_secret: &[u8; 32], tl: &PublicTimelock) -> X25519Secret {
    let ctx = [
        b"T=" as &[u8],
        tl.timestamp.as_bytes(),
        b"|L=",
        tl.label.as_bytes(),
    ]
    .concat();

    let hkdf = Hkdf::<Sha256>::new(Some(SALT_TIMEL0CK), master_secret);
    let mut seed = [0u8; 32];
    hkdf.expand(&[INFO_TIMEL0CK, &ctx].concat(), &mut seed)
        .expect("HKDF expand");

    let mut secret_key_material = [0u8; 32];
    let hkdf = Hkdf::<Sha256>::new(Some(SALT_TIMEL0CK), &seed);
    hkdf.expand(INFO_X25519_SK, &mut secret_key_material)
        .expect("HKDF expand 2");

    let secret_key = X25519Secret::from(secret_key_material);
    seed.fill(0);
    secret_key_material.fill(0);

    secret_key
}

#[derive(Clone)]
struct AppState {
    key: [u8; 32],
    _verify_id: Ed25519Public,
}

#[derive(Deserialize)]
struct PublicQuery {
    timestamp: String,
    #[serde(default)]
    label: String,
}

#[derive(Error, Debug)]
enum ApiErr {
    #[error("bad timestamp")]
    BadTs,
    #[error("not yet available")]
    NotYet,
    #[error("internal")]
    _Internal,
}

fn parse_timestamp(timestamp: &str) -> Result<OffsetDateTime, ApiErr> {
    OffsetDateTime::parse(timestamp, &Rfc3339).map_err(|_| ApiErr::BadTs)
}

async fn get_public(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PublicQuery>,
) -> Result<String, (StatusCode, String)> {
    let _ =
        parse_timestamp(&query.timestamp).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let timelock = PublicTimelock {
        timestamp: query.timestamp,
        label: query.label,
    };
    let secret_key = derive_timelock_x25519(&state.key, &timelock);

    let id_card = gxt::make_id_card(hex::encode(secret_key.as_bytes()).as_str(), timelock)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(id_card)
}

async fn get_private(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<PublicTimelock>,
) -> Result<String, (StatusCode, String)> {
    let timestamp =
        parse_timestamp(&query.timestamp).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let now = OffsetDateTime::now_utc();

    if now < timestamp {
        return Err((StatusCode::FORBIDDEN, ApiErr::NotYet.to_string()));
    }

    let timelock = PublicTimelock {
        timestamp: query.timestamp,
        label: query.label,
    };
    let secret_key = derive_timelock_x25519(&state.key, &timelock);
    let secret_key = hex::encode(secret_key.to_bytes());
    let id_card = headers
        .get("id_card")
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Bad Request".to_string()))?;
    let private_timelock = PrivateTimelock {
        label: timelock.label,
        timestamp: timelock.timestamp,
        secret_key,
    };
    let encrypted_message = gxt::encrypt_message(
        &hex::encode(state.key),
        id_card
            .to_str()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        &private_timelock,
        None,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(encrypted_message)
}

pub async fn serve(listen: SocketAddr, key: PathBuf) -> anyhow::Result<()> {
    let key = std::fs::read_to_string(key)?;
    let key: [u8; 32] = hex::decode(key)?.try_into().unwrap();
    let sign_id = Ed25519Secret::from_bytes(&[42u8; 32]);
    let verify_id = sign_id.verifying_key();

    let state = Arc::new(AppState {
        key,
        _verify_id: verify_id,
    });

    let app = Router::new()
        .route("/v1/tlock/public", get(get_public))
        .route("/v1/tlock/private", get(get_private))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}
