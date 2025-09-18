use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

#[cfg(feature = "ui")]
mod ui;

#[derive(Parser)]
#[command(name = "gxt", version, about = "GXT (Game Exchange Token)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Parser)]
#[group(required = true, multiple = false)]
pub struct MsgInput {
    /// The string token containing the message. Pass - to read from stdin
    #[arg(short, long)]
    msg: Option<String>,

    /// The path to the encrypted message
    #[arg(short, long)]
    file: Option<PathBuf>,
}

#[derive(Clone, ValueEnum)]
enum TimelockType {
    Public,
    Private,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generates a new private key
    Keygen {
        /// Where to store the key
        #[arg(short, long)]
        out: PathBuf,
    },

    /// Generate an ID card containing the data about a peer
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

    /// Verify a message
    Verify {
        #[clap(flatten)]
        msg: MsgInput,

        /// Print output as json
        #[arg(short, long)]
        json: bool,
    },

    /// Create an encrypted message
    Msg {
        /// The key of the sender
        #[arg(short, long)]
        key: PathBuf,

        /// The id card of the recipient
        #[arg(short, long)]
        to: PathBuf,

        /// The parent of this message
        #[arg(long)]
        parent: Option<String>,

        /// The payload of the message. Can be anything, but must be set. Pass - to read from stdin
        #[arg(short, long)]
        payload: String,

        /// Where to store the message token
        #[arg(short, long)]
        out: Option<PathBuf>,
    },

    /// Decrypt a message
    Decrypt {
        /// The key of the receiver
        #[arg(short, long)]
        key: PathBuf,

        #[clap(flatten)]
        msg: MsgInput,

        /// Print output as json
        #[arg(short, long)]
        json: bool,
    },

    #[cfg(feature = "ui")]
    /// Show a simple UI for opening messages
    Ui {
        /// The message to decode
        path: Option<PathBuf>,
        /// The key, if the message is encrypted
        key: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Keygen { out } => {
            let signing_key = gxt::make_key();
            write_out_string(&signing_key, Some(out.as_ref()))?;
        }

        Cmd::Id { out, key, meta } => {
            let signing_key = fs::read_to_string(key)?;
            let meta_json = value_or_stdin(&meta)?;
            let meta: serde_json::Value = serde_json::from_str(meta_json.trim())?;
            let id_card = gxt::make_id_card(&signing_key, meta)?;
            write_out_string(&id_card, out.as_deref())?;
        }

        Cmd::Verify { msg, json } => {
            let token = match (msg.msg, msg.file) {
                (Some(msg), None) => value_or_stdin(&msg)?,
                (None, Some(file)) => fs::read_to_string(file)?,
                _ => anyhow::bail!("Nothing to verify"),
            };
            let envelope = gxt::verify_message::<serde_json::Value>(&token)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("{envelope}");
            }
        }

        Cmd::Msg {
            key,
            to,
            parent,
            payload,
            out,
        } => {
            let signing_key = fs::read_to_string(key)?;
            let id_card = fs::read_to_string(to)?;
            let payload_json = value_or_stdin(&payload)?;
            let payload: serde_json::Value = serde_json::from_str(payload_json.trim())?;
            let encrypted_message = gxt::encrypt_message(&signing_key, &id_card, &payload, parent)?;
            write_out_string(&encrypted_message, out.as_deref())?;
        }

        Cmd::Decrypt { key, msg, json } => {
            let encrypted_message = match (msg.msg, msg.file) {
                (Some(msg), None) => value_or_stdin(&msg)?,
                (None, Some(file)) => fs::read_to_string(file)?,
                _ => anyhow::bail!("Nothing to verify"),
            };
            let signing_key = fs::read_to_string(key)?;
            let envelope =
                gxt::decrypt_message::<serde_json::Value>(&encrypted_message, &signing_key)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("{envelope}");
            }
        }

        #[cfg(feature = "ui")]
        Cmd::Ui { path, key } => ui::run(path, key)?,
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
