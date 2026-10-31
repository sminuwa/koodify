//! Helpers for building optional query-string parameters.

/// Collect `(key, value)` pairs, skipping `None` values.
pub fn optional_query(pairs: &[(&str, Option<String>)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|val| ((*k).to_string(), val.clone())))
        .collect()
}
