#!/usr/bin/env python3
"""CI sparse reader/command RSS; dense report-state gate is separate."""
import json
import os
from pathlib import Path
import struct
import subprocess
import tempfile

# Compile discovery is cached by the preceding CLI tests. Measure the Rust test
# process ONLY, excluding Cargo/compiler and this file generator.
build = subprocess.check_output([
    "cargo", "test", "-p", "irig106-time-cli", "--test", "s16_reader",
    "--no-run", "--locked", "--message-format=json",
], text=True)
executables = [m["executable"] for line in build.splitlines()
               if (m := json.loads(line)).get("reason") == "compiler-artifact"
               and m.get("executable") and m["target"]["name"] == "s16_reader"]
assert len(executables) == 1, executables
size = 384 * 1024 * 1024
packet = bytearray(16384)
# §11.2.1.1: primary header; uninterpreted non-time body, correct word sum.
struct.pack_into("<HHII", packet, 0, 0xEB25, 0, len(packet), len(packet) - 24)
packet[15] = 9
struct.pack_into("<H", packet, 22, sum(struct.unpack("<11H", packet[:22])) & 0xFFFF)
with tempfile.TemporaryDirectory(prefix="irigtime-reader-rss-") as tmp:
    path = Path(tmp) / "generated.ch10"
    with path.open("wb") as out:
        for _ in range(size // len(packet)):
            out.write(packet)
    binaries = subprocess.check_output([
        "cargo", "build", "-p", "irig106-time-cli", "--bin", "irigtime",
        "--locked", "--message-format=json",
    ], text=True)
    cli = [m["executable"] for line in binaries.splitlines()
           if (m := json.loads(line)).get("reason") == "compiler-artifact"
           and m.get("executable") and m["target"]["name"] == "irigtime"]
    assert len(cli) == 1, cli
    binary = cli[0]
    cases = [("transport", [executables[0], "--exact", "rss_reader_worker", "--ignored"])]
    for command in ["summary", "channels", "jumps", "timeline", "csv", "correlate"]:
        argv = [str(binary), command, str(path)]
        if command == "correlate":
            argv.append("0")
        if command == "timeline":
            argv.extend(["--limit", "50"])  # Pin the probe, not the CLI default.
        cases.append((command, argv))
    for name, argv in cases:
        with (Path(tmp) / f"{name}.log").open("w+") as log:
            proc = subprocess.Popen(argv, env={**os.environ, "IRIGTIME_RSS_INPUT": str(path)},
                                    stdout=log, stderr=subprocess.STDOUT)
            _, status, usage = os.wait4(proc.pid, 0)  # Linux ru_maxrss is KiB.
            proc.returncode = os.waitstatus_to_exitcode(status)
            log.seek(0)
            prefix = log.read(8192)  # Never load a CSV report into probe memory.
            assert proc.returncode == 0, (name, proc.returncode, prefix)
            if name != "transport":
                # CSV/timeline attempt counter-only row answers. TF-001 still
                # counts, but arrives after the first 20 diagnostic samples.
                warnings = {"csv": size // len(packet) + 1,
                            "timeline": 51, "correlate": 2}.get(name, 1)
                counts = (f"Findings: error=0, warning={warnings}, info=0 "
                          f"(total={warnings})")
                assert counts in prefix, (name, counts, prefix)
            if name == "summary":
                assert "Total Packets      : 24,576" in prefix, prefix
            if name == "transport":
                print(prefix)
        result = (f"{name} input RSS: {usage.ru_maxrss} KiB peak; {size} input bytes; "
                  f"{size // len(packet)} non-time packets; 8192-byte buffer. "
                  "Sparse transport probe; dense report-state gate is separate.")
        print(result, flush=True)
        if summary := os.environ.get("GITHUB_STEP_SUMMARY"):
            with open(summary, "a") as out:
                out.write(result + "\n")
        assert usage.ru_maxrss < 64 * 1024, (name, "input-sized memory retained")
