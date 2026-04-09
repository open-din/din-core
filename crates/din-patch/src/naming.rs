//! Safe JavaScript identifiers for codegen and UI labels derived from patch metadata.

use std::collections::BTreeSet;

const RESERVED_IDENTIFIERS: &[&str] = &[
    "break",
    "case",
    "catch",
    "children",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "includeProvider",
    "instanceof",
    "key",
    "midi",
    "new",
    "patch",
    "ref",
    "return",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Tokens that cannot be used bare when emitting JS/TS identifiers.
pub fn reserved_identifiers() -> BTreeSet<String> {
    RESERVED_IDENTIFIERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

/// Normalizes arbitrary text to a camelCase-safe identifier, honoring optional extra reserved words.
pub fn to_safe_identifier(
    value: &str,
    fallback: &str,
    reserved: Option<&BTreeSet<String>>,
) -> String {
    let normalized = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .trim()
        .to_string();

    if normalized.is_empty() {
        return fallback.to_string();
    }

    let mut parts = normalized.split_whitespace();
    let Some(first) = parts.next() else {
        return fallback.to_string();
    };

    let mut result = first.to_ascii_lowercase();
    for part in parts {
        let mut chars = part.chars();
        if let Some(head) = chars.next() {
            result.push(head.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }

    let trimmed = result
        .trim_start_matches(|ch: char| !(ch.is_ascii_alphabetic() || ch == '_'))
        .to_string();

    let mut candidate = if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed
    };
    let blocked = reserved
        .map(|entries| entries.contains(&candidate))
        .unwrap_or(false)
        || RESERVED_IDENTIFIERS.contains(&candidate.as_str());

    if blocked {
        candidate = fallback.to_string();
    }

    if candidate
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
        candidate = format!("{fallback}{candidate}");
    }

    candidate
}

/// Appends numeric suffixes (`base2`, `base3`, …) until the name is unused.
pub fn ensure_unique_name(base: &str, used_names: &BTreeSet<String>) -> String {
    if !used_names.contains(base) {
        return base.to_string();
    }

    let mut counter = 2usize;
    loop {
        let candidate = format!("{base}{counter}");
        if !used_names.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}
