// TODO: remove this and add docs
#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

const E_RUST_TO_C_STRING: &str = "Could not convert rust string to C string";
const E_C_TO_RUST_STRING: &str = "Could not convert C string to rust string";

#[unsafe(no_mangle)]
pub extern "C" fn gxt_make_key() -> *mut c_char {
    let cstr = CString::new(gxt::make_key()).expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_verify(msg: *const c_char) -> *mut c_char {
    let msg = unsafe { CStr::from_ptr(msg) };
    let rec =
        gxt::verify(msg.to_str().expect(E_C_TO_RUST_STRING)).expect("Failed to verify message");
    let cstr = CString::new(serde_json::to_string(&rec).expect("Could not serialize output"))
        .expect(E_RUST_TO_C_STRING);
    cstr.into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_encrypt(
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_encrypt_with_parent(
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_decrypt(msg: *const c_char, key: *const c_char) -> *mut c_char {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gxt_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
