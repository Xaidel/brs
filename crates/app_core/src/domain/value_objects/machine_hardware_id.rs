//! The Machine Hardware ID bound into a License Key and persisted on a
//! `LicenseGrant` (`tdd.phase-1` §6.1, ADR-0005 §1).

use std::fmt;

use super::crockford::is_crockford_char;
use crate::domain::errors::MachineHardwareIdError;

/// Canonical form: 52 Crockford Base32 data characters, grouped into 13 dashed
/// 4-character blocks (e.g. `4ZQK-…-WXYZ`). Derivation (the SHA-256 + encoding
/// step) is entirely `infra_hardware_id`'s job; this value object only validates
/// and formats the resulting string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MachineHardwareId {
    canonical: String,
}

impl MachineHardwareId {
    /// Number of dashed groups in the canonical display form.
    pub const GROUPS: usize = 13;
    /// Characters per dashed group.
    pub const GROUP_LEN: usize = 4;
    /// Total data characters (groups × group length).
    pub const DATA_CHARS: usize = Self::GROUPS * Self::GROUP_LEN;

    /// Parses and canonically formats a Machine Hardware ID string.
    ///
    /// Accepts the grouped form with or without hyphens and any letter casing;
    /// stores the canonical uppercase grouped form.
    pub fn parse(input: &str) -> Result<Self, MachineHardwareIdError> {
        let cleaned: String = input
            .trim()
            .chars()
            .filter(|c| *c != '-')
            .map(|c| c.to_ascii_uppercase())
            .collect();
        if cleaned.len() != Self::DATA_CHARS || !cleaned.chars().all(is_crockford_char) {
            return Err(MachineHardwareIdError::InvalidFormat);
        }
        let chars: Vec<char> = cleaned.chars().collect();
        let groups: Vec<String> = chars
            .chunks(Self::GROUP_LEN)
            .map(|group| group.iter().collect())
            .collect();
        Ok(Self {
            canonical: groups.join("-"),
        })
    }

    /// The canonical grouped representation (`XXXX-XXXX-…-XXXX`).
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}

impl fmt::Display for MachineHardwareId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical)
    }
}

impl std::str::FromStr for MachineHardwareId {
    type Err = MachineHardwareIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::test_support::grouped;

    #[test]
    fn parses_grouped_form() {
        let input = grouped(13);
        let id = MachineHardwareId::parse(&input).unwrap();
        assert_eq!(id.as_str(), input);
    }

    #[test]
    fn normalizes_casing_and_hyphens() {
        let input = grouped(13);
        let ungrouped = input.replace('-', "");
        let mixed = ungrouped.to_lowercase();
        let id = MachineHardwareId::parse(&mixed).unwrap();
        assert_eq!(id.as_str(), input);
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            MachineHardwareId::parse(&grouped(12)),
            Err(MachineHardwareIdError::InvalidFormat)
        );
        assert_eq!(
            MachineHardwareId::parse(&grouped(14)),
            Err(MachineHardwareIdError::InvalidFormat)
        );
        assert_eq!(
            MachineHardwareId::parse(""),
            Err(MachineHardwareIdError::InvalidFormat)
        );
    }

    #[test]
    fn rejects_non_crockford_symbols() {
        let mut input = grouped(13);
        input.replace_range(..1, "O");
        assert_eq!(
            MachineHardwareId::parse(&input),
            Err(MachineHardwareIdError::InvalidFormat)
        );
    }

    #[test]
    fn display_matches_canonical() {
        let input = grouped(13);
        let id = MachineHardwareId::parse(&input).unwrap();
        assert_eq!(id.to_string(), input);
    }
}
