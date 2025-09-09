use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
fn make_key() -> PyResult<String> {
    Ok(gxt::make_key())
}

#[pyfunction]
fn make_id_card(key: &str, meta: &str) -> PyResult<String> {
    gxt::make_id_card(key, meta).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn verify_message(msg: &str) -> PyResult<String> {
    match gxt::verify_message(msg) {
        Ok(rec) => serde_json::to_string(&rec).map_err(|e| PyValueError::new_err(e.to_string())),
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
fn encrypt_message(
    key: &str,
    id_card: &str,
    body: &str,
    parent: Option<String>,
) -> PyResult<String> {
    gxt::encrypt_message(key, id_card, body, parent)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn decrypt_message(msg: &str, key: &str) -> PyResult<String> {
    match gxt::decrypt_message(msg, key) {
        Ok(rec) => serde_json::to_string(&rec).map_err(|e| PyValueError::new_err(e.to_string())),
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pymodule]
fn gxt_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(make_key, m)?)?;
    m.add_function(wrap_pyfunction!(make_id_card, m)?)?;
    m.add_function(wrap_pyfunction!(verify_message, m)?)?;
    m.add_function(wrap_pyfunction!(encrypt_message, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt_message, m)?)?;
    Ok(())
}
