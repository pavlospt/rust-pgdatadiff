#[cfg(test)]
pub(crate) fn sanitize_raw_string(raw: impl Into<String>) -> String {
    raw.into()
        .split_ascii_whitespace()
        .map(|e| e.to_string())
        .reduce(|acc, s| format!("{acc} {s}"))
        .unwrap()
}
