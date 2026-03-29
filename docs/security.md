# Security Considerations — irig106-time

**Document:** SECURITY.md
**Crate:** irig106-time v0.5.0
**Date:** 2026-03-25

---

## 1. Threat Model

`irig106-time` parses binary data from sources that should be treated as
**untrusted input**:

| Source | Trust Level | Risk |
|--------|-----------|------|
| Ch10 files from own recorders | Medium | Could be corrupted by hardware failure |
| Ch10 files from third parties | Low | Could be intentionally malformed |
| Live UDP streams (Ch10 over IP) | Low | Network-adjacent attacker could inject packets |
| Archived data from data lakes | Low | Integrity may have degraded over storage lifetime |
| Synthetic/test data | Medium | Generator bugs create edge cases |

The crate processes this data in aerospace analysis pipelines where a parsing
crash could halt mission-critical post-flight analysis.

---

## 2. Security Properties

### 2.1 No Panics on Malformed Input

**Guarantee:** Every public function that accepts `&[u8]` returns `Result<T, TimeError>`.
No panic on any byte sequence.

**Verification:**
- 8 fuzz targets covering every parsing path (see `fuzz/`)
- 10,000-iteration property tests on random inputs (see `tests/properties.rs`)
- Explicit `BufferTooShort` checks before all slice indexing

### 2.2 No Unsafe Code

The crate uses zero `unsafe` blocks. All byte-level parsing uses safe Rust
methods (`u16::from_le_bytes`, bitwise ops, slice indexing with bounds checks).

### 2.3 No Integer Overflow

| Operation | Risk | Mitigation |
|-----------|------|-----------|
| `Rtc::to_nanos()` | 48-bit × 100 could overflow u64 | Max value = 2^48 × 100 = 2.81 × 10^16 < u64::MAX (1.84 × 10^19) ✓ |
| `Ertc::to_nanos()` | 64-bit × 100 overflows u64 | Returns `u128` |
| `elapsed_ticks()` | Subtraction could underflow | Uses `wrapping_sub` + 48-bit mask |
| `add_nanos()` | Large delta + midnight could overflow | Uses u64 intermediate, carries to days |
| BCD decoding | Multi-digit multiply | Max value per field < 1000; no overflow in u16 |

### 2.4 No Denial of Service

| Vector | Risk | Mitigation |
|--------|------|-----------|
| Huge correlation table | Memory exhaustion | `TimeCorrelator` accepts at most what the caller inserts; no amplification |
| Infinite loop on malformed BCD | CPU stall | All parsing is single-pass, bounded by buffer length |
| Checksum computation | Timing side-channel | Not relevant (checksum is not a secret) |

### 2.5 Validated Field Ranges

Every decoded value is checked against the IRIG 106 specification ranges before
being returned. A `TimeError::OutOfRange` is returned for:

- Hours > 23, minutes > 59, seconds > 59
- Day-of-year 0 or > 366, month 0 or > 12, day 0 or > 31
- IEEE-1588 nanoseconds >= 1,000,000,000
- BCD nibbles > 9

---

## 3. Fuzzing Guide

### 3.1 Setup

```sh
# Requires nightly Rust
rustup install nightly
cargo +nightly install cargo-fuzz
```

### 3.2 Running Fuzz Targets

```sh
# Run a specific target (recommended: start with BCD parsers)
cargo +nightly fuzz run fuzz_bcd_day
cargo +nightly fuzz run fuzz_bcd_dmy

# Run with a time limit
cargo +nightly fuzz run fuzz_secondary_header -- -max_total_time=300

# All available targets:
#   fuzz_bcd_day          — DayFormatTime::from_le_bytes
#   fuzz_bcd_dmy          — DmyFormatTime::from_le_bytes
#   fuzz_rtc              — Rtc construction and arithmetic
#   fuzz_secondary_header — Secondary header checksum + parsing
#   fuzz_intra_packet     — IntraPacketTime dispatch
#   fuzz_ieee1588         — Ieee1588Time parsing
#   fuzz_csdw             — CSDW field extraction
#   fuzz_correlation      — TimeCorrelator with random ref points
```

### 3.3 Corpus Seeding

For better coverage, seed the fuzz corpus with real Ch10 time packet payloads:

```sh
mkdir -p fuzz/corpus/fuzz_bcd_day
# Extract time packet payloads from real Ch10 files and place in corpus dir
```

### 3.4 What to Look For

- **Panics** — Any panic is a bug. File as a security issue.
- **Assertion failures in fuzz targets** — The fuzz targets assert post-conditions
  (e.g., parsed hours ≤ 23). A failure means the validation logic has a gap.
- **OOM / infinite allocation** — The correlation fuzz target caps at 100 refs.

---

## 4. Property-Based Testing Guide

```sh
# Run property tests (10,000 iterations each, no external deps)
cargo test --test properties

# The tests use a built-in PRNG (Xorshift) so they're deterministic
# and reproducible without proptest. When proptest is added as a
# dev-dependency, the tests can be upgraded to use its strategies
# for better coverage and shrinking.
```

### 4.1 Key Properties Verified

| Property | Why It Matters |
|----------|---------------|
| `from_raw` always produces ≤ 48-bit value | Prevents arithmetic overflow in elapsed_ticks |
| LE bytes round-trip | Data integrity through encode/decode |
| `elapsed_ticks` result ≤ 48 bits | No corruption in nanosecond conversion |
| `add_nanos` then `sub_nanos` = identity | Correlation engine correctness |
| `add_nanos` is monotonic | Time never goes backward unexpectedly |
| CSDW field extraction is idempotent | No hidden state corrupting repeated reads |

---

## 5. Recommendations for Downstream Consumers

1. **Always check `Result`** — Don't `.unwrap()` in production code. A malformed
   time packet should not crash the analysis pipeline.

2. **Validate before correlating** — Check that the `TimeCorrelator` has reference
   points before calling `.correlate()`. The `NoReferencePoint` error is expected
   for files that lack time packets.

3. **Monitor for time jumps** — Use `detect_time_jump()` to identify GPS lock events
   or clock resets. Don't silently interpolate across a 5-second jump.

4. **Don't trust secondary header checksums alone** — A valid checksum does not
   guarantee the time value is correct — only that the bytes are internally consistent.

5. **Prefer nearest reference** — The correlation engine already uses nearest-point
   lookup, but if you're building your own pipeline, don't use a single reference
   point for an entire file. RTC drift is real.
