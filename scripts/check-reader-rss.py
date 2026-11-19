#!/usr/bin/env python3
"""CI transport-only peak-RSS probe; not the still-pending whole-CLI gate."""
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
    with (Path(tmp) / "worker.log").open("w+") as log:
        proc = subprocess.Popen(
            [executables[0], "--exact", "rss_reader_worker", "--ignored"],
            env={**os.environ, "IRIGTIME_RSS_INPUT": str(path)}, stdout=log,
            stderr=subprocess.STDOUT,
        )
        _, status, usage = os.wait4(proc.pid, 0)  # Linux ru_maxrss is KiB.
        proc.returncode = os.waitstatus_to_exitcode(status)
        log.seek(0)
        print(log.read())
        assert proc.returncode == 0, proc.returncode
    result = (f"Transport-only RSS: {usage.ru_maxrss} KiB peak; {size} input bytes; "
              f"{size // len(packet)} packets; 8192-byte buffer, capture=0. "
              "Whole-CLI joining/report memory gate remains open.")
    print(result, flush=True)
    if summary := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(summary, "a") as out:
            out.write(result + "\n")
    assert usage.ru_maxrss < 64 * 1024, "transport retains input-sized state"
