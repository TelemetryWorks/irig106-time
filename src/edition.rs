//! Caller policy for packet-layout evidence; never a header-version guess.
//! Sources/pages: docs/analysis/Q-EDITION-2026-11-13.md; RDG-007.
use crate::{Irig106Version, TimeF1Layout};
use alloc::string::String;

/// Explicit supported release, independent of shared version-code list names.
/// **Traces:** L3-EDN-103 ← L2-EDN-103 ← L1-EDN-003
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketEdition {
    /// RCC 106-04.
    V04,
    /// RCC 106-05.
    V05,
    /// RCC 106-07.
    V07,
    /// RCC 106-09.
    V09,
    /// RCC 106-11.
    V11,
    /// RCC 106-13.
    V13,
    /// RCC 106-15.
    V15,
    /// RCC 106-17.
    V17,
    /// RCC 106-19.
    V19,
    /// RCC 106-20.
    V20,
    /// RCC 106-22.
    V22,
    /// RCC 106-23.
    V23,
    /// RCC 106-24.
    V24,
    /// RCC 106-24R1.
    V24R1,
}
/// Edition facts supported by the cited rule boundaries, not a fallback edition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditionRules {
    /// EXT / SRC / SRC+ITS layout.
    pub layout: TimeF1Layout,
    /// Chapter containing packet formats.
    pub chapter: u8,
    /// Highest header release code, NOT the Time Format1 current type version.
    pub max_data_type_version: u8,
    /// Format2 is defined from 106-17.
    pub format2: bool,
    /// Source boundary; historical scheduling guarantees require separate audit.
    pub source: &'static str,
}
impl PacketEdition {
    /// Rule table: §11.2.1.1 e; 04 Fig10-15; 05 Fig10-16a; 17+ Fig11-12.
    /// 20 uses 19's rules; 23/24/24R1 retain 22's version codes (T-9).
    /// **Traces:** L3-EDN-103 ← L2-EDN-103 ← L1-EDN-003
    pub fn rules(self) -> EditionRules {
        use PacketEdition::*;
        let max_data_type_version = match self {
            V04 => 1,
            V05 => 2,
            V07 => 3,
            V09 => 4,
            V11 => 5,
            V13 => 6,
            V15 => 7,
            V17 => 8,
            V19 | V20 => 9,
            V22 | V23 | V24 | V24R1 => 10,
        };
        let (layout, chapter, source) = match self {
            V04 => (
                TimeF1Layout::Legacy04,
                10,
                "106-04 10.6.3.3 Fig10-15, 10-30/31; full PDF288/289",
            ),
            V05 | V07 | V09 | V11 | V13 | V15 => (
                TimeF1Layout::Source05,
                10,
                "106-05 Fig10-16a, 10-37/PDF41; RDG-007 legacy reserved31-12",
            ),
            _ => (
                TimeF1Layout::SourceWithIts17,
                11,
                "106-17+ 11.2.3.2 a, Fig11-12; 106-24R1 11-24/PDF32; T-9",
            ),
        };
        EditionRules {
            layout,
            chapter,
            max_data_type_version,
            format2: chapter == 11,
            source,
        }
    }
    fn from_tmats(raw: &str) -> Option<Self> {
        use PacketEdition::*;
        Some(match raw {
            "4" | "04" => V04,
            "5" | "05" => V05,
            "7" | "07" => V07,
            "9" | "09" => V09,
            "11" => V11,
            "13" => V13,
            "15" => V15,
            "17" => V17,
            "19" => V19,
            "20" => V20,
            "22" => V22,
            "23" => V23,
            "24" => V24,
            _ => return None,
        })
    }
}
/// Caller identifies whether the setup field exists; reserved legacy words are
/// retained, never interpreted as RCCVER. Setup alone never selects EXT/SRC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetupVersionEvidence {
    /// No applicable field supplied (106-04 has none).
    #[default]
    Unavailable,
    /// Legacy reserved word (106-05 Fig10-34b), not a version field.
    Reserved(u32),
    /// Known applicable 106-07+ CH10VER/RCCVER code, separate from header byte.
    Rccver(u8),
}
impl SetupVersionEvidence {
    /// Decode only an applicable setup field through the shared mapping.
    pub fn decoded(self) -> Option<Irig106Version> {
        match self {
            Self::Rccver(raw) => Some(Irig106Version::from_setup_record(raw)),
            _ => None,
        }
    }
}
/// **Traces:** L3-EDN-201 ← L2-EDN-201 ← L1-EDN-002
/// Original evidence shown side by side; raw TMATS is retained even if malformed
/// or not adopted. No TMATS runtime dependency, host clock or hidden fallback.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditionEvidence {
    /// Caller declaration/override wins over contradictory evidence.
    pub override_edition: Option<PacketEdition>,
    /// Exact G\106 value, describing the TMATS file until explicitly adopted.
    pub tmats_edition: Option<String>,
    /// Caller opts into treating G\106 as packet edition.
    pub use_tmats: bool,
    /// Separate, caller-qualified setup version domain.
    pub setup: SetupVersionEvidence,
    /// Primary header's at-or-below hint, not an exact recording edition.
    pub data_type_version: Option<u8>,
}
/// ADR-0028 vocabulary; no fallback assumption selects a packet layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditionBasis {
    /// Explicit caller declaration, including retained contrary evidence.
    Override,
    /// Declared, assumed from TMATS through caller opt-in.
    DeclaredAssumedFromTmats,
    /// No usable uncontradicted adopted declaration; raw alternatives remain.
    Unresolved,
}
/// Immutable resolution; evidence cannot be altered after selecting its basis.
/// **Traces:** L3-EDN-201 ← L2-EDN-201 ← L1-EDN-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditionResolution {
    edition: Option<PacketEdition>,
    basis: EditionBasis,
    evidence: EditionEvidence,
    contradiction: bool,
}
impl EditionResolution {
    /// Selected edition, never inferred from header/setup codes alone.
    pub fn edition(&self) -> Option<PacketEdition> {
        self.edition
    }
    /// Override / declared-assumed / unresolved provenance.
    pub fn basis(&self) -> EditionBasis {
        self.basis
    }
    /// All original declarations and hints, including contradictions.
    pub fn evidence(&self) -> &EditionEvidence {
        &self.evidence
    }
    /// Only the no-override, opted-in declaration contradicted by packet case.
    /// TF-031 WARNING is owner-accepted (2026-11-14): body kept, EXT/SRC withheld.
    pub fn contradiction(&self) -> bool {
        self.contradiction
    }
}
/// Resolve before qualifying EXT/SRC/FMT/ITS (owner Q-EDITION 2026-11-13).
/// A known header release ABOVE an adopted edition contradicts it; lower
/// versions are permitted. Unknown codes are retained, not ordered numerically.
/// Setup/TMATS declaration differences alone do not create an automatic finding.
/// **Traces:** L3-EDN-201 ← L2-EDN-201 ← L1-EDN-002
pub fn resolve_edition(evidence: EditionEvidence) -> EditionResolution {
    let mut edition = evidence.override_edition;
    let mut basis = if edition.is_some() {
        EditionBasis::Override
    } else {
        EditionBasis::Unresolved
    };
    let mut contradiction = false;
    if edition.is_none() && evidence.use_tmats {
        edition = evidence
            .tmats_edition
            .as_deref()
            .and_then(PacketEdition::from_tmats);
        if let Some(e) = edition {
            contradiction = evidence.data_type_version.is_some_and(|v| {
                !matches!(
                    Irig106Version::from_data_type_version(v),
                    Irig106Version::Unknown(_)
                ) && v > e.rules().max_data_type_version
            });
            if contradiction {
                edition = None;
            } else {
                basis = EditionBasis::DeclaredAssumedFromTmats;
            }
        }
    }
    EditionResolution {
        edition,
        basis,
        evidence,
        contradiction,
    }
}
