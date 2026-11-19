//! Public-value inventory audit (S15d): controllers are not value snapshots.
//! Partial API-101 clauses; no whole-requirement test marker.
use core::{fmt::Debug, hash::Hash};
use irig106_time::*;
fn value<T: Debug + Clone + PartialEq>() {}
fn scalar<T: Debug + Clone + Copy + PartialEq + Eq + Hash>() {}
macro_rules! values { ($($t:ty),* $(,)?) => {{ $(value::<$t>();)* [$(stringify!($t)),*] }}; }
macro_rules! scalars { ($($t:ty),* $(,)?) => { $(scalar::<$t>();)* }; }
#[test]
fn public_value_derive_inventory() {
    let mut covered = Vec::new();
    covered.extend(values! { absolute::AbsoluteTime, absolute::CalendarTime });
    covered.extend(values! { answer::ClockBasis, answer::CounterBasis, answer::OwnEnvelope, answer::StampEnvelope, answer::WordEnvelope, answer::AnswerBasis, answer::AnswerValue, answer::AnswerView });
    covered.extend(values! { bcd::DayFormatTime, bcd::DmyFormatTime });
    covered.extend(values! { ch4::Ch4Words, ch4::BcdDayInterpretation, ch4::BcdContext, ch4::Ch4Context, ch4::MissingCh4Context, ch4::BcdDayWording, ch4::Ch4Value, ch4::Ch4Reading });
    covered.extend(
        values! { correlation::ReferencePoint, correlation::TimeJump, correlation::RtcReset },
    );
    covered.extend(values! { csdw::TimeF1Layout, csdw::TimeF1Source, csdw::TimeF1Csdw, csdw::ItsQualification, csdw::QualifiedCsdw });
    covered.extend(values! { declarations::DetailedComparison, declarations::DeclarationFinding });
    covered.extend(values! { declarations::GoverningRange, declarations::ChannelDeclaration, declarations::SetupDeclarations, declarations::DeclarationPresence, declarations::GoverningSetup, declarations::DeclarationBasis, declarations::PreparedObservation, declarations::Declarations, declarations::Reading, declarations::Mapping, declarations::DeclarationPolicy, declarations::PacketFields, declarations::ComparisonStatus, declarations::Comparison, declarations::DeclarationField, declarations::DeclarationIssue, declarations::ComparisonFinding, declarations::ComparisonReport });
    covered.extend(values! { edition::PacketEdition, edition::EditionRules, edition::SetupVersionEvidence, edition::EditionEvidence, edition::EditionBasis, edition::EditionResolution });
    covered.extend(values! { error::TimeError });
    covered.extend(values! { extracted::ExtractionOrigin, extracted::StampInput<'static>, extracted::ResolvedCounter, extracted::StampInspection, extracted::StampAnswer, extracted::NetworkWord, extracted::TimeWord, extracted::ComponentBinding, extracted::WordContext, extracted::WordAnswer });
    covered.extend(values! { findings::Severity, findings::FindingCode, findings::ValueValidity, findings::FindingLocation, findings::RawValue, findings::RawField, findings::NtpEvidence, findings::FindingEvidence, findings::Finding });
    covered.extend(values! { format1::CalendarReading, format1::Format1Reading, format1::Format1Input<'static>, format1::QualifiedFormat1Reading, format1::EncodedFormat1 });
    covered.extend(values! { format2::ReferenceCapture, format2::Format2Input<'static>, format2::Format2Observation, format2::AdmissionPolicy, format2::NetworkAdmission, format2::NetworkConversion, format2::Format2Reference });
    covered.extend(values! { intra_packet::IntraPacketTimeFormat, intra_packet::IntraPacketTime });
    covered.extend(values! { leap::LeapSecondEntry, leap::LeapTableMetadata, leap::LeapSecondTable, leap::PtpConversionStatus, leap::PtpConversionBasis, leap::PtpConversion });
    covered.extend(values! { network_time::NetworkTimeProtocol, network_time::NetworkTimeFormat, network_time::TimeStatus, network_time::TimeF2Csdw, network_time::NtpTime, network_time::PtpTime, network_time::NetworkTime, network_time::Format2Reading });
    covered.extend(values! { ntp::EraBasis, ntp::EraDeclaration, ntp::PivotFraction, ntp::PivotOrigin, ntp::NtpPivot, ntp::NtpContext, ntp::EraIssue, ntp::NtpConversionStatus, ntp::NtpLeapStatus, ntp::NtpRounding, ntp::NtpConversionBasis, ntp::NtpConversion });
    covered.extend(values! { own_time::EpochCounting, own_time::EpochContext, own_time::OwnContext, own_time::OwnPolicy, own_time::CounterAnswer<'static>, own_time::OwnPoint, own_time::OwnUtc, own_time::OwnAnswer<'static> });
    covered.extend(values! { packet_standard::PacketStandard });
    covered.extend(values! { policy::YearSource, policy::YearOrder, policy::SharedSettings, policy::SharedPolicy, policy::QualificationBasis });
    covered.extend(values! { quality::TimeQuality });
    covered.extend(values! { reference::Normalization, reference::NetworkReference });
    covered.extend(values! { rtc::CounterOrder, rtc::SessionRtc });
    covered.extend(values! { scale::TimeScale, scale::ScaleOrigin, scale::ScaleEvidence, scale::ScalePolicy, scale::Scaled<format1::CalendarReading> });
    covered.extend(values! { secondary::SecondaryHeaderTime, secondary::SecondaryChecksum });
    covered.extend(values! { secondary_packet::TimeApplicability, secondary_packet::SecondaryPacket<'static>, secondary_packet::SecondaryPolicy, secondary_packet::RtcErtcCheck, secondary_packet::SecondaryInspection, secondary_packet::SecondaryWithCounter<'static, own_time::CounterAnswer<'static>>, secondary_packet::ChecksumCounts });
    covered.extend(values! { stream::StreamMode, stream::BufferCaps, stream::StreamPolicy, stream::StreamChannel, stream::StreamChannelChoice, stream::ReleaseReason, stream::StreamBasis, stream::StreamAnswer, stream::StreamBatch, stream::BufferUsage });
    covered.extend(values! { streaming::StreamingRef });
    covered.extend(values! { timeline::ReferenceStrategy, timeline::ChannelChoice, timeline::OfflinePolicy, timeline::TimelineFinding, timeline::PacketKind, timeline::YearOrigin, timeline::ReferenceValue, timeline::ReferenceInput, timeline::PacketObservation, timeline::PacketKey, timeline::Rejection, timeline::TimelineReference, timeline::OfflineSession, timeline::ReferencePosition, timeline::OfflineBasis, timeline::OfflineAnswer, timeline::OfflineQuality });
    covered.extend(values! { version::Format2Availability });
    covered.extend(values! { years::YearContext, years::YearCandidate, years::YearBasis });
    verify_inventory(&covered);
}
#[test]
fn fixed_scalar_recording_metadata_derive_inventory() {
    scalars! { DateFormat, TimeSource, TimeFormat, IrigTimeSource, Irig106Version,
    PacketStandard, SecHdrTimeFormat, NanosDuration, SignedNanos, SignedRtcTicks, SignedUnixSeconds,
    UnixSeconds, TaiSeconds, TaiUtcOffset };

    scalars! { absolute::AbsoluteTime, absolute::CalendarTime }
    scalars! { bcd::DayFormatTime, bcd::DmyFormatTime }
    scalars! { ch4::Ch4Words, ch4::BcdDayInterpretation, ch4::BcdContext, ch4::Ch4Context, ch4::MissingCh4Context, ch4::BcdDayWording, ch4::Ch4Value, ch4::Ch4Reading }
    scalars! { correlation::ReferencePoint, correlation::TimeJump, correlation::RtcReset }
    scalars! { csdw::TimeF1Layout, csdw::TimeF1Source, csdw::TimeF1Csdw, csdw::ItsQualification }
    scalars! { declarations::DeclarationPresence, declarations::Reading, declarations::Mapping, declarations::PacketFields, declarations::ComparisonStatus, declarations::DeclarationField, declarations::DeclarationIssue }
    scalars! { edition::PacketEdition, edition::EditionRules, edition::SetupVersionEvidence, edition::EditionBasis }
    scalars! { extracted::NetworkWord, extracted::TimeWord }
    scalars! { findings::Severity, findings::FindingCode, findings::ValueValidity, findings::FindingLocation, findings::RawValue, findings::NtpEvidence }
    scalars! { format1::CalendarReading, format1::Format1Reading, format1::Format1Input<'static>, format1::EncodedFormat1 }
    scalars! { format2::ReferenceCapture, format2::Format2Input<'static>, format2::NetworkAdmission }
    scalars! { intra_packet::IntraPacketTimeFormat, intra_packet::IntraPacketTime }
    scalars! { leap::LeapSecondEntry, leap::PtpConversionStatus }
    scalars! { network_time::NetworkTimeProtocol, network_time::NetworkTimeFormat, network_time::TimeStatus, network_time::TimeF2Csdw, network_time::NtpTime, network_time::PtpTime, network_time::NetworkTime, network_time::Format2Reading }
    scalars! { ntp::EraBasis, ntp::PivotFraction, ntp::PivotOrigin, ntp::EraIssue, ntp::NtpConversionStatus, ntp::NtpLeapStatus, ntp::NtpRounding }
    scalars! { own_time::EpochCounting }
    scalars! { packet_standard::PacketStandard }
    scalars! { policy::YearSource }
    scalars! { rtc::CounterOrder, rtc::SessionRtc }
    scalars! { scale::TimeScale, scale::ScaleOrigin, scale::ScaleEvidence, scale::ScalePolicy, scale::Scaled<format1::CalendarReading> }
    scalars! { secondary::SecondaryHeaderTime, secondary::SecondaryChecksum }
    scalars! { secondary_packet::ChecksumCounts }
    scalars! { stream::StreamMode, stream::BufferCaps, stream::StreamChannel, stream::StreamChannelChoice, stream::ReleaseReason, stream::BufferUsage }
    scalars! { streaming::StreamingRef }
    scalars! { timeline::ReferenceStrategy, timeline::ChannelChoice, timeline::PacketKind, timeline::YearOrigin, timeline::ReferenceValue, timeline::PacketKey, timeline::Rejection, timeline::ReferencePosition }
    scalars! { version::Format2Availability }
    scalars! { years::YearContext, years::YearCandidate }
}

fn verify_inventory(covered: &[&str]) {
    use std::collections::BTreeSet;
    let covered: BTreeSet<_> = covered
        .iter()
        .map(|t| t.split('<').next().unwrap().replace(' ', ""))
        .collect();
    let controllers: BTreeSet<_> = [
        "correlation::TimeCorrelator",
        "streaming::StreamingTimeCorrelator",
        "timeline::TimelineBuilder",
        "timeline::OfflineTimeline",
        "stream::StreamTimeline",
        "declarations::DeclarationAudit",
        "secondary_packet::SecondaryAudit",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut actual = BTreeSet::new();
    for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let module = path.file_stem().unwrap().to_str().unwrap();
        // Internal file's two public values are re-exported through declarations.
        let module = if module == "declaration_checks" {
            "declarations"
        } else {
            module
        };
        for line in std::fs::read_to_string(&path).unwrap().lines() {
            if let Some(rest) = line
                .strip_prefix("pub struct ")
                .or_else(|| line.strip_prefix("pub enum "))
            {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                actual.insert(format!("{module}::{name}"));
            }
        }
    }
    assert!(controllers.is_subset(&actual));
    let values: BTreeSet<_> = actual.difference(&controllers).cloned().collect();
    assert_eq!(
        covered, values,
        "classify/check every public value when the source inventory changes"
    );
}
