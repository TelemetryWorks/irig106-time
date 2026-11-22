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
    let admit = |o, k, session, p: &OfflinePolicy, raw| {
        admit_format2_reference(
            o,
            NtpContext::new(session),
            LeapSecondTable::builtin(),
            k,
            p,
            raw,
        )
    };
    let policy = OfflinePolicy {
        include_undeclared: true,
        ..Default::default()
    };
    let (r, findings) = admit(observation.clone(), key, 2, &policy, false).unwrap();
    assert_eq!(r.key, key);
    assert!(r.admitted);
    assert!(!r.declared_timein);
    assert!(r.reasons.contains(&Rejection::Undeclared));
    assert!(r.input.network.as_ref().unwrap().reference().is_eligible());
    assert!(findings.iter().all(|f| f.session == 2 && f.packet == 7));
    assert!(findings.iter().any(|f| f.code == 8));
    for bad in [
        PacketKey { session: 1, ..key },
        PacketKey { packet: 8, ..key },
        PacketKey { ticks: 124, ..key },
    ] {
        assert!(admit(observation.clone(), bad, bad.session, &policy, false).is_err());
    }
    assert!(admit(observation.clone(), key, 1, &policy, false).is_err());
    let wrapped = PacketKey {
        ticks: (1i128 << 48) + 123,
        ..key
    };
    let (r, _) = admit(observation.clone(), wrapped, 2, &policy, true).unwrap();
    assert_eq!(r.key, wrapped);
    assert!(r.input.network.as_ref().unwrap().raw_network_rate());
    let mut bad_policy = policy.clone();
    bad_policy.version = 0;
    assert!(admit(observation.clone(), key, 2, &bad_policy, false).is_err());
    bad_policy = policy.clone();
    bad_policy.shared = Some(Box::new(
        irig106_time::policy::SharedPolicy::new(1, Default::default()).unwrap(),
    ));
    assert!(admit(observation.clone(), key, 2, &bad_policy, false).is_err());
    let (r, _) = admit(observation, key, 2, &OfflinePolicy::default(), false).unwrap();
    assert!(!r.admitted);
}
