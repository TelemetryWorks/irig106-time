# S14b — retired conversion surface and migration audit

Base74fc668, independent of event-boundary PR55 (ADR-0009). No new calendar
or Jiff conversion algorithm; no library/dependency/finding-level change.

## Export inventory

| Surface | 0.8 action | Migration / retained boundary |
|---|---|---|
| root and absolute::Ch4BinaryTimeExt; trait to_absolute | Removed (breaking); 17-bit time/day prototype split was already withdrawn by S07, not repaired here | Ch4Words::from(raw).binary(None) gives labelled elapsed partial; binary(Some(origin)) explicit nominal calendar; bcd(BcdContext) explicit year/full day/wording. Never assume day1/year/origin or nearest packet |
| NtpTime::to_absolute / to_unix_seconds / to_nanos_since_ntp_epoch | Compiler-deprecated; still return the same NeedsContext, not a new fallback | ntp::convert_ntp with session evidence/table, retaining raw count, era, basis and findings; raw_fixed_point_nanos only for explicitly modular words |
| PtpTime::to_absolute(offset) | Compiler-deprecated; unchanged scalar compatibility result | LeapSecondTable::convert_ptp retains raw TAI, second60, covered/stale/missing status; D7/TF-021 last-known-offset rule applies |
| DayFormatTime::to_absolute / From<DayFormatTime>; DmyFormatTime::to_calendar_time | Kept as explicitly warned unqualified body projections; cannot establish layout/scale/CSDW validity or retain leap labels | format1::decode_qualified_time_f1 (edition evidence) or decode_time_f1 (caller-selected layout); preserve CalendarReading/evidence |
| AbsoluteTime / CalendarTime; CalendarReading::to_absolute | Kept ordinary-calendar value/projection; no automatic UTC/year; reading projection stays checked and scale-wrapped | Preserve original reading/basis; ordinary calendar cannot represent second60 |
| TimeCorrelator / StreamingTimeCorrelator | Kept explicitly labelled legacy unqualified compatibility engines, not recommended recording paths | timeline/stream plus shared policy and AnswerView; no inference of session/source evidence from ordinary calendars |
| PtpTime::to_signed_utc_seconds / to_utc_seconds; tai_seconds/raw nanos | Kept checked explicit caller-offset arithmetic; unsigned pre1970 refuses | Not a table/historical UTC model. Caller supplies offset and retains basis; DEFAULT_TAI_UTC_OFFSET is warned historical, not an automatic policy default |
| Shared Ch4BinaryTime/Ieee1588Time/Ertc, raw parsers/encoders | Kept; shared types untouched | Raw units/word layout do not themselves establish calendar origin/scale |
| Jiff checked civil conversions | Unchanged; S14a/PR55 audits all13 existing exit cases | verification-jiff.md: known/unknown/non-UTC/leap exit semantics; no old chrono implementation restored |

Source boundary: Chapter4 §4.7, Ch11 Figure11-4, Q-CH4/D6; RDG-010;
RDG-012 and D7 for qualified network conversion; ADR-0010/0011/0020.
Audit finds no remaining17-bit count/day split in library code. Remaining
ordinary projections are explicitly unqualified, not recommended as answers.
Historical S07 narrative is retained with a subsequent retirement note.
README quick start now uses known-layout FMT4 UTC, preserving unknown year;
usage's Chapter4 and Jiff sections no longer recommend removed APIs. Remaining
prototype usage examples are labelled historical, not0.8 recommendations.

## Ordered checks and mutations

d481eeb adds six failing import/call compile-fail tests before changes.
a049c56 distinguishes two removed imports from four retained-but-deprecated
methods using deny(deprecated); all six still fail before implementation
because they compile successfully. These are compile-boundary failures, not
claims of runtime conversion defects on main. No NeedsContext error renaming.
After retirement the focused doctests pass. Existing tests intentionally
exercise deprecated helpers under scoped compatibility-test allow attributes;
production legacy NTP branches retain their identical explicit NeedsContext.
Fuzz/bench paths use the already-implemented qualified converters instead of
the retired calendar shortcuts; CI compiles them, no local full rebuild.
S07 controls migrate to Ch4Words directly: no origin stays partial, microseconds
10000 rejects. Focused s07_existing(2), s05_existing(3), slice2_wire(9) pass.
Five temporary mutations each fail a compile-fail assertion: restore both
Ch4 exports, remove each of the four network-helper deprecations individually.
Each compiled the library then failed because the forbidden example compiled;
no compiler/syntax failure counted as a kill. Sources restored afterwards.

L1-API-005 → L2/L3-API-105 dependency clause remains as audited in PR55;
no manifest edge added. L1-API-011 → L2/L3-API-111 Jiff remains unchanged.
Retirement controls are partial compatibility evidence, not whole markers.
Matrix remains196/288. Fmt and trace check locally; full targets/CLI/benches/
fuzz/deny/MSRV/WASM in PR CI under owner compute rule2026-11-18.
