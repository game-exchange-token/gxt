#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "gxt", version, about = "GXT (Game Exchange Token)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generates a new private key
    Keygen {
        /// Where to store the key
        #[arg(short, long)]
        out: PathBuf,
    },

    /// Generate an ID card that contains the public keys of the sender and some optional meta data
    Id {
        /// The key of the person creating the id card
        key: PathBuf,

        /// Meta data for the id card. Can be anything, but must be set. Pass - to read from stdin
        #[arg(short, long)]
        meta: String,

        /// Where to store the id card token
        #[arg(short, long)]
        out: Option<PathBuf>,
    },

    Msg {
        /// The key of the sender
        #[arg(short, long)]
        key: PathBuf,

        /// The id card of the recipient
        #[arg(short, long)]
        to: PathBuf,

        /// The parent of this message
        #[arg(short, long)]
        parent: Option<String>,

        /// The body of the message. Can be anything, but must be set. Pass - to read from stdin
        #[arg(short, long)]
        body: String,

        /// Where to store the message token
        #[arg(short, long)]
        out: Option<PathBuf>,
    },

    Decrypt {
        /// The key of the receiver
        #[arg(short, long)]
        key: PathBuf,

        /// The string token containing the encrypted message. Pass - to read from stdin
        msg: String,
    },

    Verify {
        /// The string token containing the message. Pass - to read from stdin
        msg: String,
    },

    DecryptFile {
        /// The key of the receiver
        #[arg(short, long)]
        key: PathBuf,

        /// The path to the encrypted message
        msg: PathBuf,
    },

    VerifyFile {
        /// The path to the message
        msg: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Keygen { out } => {
            let token = gxt::make_key()?;
            write_out_string(&token, Some(out.as_ref()))?;
        }
        Cmd::Id { out, key, meta } => {
            let sk_hex = fs::read_to_string(key)?;
            let meta_json = value_or_stdin(&meta)?;
            let token = gxt::make_identity(&sk_hex, &meta_json)?;
            write_out_string(&token, out.as_deref())?;
        }
        Cmd::Msg {
            key,
            to,
            parent,
            body,
            out,
        } => {
            let sk = fs::read_to_string(key)?;
            let id_card = fs::read_to_string(to)?;
            let body = value_or_stdin(&body)?;
            let tok = gxt::make_encrypted_message(&sk, &id_card, &body, parent)?;
            write_out_string(&tok, out.as_deref())?;
        }
        Cmd::Decrypt { key, msg } => {
            let token = value_or_stdin(&msg)?;
            let sk = fs::read_to_string(key)?;
            println!("{}", gxt::decrypt_message(&token, &sk)?);
        }
        Cmd::Verify { msg } => {
            let token = value_or_stdin(&msg)?;
            println!("{}", gxt::verify(&token)?);
        }
        Cmd::DecryptFile { key, msg } => {
            let token = fs::read_to_string(msg)?;
            let sk = fs::read_to_string(key)?;
            println!("{}", gxt::decrypt_message(&token, &sk)?);
        }
        Cmd::VerifyFile { msg } => {
            let token = fs::read_to_string(msg)?;
            println!("{}", gxt::verify(&token)?);
        }
    }

    Ok(())
}

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

fn value_or_stdin(payload: &str) -> Result<String> {
    if payload == "-" {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        Ok(payload.to_string())
    }
}
