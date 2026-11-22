//! Standalone boundary clauses only; no whole-requirement marker.
use irig106_time::{
    format2::*, ntp::NtpContext, reference::admit_format2_reference, timeline::*, *,
};
#[test]
fn standalone_network_boundary_keeps_real_session_and_checks_scope() {
    // Fig11-15/17 NTF2/TS1, TAI2017Jan2 00:00:37=1483315237s.
    let mut b = [0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    b[4..8].copy_from_slice(&1_483_315_237u32.to_le_bytes());
    let key = PacketKey {
        session: 2,
        packet: 7,
        ticks: 123,
    };
    let observation = Format2Input {
        payload: &b,
        data_length: 12,
        capture: ReferenceCapture {
            session: 2,
            packet: 7,
            channel: 4,
            rtc: Rtc::from_raw(123),
            byte_offset: Some(100),
        },
        declared_edition: None,
        data_type_version: 8,
        declared_timein: false,
    }
    .decode()
    .unwrap();
    let policy = OfflinePolicy {
        include_undeclared: true,
        ..Default::default()
    };
    let (r, findings) = admit_format2_reference(
        observation.clone(),
        NtpContext::new(2),
        LeapSecondTable::builtin(),
        key,
        &policy,
        false,
    )
    .unwrap();
    assert_eq!(r.key, key);
    assert!(r.admitted);
    assert!(r.input.network.as_ref().unwrap().reference().is_eligible());
    assert!(findings.iter().all(|f| f.session == 2 && f.packet == 7));
    assert!(findings.iter().any(|f| f.code == 8));
    for bad in [
        PacketKey { session: 1, ..key },
        PacketKey { packet: 8, ..key },
        PacketKey { ticks: 124, ..key },
    ] {
        assert!(
            admit_format2_reference(
                observation.clone(),
                NtpContext::new(2),
                LeapSecondTable::builtin(),
                bad,
                &policy,
                false
            )
            .is_err()
        );
    }
    assert!(
        admit_format2_reference(
            observation.clone(),
            NtpContext::new(1),
            LeapSecondTable::builtin(),
            key,
            &policy,
            false
        )
        .is_err()
    );
    let (r, _) = admit_format2_reference(
        observation,
        NtpContext::new(2),
        LeapSecondTable::builtin(),
        key,
        &OfflinePolicy::default(),
        false,
    )
    .unwrap();
    assert!(!r.admitted);
}
