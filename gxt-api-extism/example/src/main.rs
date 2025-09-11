use std::path::PathBuf;

use clap::Parser;
use extism::*;
use gxt_extism_types::{calls::*, json};

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Cli { path } = Cli::parse();
    let path = Wasm::file(path);
    let manifest = Manifest::new([path]);
    let mut plugin = Plugin::new(&manifest, [], true)?;
    let alice = plugin.call::<MAKE_KEY_IN, MAKE_KEY_OUT>(MAKE_KEY, ())?;
    let bob = plugin.call::<MAKE_KEY_IN, MAKE_KEY_OUT>(MAKE_KEY, ())?;
    println!("Alice Key: {alice}");
    println!("Bob Key: {bob}");

    let bobs_id_card = plugin.call::<MAKE_ID_IN, MAKE_ID_OUT>(
        MAKE_ID_CARD,
        MAKE_ID_IN {
            key: bob.to_string(),
            meta: json!({"name": "Bob"}),
        },
    )?;
    println!("Bobs ID Card Token: {bobs_id_card}");

    let bobs_id_envelope = plugin
        .call::<VERIFY_MESSAGE_IN, VERIFY_MESSAGE_OUT>(VERIFY_MESSAGE, bobs_id_card.clone())?;
    println!("Bobs ID Card: {bobs_id_envelope:#?}");

    let encrypted_msg_for_bob = plugin.call::<ENCRYPT_MESSAGE_IN, ENCRYPT_MESSAGE_OUT>(
        ENCRYPT_MESSAGE,
        ENCRYPT_MESSAGE_IN {
            key: alice,
            id_card: bobs_id_card,
            payload: json!({"trade": {"type": "sword", "amount": 5}}),
            parent: None,
        },
    )?;
    println!("Encrypted Message Token for Bob: {encrypted_msg_for_bob}");

    let encrypted_msg_for_bob_envelope = plugin.call::<VERIFY_MESSAGE_IN, VERIFY_MESSAGE_OUT>(
        VERIFY_MESSAGE,
        encrypted_msg_for_bob.clone(),
    )?;
    println!("Encrypted Message for Bob: {encrypted_msg_for_bob_envelope:#?}");

    let decrypted_msg_for_bob = plugin.call::<DECRYPT_MESSAGE_IN, DECRYPT_MESSAGE_OUT>(
        DECRYPT_MESSAGE,
        DECRYPT_MESSAGE_IN {
            message: encrypted_msg_for_bob,
            key: bob,
        },
    )?;
    println!("Decrypted Message for Bob: {decrypted_msg_for_bob:#?}");

    Ok(())
}
