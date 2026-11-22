```rust
use core::num::NonZeroUsize;
use irig106_time::{NanosDuration, StreamingTimeCorrelator, TimeCorrelator};
// Preserve an explicitly chosen old 2s window; do not inherit a changed default.
let correlator = TimeCorrelator::with_ooo_window(Some(NanosDuration::new(2_000_000_000)));
let jumps = correlator.detect_time_jump(7, NanosDuration::new(1_000_000_000));
let age = NanosDuration::new(30_000_000_000);
let age_only = StreamingTimeCorrelator::new(age);
let cap = NonZeroUsize::new(1024)
    .ok_or(irig106_time::TimeError::InvalidTimelineInput("example cap"))?;
let bounded = StreamingTimeCorrelator::with_reference_limit(age, cap);
```

```rust
use irig106_time::{
    LeapSecondEntry, LeapSecondTable, PtpTime, Result, TaiSeconds, TaiUtcOffset, UnixSeconds,
};
let table = LeapSecondTable::builtin();
// 2017-01-02 UTC, after the 2016 insertion: Unix1483315200; TAI+37s.
let offset = table.offset_at_tai(TaiSeconds::new(1_483_315_237))?;
let utc = UnixSeconds::new(1_483_315_200);
assert_eq!(table.offset_at_unix(utc)?, offset);
assert_eq!(table.offset_for_f1(2017, 2)?, offset);
assert!(!table.is_near_leap_second(utc, 0));
// Compile-check the checked mutator signature without inventing a new leap.
let _add: fn(&mut LeapSecondTable, LeapSecondEntry) -> Result<()> = LeapSecondTable::add;
let _typed_offset: TaiUtcOffset = offset;
let ptp = PtpTime {
    seconds: 1_483_315_237,
    nanoseconds: 0,
};
assert_eq!(ptp.to_utc_seconds(offset)?, utc.get()); // Explicit-offset arithmetic only.
let converted = table.convert_ptp(ptp)?; // Recording answer retains table/currency basis.
assert_eq!(
    converted.unix_seconds().map(|s| s.get()),
    Some(i128::from(utc.get()))
);
```
