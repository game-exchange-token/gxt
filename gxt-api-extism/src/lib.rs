#![allow(clippy::unnecessary_wraps)]

use extism_pdk::{FnResult, plugin_fn};

#[plugin_fn]
pub fn make_key() -> FnResult<String> {
    Ok(gxt::make_key())
}
