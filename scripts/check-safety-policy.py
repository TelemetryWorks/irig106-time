#!/usr/bin/env python3
"""Negative compiler/Clippy fixtures using the library's actual root policy.

Small standalone probes, no dependency builds. CI runs this after normal Clippy.
A failure counts only if the expected policy diagnostic is present.
"""
from pathlib import Path
import os
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
LINTS = {
    "unwrap_used": 'pub fn probe(x: Option<u8>) -> u8 { x.unwrap() }',
    "expect_used": 'pub fn probe(x: Option<u8>) -> u8 { x.expect("input") }',
    "panic": 'pub fn probe() { panic!("probe") }',
    "todo": 'pub fn probe() { todo!("probe") }',
    "unimplemented": 'pub fn probe() { unimplemented!("probe") }',
    "unreachable": 'pub fn probe() { unreachable!("probe") }',
    "dbg_macro": 'pub fn probe(x: u8) -> u8 { dbg!(x) }',
    "print_stdout": 'pub fn probe() { println!("probe") }',
    "print_stderr": 'pub fn probe() { eprintln!("probe") }',
}


def main():
    source = (ROOT / "src/lib.rs").read_text()
    attrs = re.findall(r"^#!\[(?:forbid|deny)\([\s\S]*?\)\]", source, re.M)
    policy = [a for a in attrs if "unsafe_code" in a or "clippy::" in a]
    header = "\n".join(policy)
    assert "#![forbid(unsafe_code)]" in header, "missing root forbid(unsafe_code)"
    for lint in LINTS:
        assert f"clippy::{lint}" in header, f"missing root Clippy policy: {lint}"
    driver = subprocess.check_output(["rustup", "which", "clippy-driver"], text=True).strip()
    # Do not pick up caller flags that could silence or manufacture a failure.
    env = dict(os.environ)
    env.pop("CLIPPY_ARGS", None)
    with tempfile.TemporaryDirectory(prefix="time-safety-policy-") as tmp:
        path = Path(tmp) / "probe.rs"

        def check(name, body, diagnostic=None):
            path.write_text(header + "\n" + body + "\n")
            result = subprocess.run(
                [driver, "--edition=2024", "--crate-type=lib", "--emit=metadata",
                 "-o", str(Path(tmp) / "probe.rmeta"), str(path)],
                env=env, capture_output=True, text=True,
            )
            if diagnostic is None:
                assert result.returncode == 0, result.stderr
            else:
                assert result.returncode != 0, f"{name}: violation compiled successfully"
                assert diagnostic in result.stderr, f"{name}: unexpected failure: {result.stderr}"
            print(f"{name}: {'accepted' if diagnostic is None else 'rejected by policy'}")

        check("checked positive control", "pub fn probe(x: Option<u8>) -> u8 { x.unwrap_or(0) }")
        check("unsafe block", "pub fn probe() { unsafe { core::ptr::read_volatile(&0u8); } }",
              "unsafe_code")
        check("child cannot reallow unsafe", "#[allow(unsafe_code)] mod child {}", "unsafe_code")
        for lint, body in LINTS.items():
            check(lint, body, f"clippy::{lint}")


if __name__ == "__main__":
    main()
