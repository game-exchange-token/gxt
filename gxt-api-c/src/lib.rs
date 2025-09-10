use std::ffi::{CStr, CString};
use std::os::raw::c_char;

//! # GXT (Game Exchange Token)
//!
//! Minimal, encrypted, signed and copy-pasteable tokens for manual data exchange between games.
//!
//! For details check out [`spec.md`](https://github.com/hardliner66/gxt/blob/main/spec.md).
//!
//! - [Rationale](#rationale)
//! - [About](#about)
//! - [C API](#c-api)
//! - [Install](#install)
//! - [Demo](#demo)
//! - [CLI](#cli)
//!   - [General](#general)
//!   - [Keygen](#keygen)
//!   - [Id](#id)
//!   - [Verify](#verify)
//!   - [Msg](#msg)
//!   - [Decrypt](#decrypt)
//!
//! ## Rationale
//! I was thinking about how it could be possible to add trading
//! between two players to a singleplayer game as part of a mod. Mostly out of curiosity to see
//! if it was doable or too much work. At first I thought about having a server that manages
//! the trades, but then I thought that not everybody can or wants to set up a server.
//!
//! Thats also when I had the idea to package the data into string tokens that can be sent
//! via discord and started researching how to make this somewhat secure and
//! easy to use and implement.
//!
//! With the current design, every message is signed and encrypted for a designated receiver.
//! This prevents people from fulfilling a trade request and then sending the fulfillment to
//! 50 people who all collect the rewards. Its still not as secure as server side validation,
//! but thats okay for me.
//!
//! While working on this, I also realized that there is potential for more than just trading,
//! so I removed all the trade specific fields and the protocol now takes an opaque payload
//! that can contain any valid json value. (Strings, Numbers, Maps, etc.)
//!
//! ## About
//! The protocol uses an Ed25519 key pair for signing messages and to derive a X25519 key pair
//! from encryption.
//!
//! The size of the token before encoding is limited to 64KB.
//!
//! Because this is intended to be easy to integrate by mod authors, a library and cli are provided.
//! Both are written in rust. There is also a wrapper that exposes a C API called `gxt-api-c`.
//! The library can also compile to wasm, making it possible to use this in a web context.
//!
//! ## C API
//! To use the C API, clone the repository and then build the crate `gxt-api-c`.
//! This will create a dynamic and a static library, as well as the corresponding include header,
//! inside the target directory.
//!
//! ```bash
//! git clone https://github.com/hardliner66/gxt
//! cd gxt
//! cargo build -p gxt-api-c --release
//! cd target/release
//! ls
//! ```
//!
//! ## Install
//! ```bash
//! cargo install gxt-cli
//! ```
//!
//! ## Demo
//! ```bash
//! # Create keys for communication
//! gxt keygen --out alice.key
//! gxt keygen --out bob.key
//!
//! # Create an id card for bob
//! echo '{"name":"Bob"}' | gxt id bob.key --out bob.id --meta -
//!
//! # Create a message for bob using their id card and your own key
//! gxt msg --key alice.key --to bob.id --out msg_to_bob.gxt --body '{"hello":"world"}'
//!
//! # Verify if the message is valid and signed
//! gxt verify --file msg_to_bob.gxt
//!
//! # Decrypt the message using bobs key
//! gxt decrypt --key bob.key --file msg_to_bob.gxt
//!
//! # Try decrypting a message with a key its not intended for
//! gxt keygen --out charlie.key
//! gxt decrypt --key charlie.key --file msg_to_bob.gxt
//! ```
//!
//! ## CLI
//! ### General
//! ```
//! GXT (Game Exchange Token)
//!
//! Usage: gxt <COMMAND>
//!
//! Commands:
//!   keygen   Generates a new private key
//!   id       Generate an ID card containing the data about a peer
//!   verify   Verify a message
//!   msg      Create an encrypted message
//!   decrypt  Decrypt a message
//!   help     Print this message or the help of the given subcommand(s)
//!
//! Options:
//!   -h, --help     Print help
//!   -V, --version  Print version
//! ```
//!
//! ### Keygen
//! ```
//! Generates a new private key
//!
//! Usage: gxt keygen --out <OUT>
//!
//! Options:
//!   -o, --out <OUT>  Where to store the key
//!   -h, --help       Print help
//! ```
//!
//! ### Id
//! ```
//! Generate an ID card containing the data about a peer
//!
//! Usage: gxt id [OPTIONS] --meta <META> <KEY>
//!
//! Arguments:
//!   <KEY>  The key of the person creating the id card
//!
//! Options:
//!   -m, --meta <META>  Meta data for the id card. Can be anything, but must be set. Pass - to read from stdin
//!   -o, --out <OUT>    Where to store the id card token
//!   -h, --help         Print help
//! ```
//!
//! ### Verify
//! ```
//! Verify a message
//!
//! Usage: gxt.exe verify [OPTIONS] <--msg <MSG>|--file <FILE>>
//!
//! Options:
//!   -m, --msg <MSG>    The string token containing the message. Pass - to read from stdin
//!   -f, --file <FILE>  The path to the encrypted message
//!   -j, --json         Print output as json
//!   -h, --help         Print help
//! ```
//!
//! ### Msg
//! ```
//! Create an encrypted message
//!
//! Usage: gxt msg [OPTIONS] --key <KEY> --to <TO> --body <BODY>
//!
//! Options:
//!   -k, --key <KEY>        The key of the sender
//!   -t, --to <TO>          The id card of the recipient
//!   -p, --parent <PARENT>  The parent of this message
//!   -b, --body <BODY>      The body of the message. Can be anything, but must be set. Pass - to read from stdin
//!   -o, --out <OUT>        Where to store the message token
//!   -h, --help             Print help
//! ```
//!
//! ### Decrypt
//! ```
//! Decrypt a message
//!
//! Usage: gxt.exe decrypt [OPTIONS] --key <KEY> <--msg <MSG>|--file <FILE>>
//!
//! Options:
//!   -k, --key <KEY>    The key of the receiver
//!   -m, --msg <MSG>    The string token containing the message. Pass - to read from stdin
//!   -f, --file <FILE>  The path to the encrypted message
//!   -j, --json         Print output as json
//!   -h, --help         Print help
//! ```

