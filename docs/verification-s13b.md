# S13b — validated Format 1 encoders

Originally based on59b7bf8; merged main9c81aca after Format2 PR#53. Source:106-24R1 §11.2.3.2,
Figure11-12 (printed11-23/PDF31), Figures11-13/14 (11-25/PDF33),
Table11-16 (11-26/PDF34). No new rendered-page audit is claimed.
Goldens hand-pack decimal digits into the figure slots, not through a decoder;
the contract7 day187 13:45:27.350 example is also pinned byte-for-byte.
Two high year bits are always zero: derived representable DMY year0..3999.

| Requirement | Covered here | Remaining |
|---|---|---|
| L2/L3-ENC-101 | Checked explicit-layout CSDW; DOY/DMY bodies; reserved positions; FMT/SRC/ITS; DATE/LY/scale; precision/year bounds; NONE; RDG-010 leap labels | None in the encoding statement |
| L2/L3-ENC-102 | Merged Format2 PR#53 evidence | None in that slice |
| L2/L3-ENC-103 | Existing IEEE word golden; invalid flags still reject despite valid encoded bytes | Complete header/stamp/checksum API remains deferred; no whole marker |

Tests-first:7c4a2fd has two tests on main: one fails at runtime because year4000
sets an always-zero bit, one body golden already passes. ccc71f8 adds checked
API tests before implementation (compile-first, not runtime failures).
Original focused checks: four S13b tests and20 rebuild-wire tests pass; migrated
pipeline encoding cases pass; fmt and trace check pass. CI owns the full matrix.
First CI44467905273 caught an unnecessary `as_raw()` on an already-raw u32
in a unit test; restored that unchanged raw-byte construction.
Thirteen independent temporary production mutations each compile and fail a
runtime assertion: FMT/SRC/ITS guards, reserved mask, both BCD precision guards,
year bound, NONE with time, LY consistency, lost second60, nanosecond precision,
partial-year leap day and stated scale mismatch. Sources restored afterwards.
Whole-marker aggregate covers ENC-101; after merging PR#53 the matrix is
196/288 (68.1%), delta2 against current main194/288. ENC summary has2
in each tested/verified column, totals98. No partial ENC-103 overclaim.

Recommendation: retain the owner-approved post0.8 deferral for the new complete
header/stamp encoder. Existing IEEE word writes exactly8 bytes and rejects
nanoseconds>=1e9; no flag writer exists here, so it cannot repair invalid flags.
No new full-header/checksum capability or host-derived calendar context added.

## PR54 review — tests only

Nine focused S13b tests pass. Removing `t.validate()` independently fails each
of the second61, non23:59 second60 and GPS second60 tests. Bypassing the
combined writer's canonical CSDW call with raw bytes fails the reserved-bit
golden (and a separate assertion rejects FMT6). Extending ITS to FMT3 fails
the literal0x3031 → [0x31,0,0,0] golden. All three temporary mutants compiled
and failed at runtime; production sources restored with no implementation edit.
Moved the DATE1/FMT4/DMY comment to its actual vector; the0x3001 vector now
states DATE0/FMT0/SRC1/ITS3 and Figure11-13. NeedsContext naming unchanged.
Ordinary main merge: retain both S13 CHANGELOG sections; regenerate the trace
matrix from merged markers, preserving ENC-101 and ENC-102 evidence.
