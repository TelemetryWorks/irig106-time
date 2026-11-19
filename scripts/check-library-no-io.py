#!/usr/bin/env python3
"""Compile the production root prefix under std feature; I/O must stay unavailable.

No probes are executed, and no file/network/clock/environment call is made.
Dependencies and caller callbacks still require the separate source audit.
"""
from pathlib import Path
import os
import subprocess
import tempfile
from rust_safety_scan import scan_project

ROOT = Path(__file__).resolve().parent.parent


def main():
    report = scan_project(ROOT, include_cli=False)
    for error in report.io_errors + report.source_errors:
        print(error, flush=True)
    assert not (report.io_errors or report.source_errors), "production std import/path or source indirection outside cfg(test) code"
    source = (ROOT / "src/lib.rs").read_text()
    start = source.index("#![no_std]")
    end = source.index("extern crate alloc;") + len("extern crate alloc;")
    header = source[start:end]
    probes = {
        "positive": ('pub fn probe() -> u8 { core::cmp::min(1,2) }', None),
        "clock": ('pub fn probe() { let _ = std::time::SystemTime::now(); }', "E0433"),
        "environment": ('pub fn probe() { let _ = std::env::var("IRIG_TIME"); }', "E0433"),
        "file": ('pub fn probe() { let _ = std::fs::read("not-opened"); }', "E0433"),
        "network": ('pub fn probe() { let _ = std::net::TcpStream::connect("not-connected"); }', "E0433"),
    }
    env = dict(os.environ)
    with tempfile.TemporaryDirectory(prefix="time-no-io-") as tmp:
        path = Path(tmp) / "probe.rs"
        for name, (body, error) in probes.items():
            path.write_text("//! Production-boundary fixture.\n" + header + "\n/// Probe.\n" + body)
            result = subprocess.run(
                ["rustc", "--edition=2024", "--crate-type=lib", "--emit=metadata",
                 "--cfg", 'feature="std"', "-o", str(Path(tmp) / "probe.rmeta"), str(path)],
                capture_output=True, text=True, env=env,
            )
            if error is None:
                assert result.returncode == 0, result.stderr
            else:
                assert result.returncode != 0, f"{name}: production prefix admitted std I/O"
                assert error in result.stderr and "std" in result.stderr, result.stderr
            print(f"{name}: {'accepted' if error is None else 'std unavailable'}")


if __name__ == "__main__":
    main()
