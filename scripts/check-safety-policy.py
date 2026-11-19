#!/usr/bin/env python3
"""Negative compiler/Clippy fixtures using the library's actual root policy.

Small standalone probes, no dependency builds. CI runs this after normal Clippy.
A failure counts only if the expected policy diagnostic is present.
"""
from pathlib import Path
import os
import subprocess
import tempfile
from rust_safety_scan import scan_project, root_policies

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


def check_root(root):
    source = root.read_text()
    attrs = root_policies(source)
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
            print(f"{root.relative_to(ROOT)} / {name}: {'accepted' if diagnostic is None else 'rejected by policy'}")

        check("checked positive control", "pub fn probe(x: Option<u8>) -> u8 { x.unwrap_or(0) }")
        check("unsafe block", "pub fn probe() { unsafe { core::ptr::read_volatile(&0u8); } }",
              "unsafe_code")
        check("child cannot reallow unsafe", "#[allow(unsafe_code)] mod child {}", "unsafe_code")
        for lint, body in LINTS.items():
            check(lint, body, f"clippy::{lint}")


if __name__ == "__main__":
    report = scan_project(ROOT)
    for allowance in report.allowances:
        print(f"{allowance.location}: {','.join(sorted(allowance.lints))}: "
              f"{'allowed' if allowance.allowed else 'FORBIDDEN'} ({allowance.scope})", flush=True)
    for error in report.source_errors:
        print(error, flush=True)
    assert not report.source_errors, "forbidden production source indirection(s) above"
    assert all(a.allowed for a in report.allowances), "forbidden safety-policy allowance(s) above"
    for root in [ROOT / "src/lib.rs", ROOT / "irig106-time-cli/src/lib.rs",
                 ROOT / "irig106-time-cli/src/bin/irigtime.rs"]:
        check_root(root)
