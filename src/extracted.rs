//! S10b: Ch11 §11.2.1.1g/1.3b (11-9/11, PDF17/19), Ch4 §4.7,
//! Ch9 Table9-11 and contract3.8/4.6. Extraction/scheduling belong to the caller.
use crate::{
    ch4::*,
    findings::*,
    format1::CalendarReading,
    format2::NetworkConversion,
    intra_packet::*,
    leap::*,
    ntp::*,
    own_time::*,
    rtc::{CounterOrder, SessionRtc},
    scale::*,
    secondary_packet::SecondaryPolicy,
    stream::StreamAnswer,
    timeline::*,
    *,
};
use alloc::{boxed::Box, string::String, vec::Vec};
fn need(context: &'static str) -> TimeError {
    TimeError::NeedsContext { context }
}
fn over() -> TimeError {
    TimeError::QuantityOverflow {
        quantity: "extracted time coordinate",
    }
}
/// Caller provenance, independent of any query key derived from the value.
/// **Traces:** L3-ANS-104 ← L2-ANS-104 ← L1-ANS-004
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractionOrigin {
    /// Enclosing packet identity and unwrapped primary counter/session.
    pub key: PacketKey,
    /// Enclosing data channel, not a selected time-channel declaration.
    pub channel: u16,
    /// First extracted byte if known; no body layout is inferred.
    pub byte_offset: Option<u64>,
    /// Nonempty decoder field/measurement identity, retained verbatim.
    pub field: String,
}
impl ExtractionOrigin {
    fn validate(&self, last: u64) -> Result<()> {
        if self.key.session == 0 || self.field.trim().is_empty() {
            return Err(TimeError::InvalidTimelineInput("extraction session/field"));
        }
        if self
            .byte_offset
            .is_some_and(|n| n.checked_add(last).is_none())
        {
            return Err(over());
        }
        Ok(())
    }
    fn location(&self, relative: Option<u64>) -> FindingLocation {
        FindingLocation {
            session: Some(self.key.session),
            packet: Some(self.key.packet),
            channel: Some(self.channel),
            byte_offset: relative.and_then(|r| self.byte_offset.and_then(|n| n.checked_add(r))),
        }
    }
}
fn report(
    out: &mut Vec<Finding>,
    o: &ExtractionOrigin,
    p: &SecondaryPolicy,
    finding: (u8, ValueValidity),
    offset: Option<u64>,
    error: Option<TimeError>,
    raw: Option<RawField>,
) -> Result<()> {
    let (id, validity) = finding;
    let mut e = FindingEvidence::new(o.location(offset), validity)
        .with_raw_rtc(Rtc::from_raw((o.key.ticks & ((1i128 << 48) - 1)) as u64));
    if let Some(error) = error {
        e = e.with_decode_error(error);
    }
    if let Some(raw) = raw {
        e = e.with_raw_field(raw);
    }
    out.push(Finding::new(
        FindingCode::from_number(id)?,
        p.levels.get(&id).copied(),
        e,
    )?);
    Ok(())
}
fn conversion_findings(
    out: &mut Vec<Finding>,
    o: &ExtractionOrigin,
    p: &SecondaryPolicy,
    point: &OwnPoint,
) -> Result<()> {
    for id in [21, 22] {
        if (id == 21 && point.utc_conversion().status() == PtpConversionStatus::PossiblyOutOfDate)
            || (id == 22 && point.utc_conversion().basis().leap_second)
        {
            report(
                out,
                o,
                p,
                (id, ValueValidity::Kept),
                Some(0),
                None,
                Some(RawField {
                    name: "derived_tai_ns",
                    observed: RawValue::Signed(point.tai_ns().get()),
                    expected: None,
                }),
            )?;
        }
    }
    Ok(())
}
/// Eight-byte stamp plus explicit ordering for modular counter unwrapping.
#[derive(Debug, Clone)]
pub struct StampInput<'a> {
    /// Caller extraction metadata.
    pub origin: ExtractionOrigin,
    /// Caller query identifier; distinct IDs allow multiple messages from one
    /// packet in the stream core. Original packet identity stays in origin.
    pub query_id: u64,
    /// Original primary header flags; never inherited from another packet.
    pub flags: u8,
    /// Extracted stamp prefix; at most8 bytes are inspected/retained.
    pub bytes: &'a [u8],
    /// Caller guarantee, not an inferred number of wraps.
    pub order: CounterOrder,
}
/// Actual answer at the derived stamp query, including all original basis fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedCounter {
    /// Offline lookup at the stamp counter, allowing its own reference selection.
    Offline(Box<OfflineAnswer>),
    /// Already released stream query, retaining caps/mode/fallback and no revision.
    Stream(Box<StreamAnswer>),
}
impl ResolvedCounter {
    /// Own an offline lookup snapshot with bounded enum stack size.
    pub fn offline(answer: OfflineAnswer) -> Self {
        Self::Offline(Box::new(answer))
    }
    /// Own a released stream snapshot, including its original fallback/policy.
    pub fn stream(answer: StreamAnswer) -> Self {
        Self::Stream(Box::new(answer))
    }

