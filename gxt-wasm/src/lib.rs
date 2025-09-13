use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn make_key() -> String {
    gxt::make_key()
}

#[wasm_bindgen]
pub fn make_id_card(key: &str, meta: JsValue) -> Result<String, JsValue> {
    let meta: serde_json::Value = serde_wasm_bindgen::from_value(meta)?;
    Ok(gxt::make_id_card(key, meta).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn verify_message(msg: &str) -> Result<JsValue, JsValue> {
    let envelope = gxt::verify_message::<serde_json::Value>(msg).map_err(|e| e.to_string())?;
    Ok(serde_wasm_bindgen::to_value(&envelope)?)
}

#[wasm_bindgen]
pub fn encrypt_message(
    key: &str,
    id_card: &str,
    payload: JsValue,
    parent: Option<String>,
) -> Result<String, JsValue> {
    let payload: serde_json::Value = serde_wasm_bindgen::from_value(payload)?;
    Ok(gxt::encrypt_message(key, id_card, &payload, parent).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn decrypt_message(message: &str, key: &str) -> Result<JsValue, JsValue> {
    let envelope =
        gxt::decrypt_message::<serde_json::Value>(message, key).map_err(|e| e.to_string())?;
    Ok(serde_wasm_bindgen::to_value(&envelope)?)
}
