//! The Recovery Code: a one-time, human-transcribable code (28 Crockford Base32
//! data characters + 1 mod-37 check symbol), `XXXX-…-X` groupable
//! (`tdd.phase-1` §6.1, ADR-0005 §3). Debug output is redacted; display is
//! explicit via [`RecoveryCode::formatted`].

use std::fmt;

use super::crockford::{CROCKFORD_CHECK_SYMBOLS, crockford_value, is_crockford_char};
use crate::domain::errors::RecoveryCodeError;

/// A one-time, human-transcribable recovery code: 28 Crockford Base32 data
/// characters plus one mod-37 check symbol, displayed grouped as
/// `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-X` (ADR-0005 §3). It wraps
/// `system_secret` (AES-256-GCM) rather than transcribing it directly.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RecoveryCode {
    canonical: String,
}

impl RecoveryCode {
    /// Data characters per Recovery Code.
    pub const DATA_LEN: usize = 28;
    /// Total characters including the check symbol.
    pub const TOTAL_LEN: usize = Self::DATA_LEN + 1;

    /// Parses a Recovery Code entered by the Secretary.
    ///
    /// Normalizes input (uppercases, strips hyphens, applies Crockford's
    /// lookalike substitution `O→0`, `I`/`L`→`1`) and validates the mod-37
    /// check symbol before returning `Ok`. A wrong length, a symbol outside
    /// the Crockford data alphabet, or a check symbol outside the 37-symbol
    /// check alphabet is `MalformedFormat`; a well-formed code with a
    /// mismatched check symbol is `InvalidChecksum` and fails fast — well
    /// before any AES-GCM unwrap attempt in `infra_credentials`.
    pub fn parse(input: &str) -> Result<Self, RecoveryCodeError> {
        let normalized: String = input
            .trim()
            .chars()
            .filter(|c| *c != '-')
            .map(|c| match c.to_ascii_uppercase() {
                'O' => '0',
                'I' | 'L' => '1',
                c => c,
            })
            .collect();
        // Iterate by symbol, not by byte, so a multi-byte UTF-8 char can never
        // be sliced mid-symbol: it simply fails the length or alphabet checks.
        let chars: Vec<char> = normalized.chars().collect();
        if chars.len() != Self::TOTAL_LEN {
            return Err(RecoveryCodeError::MalformedFormat);
        }
        let (data, check) = chars.split_at(Self::DATA_LEN);
        if !data.iter().all(|c| is_crockford_char(*c)) {
            return Err(RecoveryCodeError::MalformedFormat);
        }
        if !CROCKFORD_CHECK_SYMBOLS.contains(check[0]) {
            return Err(RecoveryCodeError::MalformedFormat);
        }
        let data_sum: u32 = data.iter().map(|c| crockford_value(*c)).sum();
        let expected_check = CROCKFORD_CHECK_SYMBOLS
            .chars()
            .nth((data_sum % 37) as usize)
            .expect("sum mod 37 always indexes the 37-symbol check alphabet");
        if check[0] != expected_check {
            return Err(RecoveryCodeError::InvalidChecksum);
        }
        Ok(Self {
            canonical: normalized,
        })
    }

    /// The canonical normalized code: 28 uppercase data characters plus the
    /// check symbol, no hyphens.
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// The canonical 28-character data string (no check symbol), used by
    /// `infra_credentials` as the SHA-256 wrap-key input (`tdd.phase-1` §8.4.1).
    pub fn canonical_data(&self) -> &str {
        &self.canonical[..Self::DATA_LEN]
    }

    /// The grouped display form `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-X`.
    pub fn formatted(&self) -> String {
        let groups: Vec<&str> = (0..7)
            .map(|i| &self.canonical[i * 4..i * 4 + 4])
            .chain(std::iter::once(&self.canonical[Self::DATA_LEN..]))
            .collect();
        groups.join("-")
    }
}

impl fmt::Debug for RecoveryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryCode([redacted])")
    }
}

impl std::str::FromStr for RecoveryCode {
    type Err = RecoveryCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_code(data: &str, check: &str) -> String {
        format!("{data}{check}")
    }

    // Sum of 28 '0' symbols is 0 → check symbol index 0 = '0'.
    const ALL_ZEROS: &str = "0000000000000000000000000000";

    #[test]
    fn accepts_valid_code() {
        let code = canonical_code(ALL_ZEROS, "0");
        let parsed = RecoveryCode::parse(&code).unwrap();
        assert_eq!(parsed.as_str(), code);
    }

