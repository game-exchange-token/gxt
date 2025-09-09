#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
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
    Keygen {
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    Id {
        #[arg(short, long)]
        out: Option<PathBuf>,
        #[arg(short, long)]
        meta: Option<String>,
        key: PathBuf,
    },

    Msg {
        #[arg(short, long)]
        out: Option<PathBuf>,
        #[arg(short, long)]
        parent: Option<String>,
        #[arg(short, long)]
        body: Option<String>,
        key: PathBuf,
        id_card: PathBuf,
    },
    Decrypt {
        key: PathBuf,
        msg: Option<PathBuf>,
    },

    Verify {
        msg: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Keygen { out } => {
            let token = gxt::make_key()?;
            write_out_string(&token, out.as_deref())?;
        }
        Cmd::Id { out, key, meta } => {
            let sk_hex = std::fs::read_to_string(key)?;
            let meta_json = read_payload_opt(&meta)?;
            let token = gxt::make_identity(&sk_hex, &meta_json)?;
            write_out_string(&token, out.as_deref())?;
        }
        Cmd::Msg {
            key,
            id_card,
            parent,
            body,
            out,
        } => {
            let sk = fs::read_to_string(key)?;
            let id_card = fs::read_to_string(id_card)?;
            let body = read_payload_opt(&body)?;
            let tok = gxt::make_encrypted_message(&sk, &id_card, &body, parent)?;
            write_out_string(&tok, out.as_deref())?;
        }
        Cmd::Decrypt { key, msg } => {
            let token = read_all_opt(msg.as_ref())?;
            let sk = std::fs::read_to_string(key)?;
            match gxt::decrypt_message::<serde_json::Value>(&token, &sk) {
                Ok(val) => {
                    println!("{}", serde_json::to_string_pretty(&val)?);
                }
                Err(e) => {
                    eprintln!("decrypt error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Cmd::Verify { msg } => {
            let token = read_all_opt(msg.as_ref())?;
            match gxt::verify(&token) {
                Ok(rec) => {
                    println!("{rec}");
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
