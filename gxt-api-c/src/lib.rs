use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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
