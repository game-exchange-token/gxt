#![forbid(unsafe_code)]

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::io::Read;

#[derive(Parser)]
#[command(name="gxt", version, about="GXT (CLI wrapping the gxt library)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate an Ed25519 keypair (prints sk-hex and pk-hex)
    Keygen,
    /// Create an identity token: read JSON meta from STDIN (or empty for null)
    Id { sk_hex: String },
    /// Create a message token: read JSON body from STDIN (or empty for null)
    Msg { sk_hex: String, parent_hex32: Option<String> },
    /// Verify a token string (prints details; exits 0/1)
    Verify { token: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Keygen => {
            let sk = SigningKey::generate(&mut OsRng);
            let pk = sk.verifying_key().to_bytes();
            println!("sk:{}", hex::encode(sk.to_bytes()));
            println!("pk:{}", hex::encode(pk));
        }
        Cmd::Id { sk_hex } => {
            let sk = parse_sk(&sk_hex)?;
            let meta_json = read_stdin_json()?;
            let meta_cbor = serde_cbor::value::to_value(meta_json)?;
            let token = gxt::make_identity(&sk, meta_cbor)?;
            println!("{token}");
        }
        Cmd::Msg { sk_hex, parent_hex32 } => {
            let sk = parse_sk(&sk_hex)?;
            let parent = match parent_hex32 {
                Some(h) => Some(parse_hex32(&h)?),
                None => None,
            };
            let body_json = read_stdin_json()?;
            let body_cbor = serde_cbor::value::to_value(body_json)?;
            let token = gxt::make_message(&sk, body_cbor, parent)?;
            println!("{token}");
        }
        Cmd::Verify { token } => {
            match gxt::verify(&token) {
                Ok(rec) => {
                    println!("valid:true");
                    println!("version:{}", rec.v);
                    println!("id     :{} ({})", gxt::hex32(&rec.id), &gxt::hex32(&rec.id)[..8]);
                    println!("pk     :{}...", &gxt::hex32(&rec.pk)[..16]);
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

fn parse_sk(h: &str) -> Result<SigningKey> {
    let v = hex::decode(h).context("bad hex")?;
    if v.len() != 32 {
        bail!("secret key must be 32 bytes");
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    Ok(SigningKey::from_bytes(&a))
}

fn parse_hex32(h: &str) -> Result<gxt::Bytes32> {
    let v = hex::decode(h).context("bad hex")?;
    if v.len() != 32 {
        bail!("expected 32 bytes hex");
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    Ok(a)
}

fn read_stdin_json() -> Result<serde_json::Value> {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s)?;
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
            Null => serde_json::Value::Null,
            Bool(b) => serde_json::Value::Bool(*b),
            Integer(i) => {
                        if let Ok(ii64) = i64::try_from(*i) {
                            serde_json::Value::Number(serde_json::Number::from(ii64))
                        } else {
                            serde_json::Value::String(i.to_string())
                        }
                    },
            Bytes(b) => serde_json::Value::String(format!("0x{}", hex::encode(b))),
            Float(_) => serde_json::Value::Null,
            Text(s) => serde_json::Value::String(s.clone()),
            Array(a) => serde_json::Value::Array(a.iter().map(conv).collect()),
            Map(m) => {
                        let mut o = serde_json::Map::new();
                        for (k, v) in m {
                            let key = if let Text(s) = k { s.clone() } else { format!("{:?}", k) };
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
                if t == "id" { return serde_json::json!({"k":"id","v": conv(&a[1])}); }
                if t == "msg" { return serde_json::json!({"k":"msg","v": conv(&a[1])}); }
            }
        }
    }
    serde_json::json!({"k":"?","v":null})
}
