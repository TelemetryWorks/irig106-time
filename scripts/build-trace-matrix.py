#!/usr/bin/env python3
"""Regenerate docs/TRACE-MATRIX.md from requirement sources and test markers.

Adapted from ``irig106-tmats``'s ``scripts/build-trace-matrix.py`` (itself
adapted from mie-decoder's). The document formats, marker convention, and
status rollup are the same; the file names, categories, and the reading
register's ``RDG-NNN`` identifiers are this repository's.

Sources:

1. ``docs/L1_Requirements.md`` — L1 ids (``### L1-XXX-NNN`` headings), their declared
   verification methods, and optional ``**Evidence**`` lines
2. ``docs/L2_Requirements.md`` — L2 ids (``#### L2-XXX-NNN``) with ``**Parent**:`` lines
3. ``docs/L3_Requirements.md`` — L3 one-liners
   ``**L3-XXX-NNN** · Parent: L2-XXX-NNN · Verification: T[ · Evidence: ...]``
4. ``docs/INTERPRETATIONS.md`` — the reading register
   (``### RDG-NNN`` headings with ``**Title**`` and ``**Design**`` lines)
5. Rust sources of both crates (``src/``, ``tests/``,
   ``irig106-time-cli/src/``, ``irig106-time-cli/tests/``) — every
   ``/// Requirements: ...`` or ``/// Interpretations: ...`` doc-comment line
   directly preceding a ``#[test]`` item, collected by a stateful line scan
   and emitted as ``path::name``

The coverage denominator is every L2 and L3 requirement plus the
Test-verifiable L1 *leaves* (L1s with no L2 decomposition, where markers
attach directly). Composite L1s are verified through their children and are
not counted twice.

Usage:
    python scripts/build-trace-matrix.py            # regenerate in place
    python scripts/build-trace-matrix.py --check    # fail if output drifted
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
L1_DOC = ROOT / "docs" / "L1_Requirements.md"
L2_DOC = ROOT / "docs" / "L2_Requirements.md"
L3_DOC = ROOT / "docs" / "L3_Requirements.md"
INT_DOC = ROOT / "docs" / "INTERPRETATIONS.md"
TRACE_DOC = ROOT / "docs" / "TRACE-MATRIX.md"
RUST_SOURCE_ROOTS = [
    ROOT / "src",
    ROOT / "tests",
    ROOT / "irig106-time-cli" / "src",
    ROOT / "irig106-time-cli" / "tests",
]

REQ_ID_PATTERN = re.compile(r"L(?P<level>[123])-(?P<cat>[A-Z0-9]+)-(?P<num>\d+)")
INT_ID_PATTERN = re.compile(r"RDG-\d{3}")
INT_TITLE = re.compile(r"^\*\*Title\*\*:\s+([^\n]+)$", re.MULTILINE)
INT_DESIGN = re.compile(r"^\*\*Design\*\*:\s+([^·\n]+)", re.MULTILINE)
INT_DEVELOPMENT = re.compile(r"^\*\*Development\*\*:\s+([^·\n]+)", re.MULTILINE)
INT_ANALYSIS = re.compile(r"\*\*Analysis\*\*:\s+([^·\n]+)")
L1_HEADER = re.compile(r"^###\s+(L1-[A-Z0-9]+-\d+)\s*$", re.MULTILINE)
L2_PARENT_LINE = re.compile(r"^\*\*Parent\*\*:\s+(L1-[A-Z0-9]+-\d+)\s*$", re.MULTILINE)
L3_LINE = re.compile(
    r"^\*\*L3-([A-Z0-9]+)-(\d+)\*\*\s+·\s+Parent:\s+(L2-[A-Z0-9]+-\d+)\s+·\s+Verification:\s+([^\n·]+)"
    r"(?:·\s+Evidence:\s+([^\n]+))?",
    re.MULTILINE,
)
VM_LINE = re.compile(r"^\*\*Verification Method\*\*:\s+([^\n]+)$", re.MULTILINE)
# Optional companion to Verification Method for Inspection / Analysis /
# Demonstration: names the artifact that carries the check. A declared method
# without evidence is a plan, not a result, and the requirement stays Draft.
EVIDENCE_LINE = re.compile(r"^\*\*Evidence\*\*:\s+([^\n]+)$", re.MULTILINE)
_BACKTICKED = re.compile(r"`([^`]+)`")
_METHOD_LETTER = re.compile(r"\b([TIAD])\b")

# Categories in declaration order (docs/L1_Requirements.md). L2/L3 requirements appear
# under the category of their parent L1. LIB is an L3-only category for
# library-only obligations; CLI serves both as an L1 category and as the
# L3 category for CLI-only obligations.
CATEGORIES: list[tuple[str, str]] = [
    ("RTC", "The relative time counter"),
    ("ABS", "Absolute time values"),
    ("IPT", "Packet flags and intra-packet time stamps"),
    ("SEC", "The secondary header"),
    ("CSDW", "Format 1 channel-specific data word"),
    ("BCD", "Format 1 time bodies"),
    ("F2CSDW", "Format 2 channel-specific data word"),
    ("NTP", "Format 2 NTP time"),
    ("PTP", "Format 2 PTP time"),
    ("F2COR", "Format 2 references"),
    ("TAI", "Time scales and leap seconds"),
    ("WORD", "Time words inside data"),
    ("ENC", "Encoding"),
    ("EDN", "Editions"),
    ("DECL", "Time declarations from the setup record"),
    ("COR", "Correlation over a recording (the time timeline)"),
    ("ANS", "Answers and their basis"),
    ("POL", "The time policy"),
    ("FND", "Findings"),
    ("RDG", "Readings between TMATS and the packets"),
    ("STRM", "Streaming"),
    ("ERR", "Errors and robustness"),
    ("API", "Library form and portability"),
    ("PERF", "Performance"),
    ("CLI", "The command-line crate"),
    ("REL", "Versioning and release"),
    ("TST", "Verification"),
    ("LIB", "Library implementation details (L3)"),
]


def _blocks(doc: str, heading: str) -> list[tuple[str, str]]:
    """Split a document into (id, body) pairs at headings of the given level."""
    parts = re.split(rf"^{heading}\s+(L[12]-[A-Z0-9]+-\d+)\s*$", doc, flags=re.MULTILINE)
    return [(parts[i], parts[i + 1] if i + 1 < len(parts) else "") for i in range(1, len(parts), 2)]


def _methods(text: str) -> set[str]:
    return set(_METHOD_LETTER.findall(text))


def _evidence(text: str) -> list[str]:
    found = _BACKTICKED.findall(text)
    if found:
        return found
    stripped = text.strip()
    return [stripped] if stripped else []


def parse_l1(doc: str) -> tuple[list[str], dict[str, set[str]], dict[str, list[str]]]:
    ids = L1_HEADER.findall(doc)
    methods: dict[str, set[str]] = {}
    evidence: dict[str, list[str]] = {}
    for req, body in _blocks(doc, "###"):
        if m := VM_LINE.search(body):
            methods[req] = _methods(m.group(1))
        if m := EVIDENCE_LINE.search(body):
            evidence[req] = _evidence(m.group(1))
    return ids, methods, evidence


def parse_l2(doc: str) -> tuple[dict[str, str], dict[str, set[str]], dict[str, list[str]]]:
    parent: dict[str, str] = {}
    methods: dict[str, set[str]] = {}
    evidence: dict[str, list[str]] = {}
    for req, body in _blocks(doc, "####"):
        if m := L2_PARENT_LINE.search(body):
            parent[req] = m.group(1)
        if m := VM_LINE.search(body):
            methods[req] = _methods(m.group(1))
        if m := EVIDENCE_LINE.search(body):
            evidence[req] = _evidence(m.group(1))
    return parent, methods, evidence


def parse_l3(doc: str) -> tuple[dict[str, str], dict[str, set[str]], dict[str, list[str]]]:
    parent: dict[str, str] = {}
    methods: dict[str, set[str]] = {}
    evidence: dict[str, list[str]] = {}
    for match in L3_LINE.finditer(doc):
        cat, num, par, verification, ev = match.groups()
        req = f"L3-{cat}-{num}"
        parent[req] = par
        methods[req] = _methods(verification)
        if ev:
            evidence[req] = _evidence(ev)
    return parent, methods, evidence


def parse_interpretations(doc: str) -> list[tuple[str, str, str, str, str]]:
    """Return (id, title, design status, development mark, analysis) for each
    register entry, in order. An empty analysis or ``not yet written`` means
    the entry's written analysis does not exist yet."""
    parts = re.split(r"^###\s+(RDG-\d{3})\s*$", doc, flags=re.MULTILINE)
    entries = []
    for i in range(1, len(parts), 2):
        body = parts[i + 1] if i + 1 < len(parts) else ""
        title = m.group(1).strip() if (m := INT_TITLE.search(body)) else ""
        design = m.group(1).replace("*", "").strip() if (m := INT_DESIGN.search(body)) else ""
        development = m.group(1).strip() if (m := INT_DEVELOPMENT.search(body)) else ""
        analysis = m.group(1).strip() if (m := INT_ANALYSIS.search(body)) else ""
        entries.append((parts[i], title, design, development, analysis))
    return entries


