//! Patrones comunes para identificar caracteres en Wollok

#[must_use]
/// Verifica si un caracter puede ser el inicio de un identificador
pub fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

#[must_use]
/// Verifica si un caracter puede formar parte de un identificador
pub fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[must_use]
/// Verifica si un caracter es whitespace (sin incluir newlines)
pub fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

#[must_use]
/// Verifica si un caracter es newline
pub fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

#[must_use]
/// Verifica si un caracter puede ser parte de un número
pub fn is_numeric(c: char) -> bool {
    c.is_ascii_digit()
}

#[must_use]
/// Verifica si un caracter puede ser parte de un operador
pub fn is_operator_char(c: char) -> bool {
    matches!(
        c,
        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|'
    )
}

#[must_use]
/// Verifica si un caracter es puntuación
pub fn is_punctuation(c: char) -> bool {
    matches!(
        c,
        ',' | ';' | ':' | '.' | '$' | '(' | ')' | '{' | '}' | '[' | ']'
    )
}