    /// Borrow the immutable query answer for common S10a2 conversion routines.
    pub fn as_counter(&self) -> CounterAnswer<'_> {
        match self {
            Self::Offline(a) => CounterAnswer::Offline(a),
            Self::Stream(a) => CounterAnswer::Stream(a),
        }
    }
    fn offset(&self) -> i128 {
        match self {
            Self::Offline(a) => a.counter_ns,
            Self::Stream(a) => a.counter_ns,
        }
    }
}
/// Flags/raw stamp and simultaneous findings, before calendar resolution.
/// **Traces:** L3-IPT-107 ← L2-IPT-107 ← L1-IPT-007
#[derive(Debug, Clone)]
pub struct StampInspection {
    origin: ExtractionOrigin,
    flags: u8,
    query_id: u64,
    raw: Vec<u8>,
    order: CounterOrder,
    time: Option<IntraPacketTime>,
    query: Option<PacketKey>,
    remainder: u64,
    issue: Option<TimeError>,
    findings: Vec<Finding>,
    policy: SecondaryPolicy,
}
impl StampInspection {
    /// Decoder origin retained before as well as after query resolution.
    pub fn origin(&self) -> &ExtractionOrigin {
        &self.origin
    }
    /// Immutable severity settings; they do not affect value admissibility.
    pub fn policy(&self) -> &SecondaryPolicy {
        &self.policy
    }

