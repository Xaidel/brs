//! Crockford Base32 alphabet and symbol helpers shared by `MachineHardwareId`
//! and `RecoveryCode` (ADR-0005 §1, §3).

/// The Crockford Base32 alphabet (excludes `I`, `L`, `O`, `U`).
pub(crate) const CROCKFORD_ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// The 37-symbol Crockford check-symbol alphabet: the Base32 alphabet plus
/// `*~$=U`, used for the Recovery Code's trailing mod-37 check symbol.
pub(crate) const CROCKFORD_CHECK_SYMBOLS: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ*~$=U";

/// Whether `c` is a Crockford Base32 data symbol.
pub(crate) fn is_crockford_char(c: char) -> bool {
    CROCKFORD_ALPHABET.contains(c)
}

/// The 0-31 value of a Crockford Base32 data symbol.
///
/// # Panics
///
/// Panics if `c` is not a Crockford Base32 symbol; callers must validate with
/// [`is_crockford_char`] first.
pub(crate) fn crockford_value(c: char) -> u32 {
    CROCKFORD_ALPHABET
        .find(c)
        .expect("caller must validate the symbol is in the Crockford alphabet") as u32
}
