# Coordinator handoff — recording events (T-8, ADR-0009)

For the irig106-decode maintainer; no changes made to that repository and no
replacement implementation/version is claimed. irig106-time0.8 removes its
prototype recording_event module and root RecordingEvent/RecordingEventType.

## Move the body boundary, not the prototype's meanings

106-24R1 Chapter11 §11.2.7.3 / Figure11-38:
"Event Number. Bits 11-0 represent an unsigned binary that identifies 4096
individual events types defined in the corresponding setup record recording
event number. The event number shall begin at 0x0 for the first event type
defined in the setup record" (T-8, docs/STANDARD-REVIEW.md).

Decode should own bounded event-body walking, event-number/count/state fields,
entry/tag byte offsets and setup-aware descriptions. Retain all12 event-number
bits including0 and4095. Bind definitions from the governing recording setup
via plain data supplied by the coordinator/caller; do not recreate a runtime
dependency on tmats inside time. The standard's setup-defined descriptions
replace the removed Started=1/Stopped=2/Overrun=3/IndexPoint table entirely.
Do not port from_index(u8), Display's invented labels, may_cause_time_gap(),
or has_reference_time() as an automatic time-reference admission rule.

## Time boundary retained here

- Caller/decoder extracts exactly the entry's time-tag bytes and packet flags,
  retaining packet/session/channel identity and the actual byte offset.
- `extracted::StampInput` / `ExtractionOrigin` / `inspect_stamp` validate and
  decode the time tag. Keep original raw event number/count/state separately.
- Resolve via `StampInspection::resolve` with explicit OwnContext and the
  caller's offline/stream counter-answer resolver. Retain basis and findings.
- Packet secondary own time uses `secondary_packet` / `own_time`; it is not
  an event-number-specific payload rule or an automatic reference source.
- Preserve Q-FLAGS: invalid secondary-derived flags yield no time; RTC stamps
  with reserved format value remain readable with their separate warning.
  Preserve explicit Chapter4 context, scale/session/era and leap-table evidence.

Suggested decode review fixtures: event0/1/2/3/4095 with setup-defined meanings
that differ from the prototype, missing/conflicting governing setup, truncated
entry/time tag within Data Length, RTC/ERTC/absolute tags and invalid flags.
Time's s14_event_boundary test preserves the generic extracted-tag path; it
claims no event-body or setup interpretation coverage. Coordinate any event
body source/page audit in decode before implementing its parser.
