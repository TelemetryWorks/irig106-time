# S13b — validated Format 1 encoders

Base59b7bf8; independent of Format2 PR#53. Source:106-24R1 §11.2.3.2,
Figure11-12 (printed11-23/PDF31), Figures11-13/14 (11-25/PDF33),
Table11-16 (11-26/PDF34). No new rendered-page audit is claimed.
Goldens hand-pack decimal digits into the figure slots, not through a decoder;
the contract7 day187 13:45:27.350 example is also pinned byte-for-byte.
Two high year bits are always zero: derived representable DMY year0..3999.

| Requirement | Covered here | Remaining |
|---|---|---|
| L2/L3-ENC-101 | Checked explicit-layout CSDW; DOY/DMY bodies; reserved positions; FMT/SRC/ITS; DATE/LY/scale; precision/year bounds; NONE; RDG-010 leap labels | None in the encoding statement |
| L2/L3-ENC-102 | None (independent PR#53) | Format2 PR#53 |
| L2/L3-ENC-103 | Existing IEEE word golden; invalid flags still reject despite valid encoded bytes | Complete header/stamp/checksum API remains deferred; no whole marker |

Tests-first:7c4a2fd has two tests on main: one fails at runtime because year4000
sets an always-zero bit, one body golden already passes. ccc71f8 adds checked
API tests before implementation (compile-first, not runtime failures).
Final focused checks: four S13b tests and20 rebuild-wire tests pass; migrated
pipeline encoding cases pass; fmt and trace check pass. CI owns the full matrix.
Thirteen independent temporary production mutations each compile and fail a
runtime assertion: FMT/SRC/ITS guards, reserved mask, both BCD precision guards,
year bound, NONE with time, LY consistency, lost second60, nanosecond precision,
partial-year leap day and stated scale mismatch. Sources restored afterwards.
Whole-marker aggregate covers ENC-101; matrix192→194/288 on this independent
base, not counting PR#53's separate ENC-102 gain. No partial ENC-103 overclaim.

Recommendation: retain the owner-approved post0.8 deferral for the new complete
header/stamp encoder. Existing IEEE word writes exactly8 bytes and rejects
nanoseconds>=1e9; no flag writer exists here, so it cannot repair invalid flags.
No new full-header/checksum capability or host-derived calendar context added.
