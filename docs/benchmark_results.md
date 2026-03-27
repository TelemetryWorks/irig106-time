# Benchmark Results

## Benchmark Results (release build)

```
  Benchmark                           ns/iter      ops/sec
  ──────────────────────────────────────────────────────────
  rtc_from_le_bytes                     6.7 ns     148.5M/s
  rtc_from_raw                          0.3 ns    3090.0M/s
  rtc_to_nanos                          0.3 ns    2869.9M/s
  rtc_elapsed_ticks                     0.6 ns    1625.8M/s
  bcd_day_parse                         5.9 ns     168.2M/s
  bcd_dmy_parse                         7.8 ns     128.5M/s
  bcd_day_full_pipeline                12.4 ns      80.5M/s
  csdw_from_le_bytes                    0.3 ns    3043.6M/s
  csdw_all_fields                       1.7 ns     593.7M/s
  sec_checksum_validate                 1.9 ns     516.2M/s
  ipt_parse_rtc48                       3.0 ns     331.4M/s
  corr_100refs_any                     14.3 ns      69.9M/s
  corr_3600refs_any                    22.6 ns      44.3M/s
  HOT_rtc_to_absolute                  31.0 ns      32.2M/s
```

**Hot-path: 31 ns = 32.2M ops/sec.** At 10 Gbps / 512B packets = 2.4M pkt/sec → **13x headroom.**
