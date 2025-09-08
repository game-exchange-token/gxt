#![forbid(unsafe_code)]

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use gxt::{parse_hex, verify};
use rand::rngs::OsRng;
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "gxt", version, about = "GXT (CLI wrapping the gxt library)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate an Ed25519 keypair (prints sk-hex and pk-hex)
    Keygen {
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Create an identity token: read JSON meta from STDIN (or empty for null)
    Id {
        #[arg(short, long)]
        out: Option<PathBuf>,
        #[arg(short, long)]
        meta: Option<String>,
        key: PathBuf,
    },

    /// Create an encrypted message: read JSON body from STDIN
    Msg {
        #[arg(short, long)]
        out: Option<PathBuf>,
        #[arg(short, long)]
        parent: Option<String>,
        #[arg(short, long)]
        meta: Option<String>,
        key: PathBuf,
        id_card: PathBuf,
    },
    /// Decrypt an encrypted message and print the body JSON
    Decrypt { key: PathBuf, msg: Option<PathBuf> },

    /// Verify a token string (prints details; exits 0/1)
    Verify { msg: Option<PathBuf> },
    // /// Create a message token: read JSON body from STDIN (or empty for null)
    // Msg {
    //     sk_hex: String,
    //     parent_hex32: Option<String>,
    // },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Msg {
            key,
            id_card,
            parent,
            meta,
            out,
        } => {
            let sk_hex = fs::read_to_string(key)?;
            let sk = parse_sk(&sk_hex)?;
            let id_card_token = fs::read_to_string(id_card)?;
            let id_card = verify(&id_card_token)?;
            let pk = id_card.pk;
            let parent = match parent {
                Some(h) => Some(parse_hex::<32>(&h)?),
                None => None,
            };
            let body_json = read_stdin_json(&meta)?;
            let body_cbor = serde_cbor::value::to_value(body_json)?;
            let tok = gxt::make_encrypted_message(&sk, &pk, body_cbor, parent)?;
            write_out_string(&tok, out.as_deref())?;
        }
        // Cmd::Msg {
        //     sk_hex,
        //     parent_hex32,
        // } => {
        //     let sk = parse_sk(&sk_hex)?;
        //     let parent = match parent_hex32 {
        //         Some(h) => Some(parse_hex::<32>(&h)?),
        //         None => None,
        //     };
        //     let body_json = read_stdin_json()?;
        //     let body_cbor = serde_cbor::value::to_value(body_json)?;
        //     let token = gxt::make_message(&sk, body_cbor, parent)?;
        //     println!("{token}");
        // }
        Cmd::Decrypt { key, msg } => {
            let token = read_all_opt(msg.as_ref())?;
            let rec = match gxt::verify(&token) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("invalid token: {e}");
                    std::process::exit(1);
                }
            };
            let sk_hex = std::fs::read_to_string(key)?;
            let sk = SigningKey::from_bytes(&parse_hex::<32>(&sk_hex)?);
            match gxt::decrypt_body_for_with_signing(&rec, &sk) {
                Ok(val) => {
                    println!("{}", serde_json::to_string_pretty(&payload_to_json(&val))?);
                }
                Err(e) => {
                    eprintln!("decrypt error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Cmd::Keygen { out } => {
            let sk = SigningKey::generate(&mut OsRng);
            write_out_string(&hex::encode(sk.to_bytes()), out.as_deref())?;
        }
        Cmd::Id { out, key, meta } => {
            let sk_hex = std::fs::read_to_string(key)?;
            let sk = parse_sk(sk_hex.trim())?;
            let meta_json = read_stdin_json(&meta)?;
            let meta_cbor = serde_cbor::value::to_value(meta_json)?;
            let token = gxt::make_identity(&sk, meta_cbor)?;
            write_out_string(&token, out.as_deref())?;
        }
        Cmd::Verify { msg } => {
            let token = read_all_opt(msg.as_ref())?;
            match gxt::verify(&token) {
                Ok(rec) => {
                    let id_hex = gxt::hex(&rec.id);
                    let vk_hex = gxt::hex(&rec.vk);
                    let pk_hex = gxt::hex(&rec.pk);
                    println!("valid:true");
                    println!("version:{}", rec.v);
                    println!("id     :{} ({})", id_hex, &id_hex[..8]);
                    println!("vk     :{} ({})", vk_hex, &vk_hex[..8]);
                    println!("pk     :{} ({})", pk_hex, &pk_hex[..8]);
                    println!("payload:{}", gxt::payload_kind(&rec.payload));
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&payload_to_json(&rec.payload))?
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("valid:false\nerror:{e}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

/* ---------- tiny helpers ---------- */
fn write_out_string(s: &str, path: Option<&Path>) -> Result<()> {
    write_out_bytes(s.as_bytes(), path)
}

fn write_out_bytes(bytes: &[u8], path: Option<&Path>) -> Result<()> {
    match path {
        Some(p) => {
            fs::write(p, bytes)?;
        }
        None => {
            io::stdout().write_all(bytes)?;
        }
    }
    Ok(())
}

fn read_all_opt(path: Option<&PathBuf>) -> Result<String> {
    let mut s = String::new();
    match path {
        Some(p) => s = fs::read_to_string(p)?,
        None => _ = io::stdin().read_to_string(&mut s)?,
    }
    Ok(s)
}

fn read_payload_opt(spec: &Option<String>) -> Result<String> {
    if let Some(s) = spec {
        Ok(s.to_string())
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    }
}

fn parse_sk(h: &str) -> Result<SigningKey> {
    let v = hex::decode(h).context("bad hex")?;
    if v.len() != 32 {
        bail!("secret key must be 32 bytes");
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    Ok(SigningKey::from_bytes(&a))
}

fn read_stdin_json(spec: &Option<String>) -> Result<serde_json::Value> {
    let s = read_payload_opt(spec)?;
    if s.trim().is_empty() {
        Ok(serde_json::Value::Null)
    } else {
        Ok(serde_json::from_str(&s)?)
    }
}

fn payload_to_json(p: &serde_cbor::Value) -> serde_json::Value {
    use serde_cbor::Value::*;
    fn conv(v: &serde_cbor::Value) -> serde_json::Value {
        match v {
            Bool(b) => serde_json::Value::Bool(*b),
            Integer(i) => {
                if let Ok(ii64) = i64::try_from(*i) {
                    serde_json::Value::Number(serde_json::Number::from(ii64))
                } else {
                    serde_json::Value::String(i.to_string())
                }
            }
            Bytes(b) => serde_json::Value::String(format!("0x{}", hex::encode(b))),
            Float(_) => serde_json::Value::Null,
            Text(s) => serde_json::Value::String(s.clone()),
            Array(a) => serde_json::Value::Array(a.iter().map(conv).collect()),
            Map(m) => {
                let mut o = serde_json::Map::new();
                for (k, v) in m {
                    let key = if let Text(s) = k {
                        s.clone()
                    } else {
                        format!("{:?}", k)
                    };
                    o.insert(key, conv(v));
                }
                serde_json::Value::Object(o)
            }
            Tag(_, x) => conv(x),
            _ => serde_json::Value::Null,
        }
    }
    if let Array(a) = p {
        if a.len() == 2 {
            if let Text(t) = &a[0] {
                if t == "id" {
                    return serde_json::json!({"k":"id","v": conv(&a[1])});
                }
                if t == "msg" {
                    return serde_json::json!({"k":"msg","v": conv(&a[1])});
                }
            }
        }
    }
    serde_json::json!({"k":"?","v":null})
}
