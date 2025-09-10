#![doc = include_str!("../../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

// using doc = include_str broke my syntax highlighting, so all the code is now in the internal module
mod internal;

pub use internal::{
    Envelope, GxtError, PayloadKind, decrypt_message, encrypt_message, make_id_card, make_key,
    verify_message,
};