    #[test]
    fn accepts_uppercase_and_hyphenated_form() {
        let base = "0000-0000-0000-0000-0000-0000-0000-0";
        let parsed = RecoveryCode::parse(base).unwrap();
        assert_eq!(parsed.as_str(), canonical_code(ALL_ZEROS, "0"));
        assert_eq!(parsed.formatted(), base);
    }

    #[test]
    fn normalizes_lookalike_characters() {
        // 'O'→0 substitution: 28 'O's normalize to 28 zeros (sum 0, check '0').
        let with_o = canonical_code("O".repeat(28).as_str(), "0");
        assert!(RecoveryCode::parse(&with_o).is_ok());

        // Data "00…001" (27 zeros + one 1) sums to 1 → check symbol index 1 = '1'.
        let data = "0000000000000000000000000001";
        assert_eq!(
            RecoveryCode::parse(&canonical_code(data, "1")),
            Ok(RecoveryCode {
                canonical: canonical_code(data, "1")
            })
        );

        // 'I' maps to '1' in the data: "00…00I" normalizes to "00…001",
        // whose checksum is '1' (the 'I' in the check position also → '1').
        let data_with_i = format!("{}{}", "0".repeat(27), "I");
        assert_eq!(
            RecoveryCode::parse(&canonical_code(&data_with_i, "I")),
            RecoveryCode::parse(&canonical_code(data, "1"))
        );

        // 'L' maps to '1' identically.
        let data_with_l = format!("{}{}", "0".repeat(27), "L");
        assert_eq!(
            RecoveryCode::parse(&canonical_code(&data_with_l, "L")),
            RecoveryCode::parse(&canonical_code(data, "1"))
        );
    }

    #[test]
    fn rejects_bad_checksum() {
        // Correct checksum for 28 zeros is '0', not '1'.
        let code = canonical_code(ALL_ZEROS, "1");
        assert_eq!(
            RecoveryCode::parse(&code),
            Err(RecoveryCodeError::InvalidChecksum)
        );
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            RecoveryCode::parse("1234-5678"),
            Err(RecoveryCodeError::MalformedFormat)
        );
        // 28 data chars without a check symbol.
        assert_eq!(
            RecoveryCode::parse(ALL_ZEROS),
            Err(RecoveryCodeError::MalformedFormat)
        );
    }

    #[test]
    fn rejects_unknown_symbol_in_data() {
        // 'U' is not a Crockford data symbol; the malformed data char fails
        // before any checksum computation.
        let mut data = "0".repeat(28);
        data.replace_range(0..1, "U");
        let code = canonical_code(&data, "0");
        assert_eq!(
            RecoveryCode::parse(&code),
            Err(RecoveryCodeError::MalformedFormat)
        );
    }

    #[test]
    fn valid_alphabet_wrong_check_symbol_is_checksum_error() {
        // '~' IS a Crockford check symbol, but it is not the mod-37 check
        // symbol for 28 zeros (which is '0'), so the code is well-formed with
        // a mismatched check symbol: `InvalidChecksum`, not `MalformedFormat`.
        let code = canonical_code(ALL_ZEROS, "~");
        assert_eq!(
            RecoveryCode::parse(&code),
            Err(RecoveryCodeError::InvalidChecksum)
        );
    }

    #[test]
    fn rejects_non_ascii_input_as_malformed() {
        // 'ü' is a 2-byte UTF-8 char; parsing must never slice mid-symbol.
        let mut code = canonical_code(ALL_ZEROS, "0");
        code.replace_range(code.len() - 1.., "ü");
        assert_eq!(
            RecoveryCode::parse(&code),
            Err(RecoveryCodeError::MalformedFormat)
        );
    }

    #[test]
    fn rejects_out_of_alphabet_check_symbol() {
        // '!' is not a Crockford check symbol (the 37-symbol alphabet); the
        // code is malformed, not a well-formed code with a bad checksum.
        let mut code = canonical_code(ALL_ZEROS, "0");
        code.replace_range(code.len() - 1.., "!");
        assert_eq!(
            RecoveryCode::parse(&code),
            Err(RecoveryCodeError::MalformedFormat)
        );
    }

    #[test]
    fn canonical_data_excludes_check_symbol() {
        let code = canonical_code(ALL_ZEROS, "0");
        let parsed = RecoveryCode::parse(&code).unwrap();
        assert_eq!(parsed.canonical_data(), ALL_ZEROS);
    }
}
