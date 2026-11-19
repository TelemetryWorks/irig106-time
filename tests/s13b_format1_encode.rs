//! Independent106-24R1 §11.2.3.2a/b, Figures11-12..14 (11-23/25, PDF31/33).
use irig106_time::{TimeF1Csdw,bcd::{DayFormatTime,DmyFormatTime}};

#[test]
fn dmy_always_zero_year_bits_are_not_emitted() {
    // Fig11-14: year3999 =>0x3999; bits15..14 MUST be zero.
    let t=DmyFormatTime::new(3999,12,31,23,59,59,990).unwrap();
    assert_eq!(t.to_le_bytes(),[0x99,0x59,0x59,0x23,0x31,0x12,0x99,0x39]);
    // Existing writer must not set the reserved high-year position for4000.
    let t=DmyFormatTime::new(4000,1,1,0,0,0,0).unwrap();
    assert_eq!(t.to_le_bytes()[7]&0xc0,0);
}
#[test]
fn literal_bcd_slots_do_not_depend_on_a_decoder() {
    // §11.2.3.2b/Fig11-13: .340s=>Tmn4/Hmn3;25s,12h30m,DOY100.
    assert_eq!(DayFormatTime::new(100,12,30,25,340).unwrap().to_le_bytes(),
        [0x34,0x25,0x30,0x12,0,1]);
    // Fig11-14: Apr10=>0x0410; year2025=>0x2025.
    assert_eq!(DmyFormatTime::new(2025,4,10,12,30,25,340).unwrap().to_le_bytes(),
        [0x34,0x25,0x30,0x12,0x10,4,0x25,0x20]);
    assert_eq!(TimeF1Csdw::from_raw(0x241).as_raw(),0x241);
}
