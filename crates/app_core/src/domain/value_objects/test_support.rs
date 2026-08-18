//! Test-only helpers shared across domain test modules (`#[cfg(test)]`).

/// The Crockford Base32 alphabet used to build sample grouped strings.
const ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Builds the dashed-grouped Crockford string of `n` 4-character groups,
/// e.g. `grouped(13)` yields a 13-group Machine Hardware ID.
pub(crate) fn grouped(n: usize) -> String {
    let chars: String = (0..n * 4)
        .map(|i| ALPHABET.as_bytes()[i % 32] as char)
        .collect();
    chars
        .as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join("-")
}
