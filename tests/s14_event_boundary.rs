//! ADR-0009: caller extracts §11.2.7.3 event tags; time knows no event meaning.
use irig106_time::{
    extracted::{ExtractionOrigin, StampInput, inspect_stamp},
    rtc::CounterOrder,
    secondary_packet::SecondaryPolicy,
    timeline::PacketKey,
};

// Partial adapter-preservation evidence, not a whole API-105/ANS-103 marker.
#[test]
fn caller_extracted_event_tags_keep_their_time_adapter() {
    // §11.2.1.3b:48-bit RTC plus16 zero bits.42 ticks =>4200ns;
    // event numbers0..4095 are caller/setup data, not a fixed enum here.
    for event in [0, 1, 2, 3, 4095] {
        let origin = ExtractionOrigin {
            key: PacketKey {
                session: 1,
                packet: 7,
                ticks: 0,
            },
            channel: 8,
            byte_offset: Some(40),
            field: format!("caller event {event} time tag"),
        };
        let tag = inspect_stamp(
            StampInput {
                origin: origin.clone(),
                query_id: 10,
                flags: 0,
                bytes: &[42, 0, 0, 0, 0, 0, 0, 0],
                order: CounterOrder::Nearest,
            },
            &SecondaryPolicy::default(),
        )
        .unwrap();
        assert_eq!(tag.origin(), &origin);
        assert_eq!(tag.query().unwrap().ticks, 42);
        assert_eq!(tag.raw(), [42, 0, 0, 0, 0, 0, 0, 0]);
        assert!(tag.findings().is_empty());
    }
}