const E_RUST_TO_C_STRING: &str = "Could not convert rust string to C string";
const E_C_TO_RUST_STRING: &str = "Could not convert C string to rust string";

/// Creates a new key and returns it as hex string.
///
/// # Safety
/// - Returned string must be freed with [`gxt_free_string`] after use.
/// - Currently panics on error.
#[unsafe(no_mangle)]
pub extern "C" fn gxt_make_key() -> *mut c_char {
    let cstr = CString::new(gxt::make_key()).expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

/// Creates a new id card from a key and returns it as gxt message.
///
/// # Safety
/// - Returned string must be freed with [`gxt_free_string`] after use.
/// - Currently panics on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_make_id_card(key: *const c_char, meta: *const c_char) -> *mut c_char {
    let key = unsafe { CStr::from_ptr(key) };
    let meta = unsafe { CStr::from_ptr(meta) };
    let id = gxt::make_id_card(
        key.to_str().expect(E_C_TO_RUST_STRING),
        meta.to_str().expect(E_C_TO_RUST_STRING),
    )
    .expect("Failed to make identity");
    let cstr = CString::new(id).expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

/// Verifies a message and returns the contents as JSON string on success.
///
/// # Safety
/// - Returned string must be freed with [`gxt_free_string`] after use.
/// - Currently panics on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_verify_message(msg: *const c_char) -> *mut c_char {
    let msg = unsafe { CStr::from_ptr(msg) };
    let rec = gxt::verify_message(msg.to_str().expect(E_C_TO_RUST_STRING))
        .expect("Failed to verify message");
    let cstr = CString::new(serde_json::to_string(&rec).expect("Could not serialize output"))
        .expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

/// Encrypts the payload and returns the gxt message containing the encrypted data.
///
/// # Safety
/// - Returned string must be freed with [`gxt_free_string`] after use.
/// - Currently panics on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_encrypt_message(
    key: *const c_char,
    id_card: *const c_char,
    body: *const c_char,
) -> *mut c_char {
    let key = unsafe { CStr::from_ptr(key) };
    let id_card = unsafe { CStr::from_ptr(id_card) };
    let body = unsafe { CStr::from_ptr(body) };
    let msg = gxt::encrypt_message(
        key.to_str().expect(E_C_TO_RUST_STRING),
        id_card.to_str().expect(E_C_TO_RUST_STRING),
        body.to_str().expect(E_C_TO_RUST_STRING),
        None,
    )
    .expect("Failed to verify message");
    let cstr = CString::new(msg).expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

/// Encrypts the payload and returns the gxt message containing the encrypted data and a parent reference.
///
/// # Safety
/// - Returned string must be freed with [`gxt_free_string`] after use.
/// - Currently panics on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_encrypt_message_with_parent(
    key: *const c_char,
    id_card: *const c_char,
    body: *const c_char,
    parent: *const c_char,
) -> *mut c_char {
    let key = unsafe { CStr::from_ptr(key) };
    let id_card = unsafe { CStr::from_ptr(id_card) };
    let body = unsafe { CStr::from_ptr(body) };
    let parent = unsafe { CStr::from_ptr(parent) };
    let msg = gxt::encrypt_message(
        key.to_str().expect(E_C_TO_RUST_STRING),
        id_card.to_str().expect(E_C_TO_RUST_STRING),
        body.to_str().expect(E_C_TO_RUST_STRING),
        Some(parent.to_str().expect(E_C_TO_RUST_STRING).to_string()),
    )
    .expect("Failed to verify message");
    let cstr = CString::new(msg).expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

/// Verifies and decrypts the payload inside a gxt message and returns it as a json string.
///
/// # Safety
/// - Returned string must be freed with [`gxt_free_string`] after use.
/// - Currently panics on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_decrypt_message(
    msg: *const c_char,
    key: *const c_char,
) -> *mut c_char {
    let msg = unsafe { CStr::from_ptr(msg) };
    let key = unsafe { CStr::from_ptr(key) };
    let rec = gxt::decrypt_message(
        msg.to_str().expect(E_C_TO_RUST_STRING),
        key.to_str().expect(E_C_TO_RUST_STRING),
    )
    .expect("Failed to verify message");
    let cstr = CString::new(serde_json::to_string(&rec).expect("Could not serialize output"))
        .expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

/// This function must be used to free returned strings after they are used.
///
/// # Safety
/// - Only pass strings that have been returned by rust into this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
