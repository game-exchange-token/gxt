#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    path::PathBuf,
    process::{Command as StdCommand, exit},
};

use clap::Parser;
use gxt::PayloadKind;
use slint::ToSharedString;

use registry::{Data, Hive, Security};

#[cfg(target_os = "windows")]
use elevated_command::Command;

#[cfg(target_os = "windows")]
use utfx::U16CString;

#[derive(Parser)]
struct Cli {
    path: Option<PathBuf>,
    key: Option<PathBuf>,

    #[arg(long)]
    register: bool,
}

slint::include_modules!();

impl From<gxt::Envelope<serde_json::Value>> for Envelope {
    fn from(value: gxt::Envelope<serde_json::Value>) -> Self {
        let gxt::Envelope {
            version,
            verification_key,
            encryption_key,
            kind,
            payload,
            parent,
            id,
            signature,
        } = value;
        Envelope {
            version: version.into(),
            encryption_key: encryption_key.into(),
            id: id.into(),
            kind: kind.to_shared_string(),
            parent: parent.unwrap_or_default().into(),
            payload: serde_json::to_string_pretty(&payload).unwrap().into(),
            signature: signature.into(),
            verification_key: verification_key.into(),
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn register_protocol_handler() -> anyhow::Result<()> {
    bail!("Automatic handler registration is only supported on Windows");
}

#[cfg(target_os = "windows")]
fn register_protocol_handler() -> anyhow::Result<()> {
    if !Command::is_elevated() {
        let cmd = StdCommand::new(std::env::current_exe()?);
        let elevated_cmd = Command::new(cmd);
        _ = elevated_cmd.output().unwrap();
        return Ok(());
    }

    let hive = Hive::ClassesRoot.create("gxt", Security::Write)?;
    hive.set_value("URL Protocol", &Data::String(U16CString::from_str("")?))?;
    let hive = Hive::ClassesRoot.create(r"gxt\shell\open\command", Security::Write)?;
    let command = format!("{} \"%1\"", std::env::current_exe()?.to_string_lossy());
    hive.set_value("", &Data::String(U16CString::from_str(command)?))?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let Cli {
        path,
        key,
        register,
    } = Cli::parse();
    if register {
        register_protocol_handler()?;
        exit(0);
    }

    let ui = AppWindow::new()?;

    ui.set_can_decrypt(false);

    if let Some(path) = path {
        let text = std::fs::read_to_string(path)?;
        let gxt::Envelope {
            version,
            verification_key,
            encryption_key,
            kind,
            payload,
            parent,
            id,
            signature,
        } = if let Some(key) = key {
            let key = std::fs::read_to_string(key)?;
            gxt::decrypt_message::<serde_json::Value>(&text, &key)?
        } else {
            let envelope = gxt::verify_message::<serde_json::Value>(&text)?;
            ui.set_can_decrypt(envelope.kind == PayloadKind::Msg);
            envelope
        };
        ui.set_token_text(text.into());

        let ui_envelope = Envelope {
            version: version.into(),
            encryption_key: encryption_key.into(),
            id: id.into(),
            kind: kind.to_shared_string(),
            parent: parent.unwrap_or_default().into(),
            payload: serde_json::to_string_pretty(&payload)?.into(),
            signature: signature.into(),
            verification_key: verification_key.into(),
        };

        ui.set_envelope(ui_envelope);
    }

    ui.on_request_load({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let file = rfd::FileDialog::new().pick_file().unwrap();
            let text = std::fs::read_to_string(file).unwrap();
            let envelope = gxt::verify_message::<serde_json::Value>(&text).unwrap();
            ui.set_token_text(text.into());
            ui.set_can_decrypt(envelope.kind == PayloadKind::Msg);
            ui.set_envelope(envelope.into());
        }
    });

    ui.on_request_decrypt({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let file = rfd::FileDialog::new().pick_file().unwrap();
            let key = std::fs::read_to_string(file).unwrap();
            let envelope =
                gxt::decrypt_message::<serde_json::Value>(&ui.get_token_text(), &key).unwrap();
            ui.set_envelope(envelope.into());
            ui.set_can_decrypt(false);
        }
    });

    ui.run()?;

    Ok(())
}