    /// Caller query identity, separate from the enclosing packet identity.
    pub fn query_id(&self) -> u64 {
        self.query_id
    }
    /// Original flags.
    pub fn flags(&self) -> u8 {
        self.flags
    }
    /// At most8 original bytes, unaffected by reserved warnings.
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }
    /// Explicit caller ordering retained independently of the decoded format.
    pub fn order(&self) -> CounterOrder {
        self.order
    }
    /// Decoded wire value; no fabricated value for invalid flag combinations.
    pub fn time(&self) -> Option<IntraPacketTime> {
        self.time
    }
    /// Named-session counter query for RTC/ERTC; None for absolute/invalid stamps.
    pub fn query(&self) -> Option<PacketKey> {
        self.query
    }
    /// Simultaneous findings; TF028 can coexist with a usable RTC stamp.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
    /// Resolve only this query. The caller may use OfflineTimeline::lookup or
    /// register/retrieve a stream query using its normal progress/caps policy.
    /// Absolute stamps never invoke the resolver. Mismatched returned keys fail.
    /// **Traces:** L3-ANS-103 ← L2-ANS-103 ← L1-ANS-003
    pub fn resolve(
        self,
        context: &OwnContext,
        resolve: impl FnOnce(PacketKey) -> Result<ResolvedCounter>,
    ) -> Result<StampAnswer> {
        let mut a = StampAnswer {
            inspection: self,
            counter: None,
            point: None,
            calendar: None,
            offset: None,
            issue: None,
            context: context.clone(),
        };
        a.issue = a.inspection.issue.clone();
        if a.issue.is_some() {
            return Ok(a);
        }
        let conversion = if let Some(key) = a.inspection.query {
            let counter = resolve(key)?;
            if counter.as_counter().key() != key {
                return Err(TimeError::InvalidTimelineInput(
                    "stamp resolver key/session mismatch",
                ));
            }
            a.calendar = counter.as_counter().reading().and_then(|c| {
                c.checked_shift(SignedNanos::new(a.inspection.remainder.into()))
                    .ok()
            });
            a.offset = Some(SignedNanos::new(
                counter
                    .offset()
                    .checked_add(a.inspection.remainder.into())
                    .ok_or_else(over)?,
            ));
            let converted = (|| {
                let b = counter.as_counter();
                let ns = b
                    .normalized(&context.table)?
                    .checked_add(b.ertc_shift(a.inspection.remainder.into())?)
                    .ok_or_else(over)?;
                crate::own_time::point(
                    ns,
                    b.reading()
                        .ok_or(need("stamp reference calendar"))?
                        .scale
                        .effective(),
                    &context.table,
                )
            })();
            a.counter = Some(counter);
            converted
        } else {
            match a.inspection.time {
                Some(IntraPacketTime::Ch4(t)) => {
                    let Ch4Value::BinaryElapsed(n) = Ch4Words::from(t).binary(None)?.value() else {
                        return Err(need("binary stamp"));
                    };
                    crate::own_time::from_epoch(
                        n.get().into(),
                        context.ch4.as_ref(),
                        &context.table,
                    )
                }
                Some(IntraPacketTime::Ieee1588(t)) => crate::own_time::from_epoch(
                    i128::from(t.seconds) * 1_000_000_000 + i128::from(t.nanoseconds),
                    context.ieee.as_ref(),
                    &context.table,
                ),
                _ => Err(need("readable stamp")),
            }
        };
        match conversion {
            Ok(point) => {
                a.calendar = point.native();
                conversion_findings(
                    &mut a.inspection.findings,
                    &a.inspection.origin,
                    &a.inspection.policy,
                    &point,
                )?;
                a.point = Some(point);
            }
            Err(e) => a.issue = Some(e),
        }
        Ok(a)
    }
}
/// Raw, partial or absolute stamp result with the actual query answer beside it.
#[derive(Debug)]
pub struct StampAnswer {
    inspection: StampInspection,
    counter: Option<ResolvedCounter>,
    point: Option<OwnPoint>,
    calendar: Option<CalendarReading>,
    offset: Option<SignedNanos>,
    issue: Option<TimeError>,
    context: OwnContext,
}
impl StampAnswer {
    /// Original enclosing packet/field, not overwritten by the query coordinate.
    pub fn origin(&self) -> &ExtractionOrigin {
        &self.inspection.origin
    }
    /// Original bytes, flags, ordering, findings and query.
    pub fn inspection(&self) -> &StampInspection {
        &self.inspection
    }
    /// Actual offline/stream result at the stamp's floor-RTC query, if relative.
    pub fn counter(&self) -> Option<&ResolvedCounter> {
        self.counter.as_ref()
    }
    /// Qualified physical instant when all required context exists.
    pub fn point(&self) -> Option<&OwnPoint> {
        self.point.as_ref()
    }
    /// Native calendar, possibly unknown-year/scale; consult issue and scale.
    pub fn calendar(&self) -> Option<CalendarReading> {
        self.calendar
    }
    /// Nominal session counter duration, including ERTC's sub-tick ns remainder.
    pub fn counter_offset(&self) -> Option<SignedNanos> {
        self.offset
    }
    /// Partial/invalid reason, never substituted by epoch/day1 or a zero time.
    pub fn issue(&self) -> Option<&TimeError> {
        self.issue.as_ref()
    }
    /// Immutable conversion context/table snapshot used by this answer.
    pub fn context(&self) -> &OwnContext {
        &self.context
    }
}
/// Bound stamp decoding, collect all flag findings, and derive a separate query.
/// **Traces:** L3-IPT-107 ← L2-IPT-107 ← L1-IPT-007
pub fn inspect_stamp(input: StampInput<'_>, policy: &SecondaryPolicy) -> Result<StampInspection> {
    input.origin.validate(7)?;
    let mut r = StampInspection {
        origin: input.origin,
        flags: input.flags,
        query_id: input.query_id,
        raw: input.bytes.iter().take(8).copied().collect(),
        order: input.order,
        time: None,
        query: None,
        remainder: 0,
        issue: None,
        findings: Vec::new(),
        policy: policy.clone(),
    };
    let flags = r.flags;
    for (id, condition, error, validity) in [
        (
            15,
            flags & 0xc0 == 0x40,
            TimeError::InvalidPacketFlags { flags },
            ValueValidity::Unusable,
        ),
        (
            6,
            flags & 0x0c == 0x0c,
            TimeError::ReservedPacketTimeFormat { flags },
            ValueValidity::Kept,
        ),
        (
            28,
            flags & 0xc0 != 0 && flags & 0x0c == 0x0c,
            TimeError::UndefinedSecondaryTimeFormat { flags },
            ValueValidity::Unusable,
        ),
    ] {
        if condition {
            report(
                &mut r.findings,
                &r.origin,
                policy,
                (id, validity),
                None,
                Some(error),
                Some(RawField {
                    name: "packet.flags",
                    observed: RawValue::Unsigned(flags.into()),
                    expected: None,
                }),
            )?;
        }
    }
    if flags & 0x20 != 0 {
        report(
            &mut r.findings,
            &r.origin,
            policy,
            (24, ValueValidity::Kept),
            None,
            None,
            Some(RawField {
                name: "packet.flags",
                observed: RawValue::Unsigned(flags.into()),
                expected: None,
            }),
        )?;
    }
    if r.raw.len() < 8 {
        let e = TimeError::BufferTooShort {
            expected: 8,
            actual: r.raw.len(),
        };
        report(
            &mut r.findings,
            &r.origin,
            policy,
            (4, ValueValidity::Unusable),
            Some(0),
            Some(e.clone()),
            None,
        )?;
        r.issue = Some(e);
        return Ok(r);
    }
    let fmt = IntraPacketTimeFormat::from_packet_flags(flags);
    let reserved = match fmt {
        IntraPacketTimeFormat::Rtc48 => validate_rtc48_reserved(&r.raw).err().map(|e| (6, e)),
        IntraPacketTimeFormat::Ch4Binary => crate::secondary::validate_ch4_reserved(&r.raw)
            .err()
            .map(|e| (0, e)),
        _ => None,
    };
    if let Some((n, e)) = reserved {
        let bits = u16::from_le_bytes([r.raw[n], r.raw[n + 1]]);
        report(
            &mut r.findings,
            &r.origin,
            policy,
            (5, ValueValidity::Kept),
            Some(n as u64),
            Some(e),
            Some(RawField {
                name: "stamp.reserved",
                observed: RawValue::Unsigned(bits.into()),
                expected: Some(RawValue::Unsigned(0)),
            }),
        )?;
    }
    match parse_intra_packet_time(&r.raw, fmt) {
        Err(e) => {
            if !matches!(fmt, IntraPacketTimeFormat::InvalidFlags(_)) {
                report(
                    &mut r.findings,
                    &r.origin,
                    policy,
                    (4, ValueValidity::Unusable),
                    Some(0),
                    Some(e.clone()),
                    None,
                )?;
            }
            r.issue = Some(e);
        }
        Ok(time) => r.time = Some(time),
    }
    let rtc = match r.time {
        Some(IntraPacketTime::Rtc(t)) => Some(t),
        Some(IntraPacketTime::Ertc(t)) => {
            r.remainder = (t.to_nanos() % 100) as u64;
            Some(Rtc::from_raw(
                ((t.to_nanos() / 100) & ((1u128 << 48) - 1)) as u64,
            ))
        }
        _ => None,
    };
    if let Some(rtc) = rtc {
        let origin = SessionRtc {
            session: r.origin.key.session,
            rtc: Rtc::from_raw((r.origin.key.ticks & ((1i128 << 48) - 1)) as u64),
        };
        match origin
            .delta_to(
                SessionRtc {
                    session: origin.session,
                    rtc,
                },
                r.order,
            )
            .and_then(|d| r.origin.key.ticks.checked_add(d.get()).ok_or_else(over))
        {
            Ok(ticks) => {
                r.query = Some(PacketKey {
                    ticks,
                    packet: r.query_id,
                    ..r.origin.key
                })
            }
            Err(e) => r.issue = Some(e),
        }
    }
    Ok(r)
}
/// All six C-d\NTM kinds; components are not complete timestamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkWord {
    /// Full PTP seconds and nanoseconds, in TAI.
    Ptp(PtpTime),
    /// PTP seconds component only.
    PtpSeconds(u32),
    /// PTP nanoseconds component only, less than1e9.
    PtpNanoseconds(u32),
    /// Full NTP seconds/fraction, RDG-012 UTC reckoning.
    Ntp(NtpTime),
    /// NTP seconds component only.
    NtpSeconds(u32),
    /// NTP fraction component only,2^-32s units.
    NtpFraction(u32),
}
/// Caller-supplied words in significance order, not payload byte extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWord {
    /// Chapter4 binary weighting, including low bit15 as count data.
    Binary(Ch4Words),
    /// Chapter4 BCD, with the unresolved historical wording retained.
    Bcd(Ch4Words),
    /// Table9-11 full or component network time.
    Network(NetworkWord),
}
/// Explicit binding of matching components; no proximity or zero-fill heuristic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentBinding {
    /// Exact extraction being completed.
    pub target: ExtractionOrigin,
    /// Companion extraction; may differ in packet/channel, but same session.
    pub other: ExtractionOrigin,
    /// Caller-extracted complementary value, not an inferred missing field.
    pub word: NetworkWord,
}
/// Immutable caller word context; no unrelated packet answer is accepted here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordContext {
    /// Explicit binary epoch and shared table; IEEE epoch is never a binary fallback.
    pub own: OwnContext,
    /// Full day/year and selected interpretation for BCD.
    pub bcd: BcdContext,
    /// BCD scale assumption, default Unknown; source wording stays unresolved.
    pub bcd_scale: TimeScale,
    /// RDG-012 era declaration/pivots, or its labelled era0 fallback.
    pub ntp: NtpContext,
    /// Explicit complementary-word binding, retained in the answer.
    pub matching: Option<ComponentBinding>,
}
impl WordContext {
    /// Empty caller context; only the versioned S04 table and RDG-012 policy apply.
    pub fn new(session: usize) -> Self {
        Self {
            own: OwnContext::default(),
            bcd: BcdContext::default(),
            bcd_scale: TimeScale::Unknown,
            ntp: NtpContext::new(session),
            matching: None,
        }
    }
}
/// Caller word, extraction, context and conversion; raw values always retained.
#[derive(Debug)]
pub struct WordAnswer {
    origin: ExtractionOrigin,
    word: TimeWord,
    context: WordContext,
    chapter4: Option<Ch4Reading>,
    network: Option<NetworkConversion>,
    point: Option<OwnPoint>,
    calendar: Option<CalendarReading>,
    issue: Option<TimeError>,
    findings: Vec<Finding>,
    policy: SecondaryPolicy,
}
impl WordAnswer {
    /// Exact decoder provenance.
    pub fn origin(&self) -> &ExtractionOrigin {
        &self.origin
    }
    /// Original full/component kind and raw values.
    pub fn word(&self) -> TimeWord {
        self.word
    }
    /// All supplied context, including both explicitly bound extraction origins.
    pub fn context(&self) -> &WordContext {
        &self.context
    }
    /// Snapshot of severity overrides; validity never changes with levels.
    pub fn policy(&self) -> &SecondaryPolicy {
        &self.policy
    }
    /// Weighting/partial fields and historical BCD wording, before scale conversion.
    pub fn chapter4(&self) -> Option<&Ch4Reading> {
        self.chapter4.as_ref()
    }
    /// Complete network conversion and its era/table/currency/ambiguity evidence.
    pub fn network(&self) -> Option<&NetworkConversion> {
        self.network.as_ref()
    }
    /// Chapter4 physical conversion, when explicit epoch/scale context suffices.
    pub fn point(&self) -> Option<&OwnPoint> {
        self.point.as_ref()
    }
    /// Native calendar; PTP stays TAI here, UTC is in its separate conversion.
    pub fn calendar(&self) -> Option<CalendarReading> {
        self.calendar
    }
    /// Missing context or malformed value reason, with raw values retained.
    pub fn issue(&self) -> Option<&TimeError> {
        self.issue.as_ref()
    }
    /// Findings including NTP zero/ambiguity and PTP stale-offset status.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}
fn complete(w: NetworkWord, o: &ExtractionOrigin, c: &WordContext) -> Result<Option<NetworkWord>> {
    if let NetworkWord::PtpNanoseconds(n) = w {
        if n >= 1_000_000_000 {
            return Err(TimeError::OutOfRange {
                field: "ptp_nanoseconds",
                value: n,
                max: 999_999_999,
            });
        }
    }
    if matches!(w, NetworkWord::Ptp(_) | NetworkWord::Ntp(_)) {
        return Ok(Some(w));
    }
    let Some(b) = &c.matching else {
        return Ok(None);
    };
    if b.target != *o || b.other.key.session != o.key.session {
        return Ok(None);
    }
    b.other.validate(0)?;
    Ok(match (w, b.word) {
        (NetworkWord::PtpSeconds(s), NetworkWord::PtpNanoseconds(n))
        | (NetworkWord::PtpNanoseconds(n), NetworkWord::PtpSeconds(s)) => {
            Some(NetworkWord::Ptp(PtpTime {
                seconds: s,
                nanoseconds: n,
            }))
        }
        (NetworkWord::NtpSeconds(s), NetworkWord::NtpFraction(n))
        | (NetworkWord::NtpFraction(n), NetworkWord::NtpSeconds(s)) => {
            Some(NetworkWord::Ntp(NtpTime {
                seconds: s,
                fraction: n,
            }))
        }
        _ => None,
    })
}
/// Resolve already-extracted values only. Missing components/context stay partial.
/// **Traces:** L3-ANS-104 ← L2-ANS-104 ← L1-ANS-004
/// **Traces:** L3-WORD-102 ← L2-WORD-102 ← L1-WORD-002
pub fn answer_word(
    origin: ExtractionOrigin,
    word: TimeWord,
    context: &WordContext,
    policy: &SecondaryPolicy,
) -> Result<WordAnswer> {
    origin.validate(0)?;
    let mut a = WordAnswer {
        origin,
        word,
        context: context.clone(),
        chapter4: None,
        network: None,
        point: None,
        calendar: None,
        issue: None,
        findings: Vec::new(),
        policy: policy.clone(),
    };
    let converted = (|| -> Result<()> {
        match word {
            TimeWord::Binary(w) => {
                let r = w.binary(None)?;
                let Ch4Value::BinaryElapsed(n) = r.value() else {
                    return Err(need("binary words"));
                };
                a.chapter4 = Some(r);
                let p = crate::own_time::from_epoch(
                    n.get().into(),
                    context.own.ch4.as_ref(),
                    &context.own.table,
                )?;
                a.calendar = p.native();
                a.point = Some(p);
            }
            TimeWord::Bcd(w) => {
                let r = w.bcd(context.bcd)?;
                if r.reserved_bits() != 0 {
                    report(
                        &mut a.findings,
                        &a.origin,
                        policy,
                        (5, ValueValidity::Kept),
                        Some(0),
                        None,
                        Some(RawField {
                            name: "word.bcd_reserved",
                            observed: RawValue::Unsigned(r.reserved_bits().into()),
                            expected: Some(RawValue::Unsigned(0)),
                        }),
                    )?;
                }
                a.chapter4 = Some(r);
                let t = r
                    .calendar()
                    .ok_or(need("BCD year/full day/interpretation"))?;
                let c = CalendarReading {
                    year: t.year(),
                    day_of_year: t.day_of_year(),
                    hours: t.hours(),
                    minutes: t.minutes(),
                    seconds: t.seconds(),
                    nanoseconds: t.nanoseconds(),
                    scale: ScaleEvidence::assumed(context.bcd_scale),
                };
                a.calendar = Some(c);
                let n = crate::own_time::normalize(c, &context.own.table)?;
                a.point = Some(crate::own_time::point(
                    n,
                    context.bcd_scale,
                    &context.own.table,
                )?);
            }
            TimeWord::Network(w) => {
                match complete(w, &a.origin, context)?.ok_or(need("matching network component"))? {
                    NetworkWord::Ntp(raw) => {
                        if context.ntp.session != a.origin.key.session
                            && (raw.seconds != 0 || raw.fraction != 0)
                        {
                            return Err(need("NTP word session context"));
                        }
                        let n = convert_ntp(raw, &context.ntp, Some(&context.own.table));
                        for f in n.findings(a.origin.location(Some(0)), None)? {
                            a.findings.push(Finding::new(
                                f.code(),
                                policy.levels.get(&f.code().number()).copied(),
                                f.into_evidence(),
                            )?);
                        }
                        a.calendar = n.utc();
                        a.network = Some(NetworkConversion::Ntp(n));
                        if a.calendar.is_none() {
                            return Err(need("NTP era/zero/calendar context"));
                        }
                    }
                    NetworkWord::Ptp(raw) => {
                        let p = context.own.table.convert_ptp(raw)?;
                        for f in p.findings(a.origin.location(Some(0)), None)? {
                            a.findings.push(Finding::new(
                                f.code(),
                                policy.levels.get(&f.code().number()).copied(),
                                f.into_evidence(),
                            )?);
                        }
                        let mut c = crate::leap::calendar(
                            SignedUnixSeconds::new(raw.seconds.into()),
                            raw.nanoseconds,
                            false,
                        )?;
                        c.scale = ScaleEvidence::standard(TimeScale::Tai);
                        a.calendar = Some(c);
                        let missing = p.utc().is_none();
                        a.network = Some(NetworkConversion::Ptp(p));
                        if missing {
                            return Err(need("UTC offset for PTP word"));
                        }
                    }
                    _ => return Err(need("complete network word")),
                }
            }
        }
        Ok(())
    })();
    if let Some(p) = &a.point {
        conversion_findings(&mut a.findings, &a.origin, policy, p)?;
    }
    if let Err(e) = converted {
        if matches!(e, TimeError::InvalidTimelineInput(_)) {
            return Err(e);
        }
        if !matches!(e, TimeError::NeedsContext { .. }) {
            report(
                &mut a.findings,
                &a.origin,
                policy,
                (4, ValueValidity::Unusable),
                Some(0),
                Some(e.clone()),
                None,
            )?;
        }
        a.issue = Some(e);
    }
    Ok(a)
}