def _analysis_written(analysis: str) -> bool:
    return bool(analysis) and analysis.lower() != "not yet written"


_FN_DECL = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*[(<]")


def collect_rust_markers(source_roots: list[Path]) -> dict[str, list[str]]:
    """Collect ``/// Requirements:`` and ``/// Interpretations:`` markers that
    precede ``#[test]`` items.

    A marker pairs with the next ``fn name(`` declaration only if a test
    attribute appears between them; any other code line resets the pending
    state, so a marker on a non-test function is ignored. Multi-line
    attributes between the marker and the function are skipped rather than
    treated as code, so they cannot silently drop a trace link.
    """
    marker_map: dict[str, list[str]] = defaultdict(list)
    for source_root in source_roots:
        if not source_root.is_dir():
            continue
        for rs_file in sorted(source_root.rglob("*.rs")):
            try:
                text = rs_file.read_text(encoding="utf-8")
            except OSError:
                continue
            rel = rs_file.relative_to(ROOT).as_posix()
            pending: list[str] = []
            saw_test = False
            attr_depth = 0
            for line in text.splitlines():
                stripped = line.strip()
                if attr_depth > 0:
                    attr_depth += stripped.count("[") - stripped.count("]")
                    continue
                if stripped.startswith("///") and "Requirements:" in stripped:
                    _, _, after = stripped.partition("Requirements:")
                    for m in REQ_ID_PATTERN.finditer(after):
                        pending.append(f"L{m.group('level')}-{m.group('cat')}-{m.group('num')}")
                    continue
                if stripped.startswith("///") and "Interpretations:" in stripped:
                    _, _, after = stripped.partition("Interpretations:")
                    pending.extend(INT_ID_PATTERN.findall(after))
                    continue
                if stripped.startswith("#["):
                    if stripped.startswith(("#[test", "#[tokio::test", "#[rstest")) or "::test]" in stripped:
                        saw_test = True
                    attr_depth = stripped.count("[") - stripped.count("]")
                    continue
                if stripped.startswith("//") or not stripped:
                    continue
                fn = _FN_DECL.match(line)
                if fn and saw_test and pending:
                    for req in pending:
                        marker_map[req].append(f"{rel}::{fn.group(1)}")
                pending = []
                saw_test = False
    for req in marker_map:
        marker_map[req] = sorted(set(marker_map[req]))
    return marker_map


def _sort_key(req_id: str) -> tuple[int, str, int]:
    m = REQ_ID_PATTERN.search(req_id)
    if not m:
        return (9, req_id, 0)
    order = {code: i for i, (code, _) in enumerate(CATEGORIES)}
    return (order.get(m.group("cat"), len(order)), m.group("cat"), int(m.group("num")))


def compute_status(
    *,
    has_direct_artifacts: bool,
    children_statuses: list[str],
    verification_methods: set[str] | None = None,
    evidence: list[str] | None = None,
) -> str:
    """Roll up status for one requirement node (same rule as mie-decoder).

    A leaf with no test marker that declares only Inspection / Analysis /
    Demonstration reaches ``Implemented (I)`` etc. only if it also names its
    evidence; a declared method alone is a plan and stays ``Draft``.
    """

    def _non_test_status() -> str | None:
        if not verification_methods or "T" in verification_methods or not evidence:
            return None
        return f"Implemented ({'+'.join(sorted(verification_methods))})"

    if not children_statuses:
        if has_direct_artifacts:
            return "Implemented"
        return _non_test_status() or "Draft"
    n = len(children_statuses)
    impl_count = sum(1 for s in children_statuses if s.startswith("Implemented"))
    draft_count = sum(1 for s in children_statuses if s == "Draft")
    if impl_count == n:
        return "Implemented"
    if draft_count == n and not has_direct_artifacts:
        return _non_test_status() or "Draft"
    return "Partially Implemented"


def build_matrix() -> str:
    l1_ids, l1_methods, l1_evidence = parse_l1(L1_DOC.read_text(encoding="utf-8"))
    l2_parent, l2_methods, l2_evidence = parse_l2(L2_DOC.read_text(encoding="utf-8"))
    l3_parent, l3_methods, l3_evidence = parse_l3(L3_DOC.read_text(encoding="utf-8"))
    markers = collect_rust_markers(RUST_SOURCE_ROOTS)
    interpretations = (
        parse_interpretations(INT_DOC.read_text(encoding="utf-8")) if INT_DOC.is_file() else []
    )

    l1_to_l2: dict[str, list[str]] = defaultdict(list)
    for l2, l1 in l2_parent.items():
        l1_to_l2[l1].append(l2)
    l2_to_l3: dict[str, list[str]] = defaultdict(list)
    for l3, l2 in l3_parent.items():
        l2_to_l3[l2].append(l3)
    for children in list(l1_to_l2.values()) + list(l2_to_l3.values()):
        children.sort(key=_sort_key)

    def l3_status(l3: str) -> str:
        return compute_status(
            has_direct_artifacts=bool(markers.get(l3)),
            children_statuses=[],
            verification_methods=l3_methods.get(l3),
            evidence=l3_evidence.get(l3),
        )

    def l2_status(l2: str) -> str:
        return compute_status(
            has_direct_artifacts=bool(markers.get(l2)),
            children_statuses=[l3_status(l3) for l3 in l2_to_l3.get(l2, [])],
            verification_methods=l2_methods.get(l2),
            evidence=l2_evidence.get(l2),
        )

    def l1_status(l1: str) -> str:
        return compute_status(
            has_direct_artifacts=bool(markers.get(l1)),
            children_statuses=[l2_status(l2) for l2 in l1_to_l2.get(l1, [])],
            verification_methods=l1_methods.get(l1),
            evidence=l1_evidence.get(l1),
        )

    def artifacts_cell(items: list[str]) -> str:
        return "<br>".join(f"`{a}`" for a in items) if items else "_(none)_"

    out: list[str] = [
        "# irig106-time — Requirements Trace Matrix",
        "",
        "<!-- AUTO-GENERATED by scripts/build-trace-matrix.py. Do not edit by hand. -->",
        "",
        "## Purpose",
        "",
        "Forward trace from L1 through L2 and L3 to verification artifacts. This file is "
        "regenerated from `L1_Requirements.md`, `L2_Requirements.md`, `L3_Requirements.md`, and the `/// Requirements:` "
        "doc-comment tags above `#[test]` items in both crates' Rust sources, each time "
        "`scripts/build-trace-matrix.py` is run.",
        "",
        "## Status rollup",
        "",
        "This matrix is the single source of truth for live status; the requirement "
        "documents carry only spec content.",
        "",
        "* **Draft** — Test verification is required but no test marker found, or a "
        "non-Test method is declared without evidence.",
        "* **Implemented** — at least one test marker exists (leaf), or every child rolls "
        "up to Implemented.",
        "* **Implemented (I)** / **(A)** / **(D)** — verification by Inspection / Analysis / "
        "Demonstration only, with the artifact that carries the check named on an "
        "`**Evidence**` line (shown in the artifact column).",
        "* **Partially Implemented** — some children are Implemented and others are Draft, or "
        "the row has direct artifacts but children that are Draft.",
        "",
        "---",
        "",
    ]

    for code, title in CATEGORIES:
        cat_l1s = [r for r in l1_ids if r.startswith(f"L1-{code}-")]
        if not cat_l1s:
            continue
        out += [f"### L1-{code}: {title}", "", "**L1 -> L2**", "",
                "| L1 ID | L2 Children | Test Artifacts | Status |",
                "|-------|-------------|----------------|--------|"]
        for l1 in cat_l1s:
            children = l1_to_l2.get(l1, [])
            direct = markers.get(l1, []) or (l1_evidence.get(l1, []) if not children else [])
            out.append(
                f"| {l1} | {', '.join(children) if children else '_(none)_'} | "
                f"{artifacts_cell(direct)} | {l1_status(l1)} |"
            )
        out.append("")
        cat_l2s = sorted((l2 for l2, p in l2_parent.items() if p in set(cat_l1s)), key=_sort_key)
        if cat_l2s:
            out += ["**L2 -> L3 -> Verification Artifacts**", "",
                    "| L2 ID | L3 Children | Test Artifacts | Status |",
                    "|-------|-------------|----------------|--------|"]
            for l2 in cat_l2s:
                l3s = l2_to_l3.get(l2, [])
                arts = sorted(set(markers.get(l2, []) + [a for l3 in l3s for a in markers.get(l3, [])]))
                if not arts:
                    arts = sorted(set(l2_evidence.get(l2, [])) | {e for l3 in l3s for e in l3_evidence.get(l3, [])})
                out.append(
                    f"| {l2} | {', '.join(l3s) if l3s else '_(none)_'} | "
                    f"{artifacts_cell(arts) if arts else '_(TBD)_'} | {l2_status(l2)} |"
                )
            out.append("")

    def verified(req: str, methods: dict[str, set[str]], evidence: dict[str, list[str]]) -> bool:
        if markers.get(req):
            return True
        m = methods.get(req, set())
        return bool(m) and "T" not in m and bool(evidence.get(req))

    out += ["---", "", "## Coverage summary", "",
            "* **Tested** — at least one `/// Requirements:` test marker names this requirement.",
            "* **Verified** — Tested, or verified by Inspection / Analysis / Demonstration with "
            "named evidence.", "",
            "| Category | L1 | L2 | L3 | L2 tested | L3 tested | L2 verified | L3 verified |",
            "|----------|----|----|----|-----------|-----------|-------------|-------------|"]
    totals = [0] * 7
    for code, _ in CATEGORIES:
        l1s = [r for r in l1_ids if r.startswith(f"L1-{code}-")]
        l2s = [r for r in l2_parent if r.startswith(f"L2-{code}-")]
        l3s = [r for r in l3_parent if r.startswith(f"L3-{code}-")]
        if not (l1s or l2s or l3s):
            continue
        row = [
            len(l1s), len(l2s), len(l3s),
            sum(1 for r in l2s if markers.get(r)),
            sum(1 for r in l3s if markers.get(r)),
            sum(1 for r in l2s if verified(r, l2_methods, l2_evidence)),
            sum(1 for r in l3s if verified(r, l3_methods, l3_evidence)),
        ]
        totals = [a + b for a, b in zip(totals, row)]
        out.append(f"| {code} | " + " | ".join(str(v) for v in row) + " |")
    out.append("| **Total** | " + " | ".join(f"**{v}**" for v in totals) + " |")
    out.append("")

    leaves = [r for r in l1_ids if not l1_to_l2.get(r)]
    countable = totals[1] + totals[2] + len(leaves)
    if countable:
        tested = totals[3] + totals[4] + sum(1 for r in leaves if markers.get(r))
        ver = totals[5] + totals[6] + sum(1 for r in leaves if verified(r, l1_methods, l1_evidence))
        out += [
            f"The countable requirement set is every L2 and L3 requirement plus the "
            f"{len(leaves)} L1 *leaf* requirement(s) (L1s with no L2 decomposition yet). "
            f"Composite L1s are verified through their children.",
            "",
            f"**Tested by at least one test marker**: {tested} of {countable} "
            f"({tested * 100 / countable:.1f}%).",
            "",
            f"**Verified (Test or evidenced Inspection/Analysis/Demonstration)**: {ver} of "
            f"{countable} ({ver * 100 / countable:.1f}%).",
            "",
        ]

    orphan_l2 = sorted((r for r in l2_parent if l2_parent[r] not in l1_ids), key=_sort_key)
    orphan_l3 = sorted((r for r in l3_parent if l3_parent[r] not in l2_parent), key=_sort_key)
    out += ["### Orphan check", "",
            f"* Orphan L2s (parent L1 not found): **{len(orphan_l2)}**",
            f"* Orphan L3s (parent L2 not found): **{len(orphan_l3)}**"]
    out += [f"* {r} -> parent {l2_parent[r]} not in L1_Requirements.md" for r in orphan_l2]
    out += [f"* {r} -> parent {l3_parent[r]} not in L2_Requirements.md" for r in orphan_l3]
    out.append("")

    untested = [e[0] for e in interpretations if not markers.get(e[0])]
    unanalysed = [e[0] for e in interpretations if not _analysis_written(e[4])]
    unmarked = [e[0] for e in interpretations if not e[3]]
    out += ["### Interpretation register", "",
            "Every entry of `docs/INTERPRETATIONS.md` with its written analysis and the tests "
            "that name it (`/// Interpretations: RDG-NNN`). Every entry requires intensive "
            "testing and deep analysis during development (the register's conventions); an "
            "entry missing either is not ready.", "",
            "| Entry | Title | Design | Analysis | Tests |",
            "|-------|-------|--------|----------|-------|"]
    for int_id, title, design, _, analysis in interpretations:
        out.append(
            f"| {int_id} | {title} | {design} | {analysis or '_(missing)_'} | "
            f"{artifacts_cell(markers.get(int_id, []))} |"
        )
    out += ["",
            f"* Entries without a written analysis: **{len(unanalysed)}** of {len(interpretations)}",
            f"* Entries without a test: **{len(untested)}** of {len(interpretations)}",
            f"* Entries missing the development mark: **{len(unmarked)}**"]
    out += [f"* {i} has no **Development** line" for i in unmarked]
    out.append("")

    known = set(l1_ids) | set(l2_parent) | set(l3_parent) | {e[0] for e in interpretations}
    unknown = sorted(set(markers) - known, key=_sort_key)
    out += ["### Marker reference check", "",
            f"* Markers referencing unknown requirement or interpretation ids: **{len(unknown)}**"]
    out += [f"* `{r}` — referenced by {len(markers[r])} test(s)" for r in unknown]
    return "\n".join(out) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="Do not write; exit non-zero if the file would change.")
    args = parser.parse_args(argv)
    content = build_matrix()
    if args.check:
        try:
            current = TRACE_DOC.read_bytes().decode("utf-8")
        except OSError:
            current = ""
        if current != content:
            print(f"{TRACE_DOC.relative_to(ROOT).as_posix()} is out of date. "
                  "Run `python scripts/build-trace-matrix.py` to regenerate.", file=sys.stderr)
            return 1
        return 0
    # Write LF line endings on every platform.
    TRACE_DOC.write_bytes(content.encode("utf-8"))
    print(f"Wrote {TRACE_DOC.relative_to(ROOT).as_posix()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
