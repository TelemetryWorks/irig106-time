#![allow(dead_code)]
use irig106_time::{snapshot::*, timeline::*, *};
pub fn d<T: ExactInteger>(v: T) -> Decimal<T> {
    Decimal::new(v)
}
pub fn packet(id: u64, ticks: u64, body: ReferenceSpec) -> RecordedPacket {
    RecordedPacket {
        packet: ReplayPacket {
            id: d(id),
            rtc: d(ticks),
            channel: 7,
            kind: if matches!(body, ReferenceSpec::None) {
                PacketKind::Data
            } else {
                PacketKind::Time
            },
            declared_timein: true,
            declared_external: true,
            flags: 0,
            prepare: vec![],
            year_context: None,
            body,
        },
        boundary: false,
    }
}
pub fn limits() -> ReplayLimits {
    ReplayLimits {
        steps: d(100),
        answers: d(100),
        raw_bytes: d(10000),
    }
}
pub fn plan() -> ReplayPlan {
    ReplayPlan {
        limits: limits(),
        clock: ClockRecipe::Offline {
            policy: PolicySnapshot::offline(OfflinePolicy::default()).unwrap(),
            packets: vec![
                packet(
                    1,
                    0,
                    ReferenceSpec::Format1 {
                        payload: vec![0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
                        data_length: d(12),
                        layout: TimeF1Layout::SourceWithIts17,
                        scales: scale::ScalePolicy::default(),
                    },
                ),
                packet(2, 10, ReferenceSpec::None),
            ],
            quality: vec![],
        },
        requests: vec![ReportRequest::Counter {
            key: ReplayKey {
                session: d(1),
                ticks: d(10),
                packet: d(2),
            },
        }],
    }
}
pub fn context(id: u64) -> PolicySnapshot {
    use irig106_time::{own_time::*, policy::*, scale::*};
    let origin = format1::CalendarReading {
        year: Some(2017),
        day_of_year: 1,
        hours: 0,
        minutes: 0,
        seconds: 0,
        nanoseconds: 0,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    };
    let shared = SharedPolicy::new(
        id,
        SharedSettings {
            own: OwnContext {
                ieee: Some(EpochContext {
                    origin,
                    counting: EpochCounting::Continuous,
                    source: "caller epoch".into(),
                }),
                ..Default::default()
            },
            own_tolerance: Some(NanosDuration::new(17)),
            ..Default::default()
        },
    )
    .unwrap();
    PolicySnapshot::offline(shared.offline(OfflinePolicy::default()).unwrap()).unwrap()
}
pub fn origin() -> ReplayOrigin {
    ReplayOrigin {
        key: ReplayKey {
            session: d(1),
            ticks: d(10),
            packet: d(2),
        },
        channel: 7,
        byte_offset: Some(d(9007199254740993)),
        field: "explicit measurement".into(),
    }
}
pub fn calendar(sec: u8, nanos: u32) -> ReferenceSpec {
    use irig106_time::scale::*;
    ReferenceSpec::Calendar {
        value: CalendarSnapshot::new(format1::CalendarReading {
            year: Some(2017),
            day_of_year: 1,
            hours: 0,
            minutes: 0,
            seconds: sec,
            nanoseconds: nanos,
            scale: ScaleEvidence::standard(TimeScale::Utc),
        })
        .unwrap(),
        valid: true,
        source: Some(TimeF1Source::Source(TimeSource::External)),
        format: Some(TimeFormat::Utc),
        its: Some(IrigTimeSource::ExternalIrig),
        precision_ns: d(1),
        normalize: Some(LeapTableSnapshot::new(leap::LeapSecondTable::builtin())),
    }
}
