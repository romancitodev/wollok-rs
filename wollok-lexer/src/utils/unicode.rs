#[must_use]
/// Normaliza un string Unicode para comparaciones
pub fn normalize_identifier(s: &str) -> String {
    // Por ahora solo convertimos a lowercase, pero podríamos usar
    // normalización Unicode más sofisticada aquí
    s.to_lowercase()
}

#[must_use]
/// Verifica si dos identificadores son equivalentes considerando Unicode
pub fn identifiers_equal(a: &str, b: &str) -> bool {
    normalize_identifier(a) == normalize_identifier(b)
}
