#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::bcd::DmyFormatTime;

fuzz_target!(|data: &[u8]| {
    match DmyFormatTime::from_le_bytes(data) {
        Ok(t) => {
            assert!(t.day >= 1 && t.day <= 31);
            assert!(t.month >= 1 && t.month <= 12);
            assert!(t.year <= 9999);
            assert!(t.hours <= 23);
            assert!(t.minutes <= 59);
            assert!(t.seconds <= 59);
            assert!(t.milliseconds <= 990);
            assert!(t.milliseconds % 10 == 0);

            let abs = t.to_absolute();
            assert!(abs.day_of_year >= 1 && abs.day_of_year <= 366);
            assert_eq!(abs.year, Some(t.year));
            assert_eq!(abs.month, Some(t.month));
        }
        Err(_) => {}
    }
});
