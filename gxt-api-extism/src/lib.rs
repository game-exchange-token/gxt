use extism_pdk::*;

use getrandom::Error;

#[host_fn]
extern "ExtismHost" {
    fn get_random_bytes(len: u64) -> Vec<u8>;
}

#[no_mangle]
unsafe extern "Rust" fn __getrandom_custom(dest: *mut u8, len: usize) -> Result<(), Error> {
    let buf = unsafe {
        // fill the buffer with zeros
        core::ptr::write_bytes(dest, 0, len);
        // create mutable byte slice
        core::slice::from_raw_parts_mut(dest, len)
    };
    let bytes = get_random_bytes(len as u64).unwrap();
    buf[..len].copy_from_slice(&bytes[..len]);

    Ok(())
}

#[plugin_fn]
pub fn make_key() -> FnResult<String> {
    Ok(gxt::make_key())
}
