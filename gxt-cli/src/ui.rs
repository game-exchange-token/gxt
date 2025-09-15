use std::path::PathBuf;

use gxt::PayloadKind;

use slint::ToSharedString;

slint::include_modules!();

#[cfg(windows)]
fn hide_console() {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::System::Console::GetConsoleWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;
    use windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow;

    unsafe {
        let window: HWND = GetConsoleWindow();
        if window != std::ptr::null_mut() {
            ShowWindow(window, SW_HIDE);
        }
    }
}

impl From<gxt::Envelope<serde_json::Value>> for UiEnvelope {
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
        UiEnvelope {
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

pub fn run(path: Option<PathBuf>, key: Option<PathBuf>) -> anyhow::Result<()> {
    hide_console();
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

        let ui_envelope = UiEnvelope {
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
            if let Some(file) = rfd::FileDialog::new().pick_file() {
                let ui = ui_handle.unwrap();

                let text = std::fs::read_to_string(file).unwrap();
                let envelope = gxt::verify_message::<serde_json::Value>(&text).unwrap();
                ui.set_token_text(text.into());
                ui.set_can_decrypt(envelope.kind == PayloadKind::Msg);
                ui.set_envelope(envelope.into());
            }
        }
    });

    ui.on_request_decrypt({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(file) = rfd::FileDialog::new().pick_file() {
                let ui = ui_handle.unwrap();

                let key = std::fs::read_to_string(file).unwrap();
                let envelope =
                    gxt::decrypt_message::<serde_json::Value>(&ui.get_token_text(), &key).unwrap();
                ui.set_envelope(envelope.into());
                ui.set_can_decrypt(false);
            }
        }
    });

    ui.run()?;

    Ok(())
}
