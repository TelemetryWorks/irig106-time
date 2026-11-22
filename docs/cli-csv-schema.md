# Packet CSV columns (S17b1)

This documents the existing `irigtime csv` packet projection. The header names
and order below are the column schema; each row has exactly 15 fields. The
counter-only row's previously missing final field is corrected in 0.8.0.
CSV is its own machine format, separate from the stable v1 JSON envelope.
`csv --format json` is unsupported (usage exit 2); use `timeline --format json`
for packet JSON or `correlate --format json` for counter queries. L1-CLI-006 and
L2/L3-CLI-106 require the three output formats and consistent report data, not
JSON wrapping of CSV. This follows the coordinator's conditional S17b instruction;
it adds no renderer or dependency and does not weaken protocol/basis obligations.

| Column | Representation and unit | Empty meaning |
| --- | --- | --- |
| packet_num | Decimal, zero-based packet index in file order | Never |
| offset_hex | `0x` hexadecimal byte offset of the header | Never |
| channel_id | Decimal unsigned 16-bit channel ID | Never |
| data_type_hex | `0x` two-digit hexadecimal packet data type | Never |
| data_type_name | Short inventory label, e.g. `Time-F1` | Never |
| rtc_raw | Decimal 48-bit header RTC, 100 ns ticks | Never |
| rtc_nanos | Decimal RTC count multiplied by 100, ns | Never |
| day_of_year | Decimal ordinal day, 1–366 | No resolved calendar value |
| hours | Decimal hour, 0–23 | No resolved calendar value |
| minutes | Decimal minute, 0–59 | No resolved calendar value |
| seconds | Decimal second, including permitted label 60 | No resolved calendar value |
| nanoseconds | Decimal fraction of second, ns | No resolved calendar value |
| year | Decimal calendar year | Year unknown, or no resolved calendar value |
| month | Reserved legacy column, always empty | Not projected |
| day | Reserved legacy column, always empty | Not projected |

Zero is a value, never a substitute for absent time. A time with unknown year
still has its day/time fields; a counter-only row leaves all eight calendar
fields empty. Timeline displays the same observation as `N/A`. Neither renderer
invents a year, origin, data-to-time-channel association or segment selection.
The current CSV lacks scale/basis columns: consumers must not interpret every
resolved clock as UTC. Protocol/scale/status and complete qualified provenance remain full-adapter
work; this document does not claim those clauses are complete.

Output is header-present, comma-delimited ASCII with LF record terminators and
no BOM. Field quoting follows [RFC 4180 §2](https://www.rfc-editor.org/rfc/rfc4180.html#section-2)
items 5–7: separators, CR/LF or quotes require surrounding double quotes;
embedded double quotes are doubled. LF rather than RFC 4180's CRLF preserves
the existing CLI dialect. Current data-type labels contain no such characters;
the shared field writer is tested with comma/quote/CR/LF values as well.
Numeric values are exact text; consumers must avoid floating-point conversion
for large RTC/nanosecond counts. File names are not CSV columns.

All rows pass through the same bounded packet observation model as timeline.
CSV spools rows as they are analysed; timeline retains only `--limit` rows.
The established atomic publication, cleanup, permissions and exit precedence
remain unchanged. Findings still go to stderr; empty calendar fields alone do
not specify the finding or its severity.


PTP readings beyond the table's declared currency retain UTC using the last known
offset, with a TF-021 warning that labels the UTC value possibly out of date and
keeps raw TAI operands, applied offset and table identity in sampled diagnostics.
The warning is packet/channel-located in JSON findings. Summary text renders its
diagnostics on stdout; CSV and other text commands use stderr. Sampling stays
bounded with exact counts; this does not promise one raw-evidence record per CSV
row. Never infer UTC currency from the mere presence of calendar fields.
