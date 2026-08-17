//! A Feature Flag: one per add-on module, unlocked by a License Key (ADR-0003).

use std::fmt;

/// Core Engine has no flag — it is unconditionally unlocked once installed, so
/// no `Core` variant exists. Serializes to/from the SCREAMING_SNAKE_CASE strings
/// `KP_BLOTTER`, `TREASURY`, `BUSINESS_PERMITS` (a JSON array of strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeatureFlag {
    /// Unlocks the KP Blotter module.
    KpBlotter,
    /// Unlocks the Treasury module.
    Treasury,
    /// Unlocks the Business Permits module.
    BusinessPermits,
}

impl FeatureFlag {
    /// The ADR-0003 SCREAMING_SNAKE_CASE wire/storage string for this flag.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KpBlotter => "KP_BLOTTER",
            Self::Treasury => "TREASURY",
            Self::BusinessPermits => "BUSINESS_PERMITS",
        }
    }
}

impl fmt::Display for FeatureFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_strings_match_adr_0003() {
        assert_eq!(FeatureFlag::KpBlotter.as_str(), "KP_BLOTTER");
        assert_eq!(FeatureFlag::Treasury.as_str(), "TREASURY");
        assert_eq!(FeatureFlag::BusinessPermits.as_str(), "BUSINESS_PERMITS");
    }

    #[test]
    fn serde_round_trip_uses_screaming_snake_case() {
        let flags = vec![FeatureFlag::KpBlotter, FeatureFlag::Treasury];
        let json = serde_json::to_string(&flags).unwrap();
        assert_eq!(json, r#"["KP_BLOTTER","TREASURY"]"#);
        let back: Vec<FeatureFlag> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, flags);
    }

    #[test]
    fn unknown_flag_string_fails_deserialization() {
        let err = serde_json::from_str::<FeatureFlag>(r#""BOGUS""#).unwrap_err();
        assert!(err.to_string().contains("unknown variant"));
    }
}
