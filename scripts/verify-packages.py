#!/usr/bin/env python3
"""Verify both crate archives and compile their isolated test sources; no publication."""
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
PACKAGES = ['irig106-time', 'irig106-time-cli']
ENV = dict(os.environ, CARGO_BUILD_JOBS='2')


def cargo(*args, cwd=ROOT):
    subprocess.run(['cargo', *args], cwd=cwd, env=ENV, check=True)


def main():
    original_lock = (ROOT / 'Cargo.lock').read_bytes()
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'],
        cwd=ROOT, env=ENV, text=True))
    target = Path(metadata['target_directory'])
    try:
        # Batch both packages so Cargo stages the matching internal dependency.
        # Verification is deliberately enabled (the default); never --no-verify.
        cargo('package', '-p', PACKAGES[0], '-p', PACKAGES[1], '--locked')
        with tempfile.TemporaryDirectory(prefix='irigtime-packaged-tests-') as temporary:
            extracted = {}
            for name in PACKAGES:
                with tarfile.open(target / 'package' / f'{name}-{version}.crate') as archive:
                    archive.extractall(temporary, filter='data')
                extracted[name] = Path(temporary) / f'{name}-{version}'
            lib, cli = (extracted[name] for name in PACKAGES)
            test_env = dict(ENV, CARGO_TARGET_DIR=str(target))
            for package in [lib, cli]:
                command = ['cargo', 'test', '--manifest-path', str(package / 'Cargo.toml'),
                           '--all-features', '--no-run']
                if package == lib:
                    command.append('--locked')
                else:
                    # Only this disposable fixture's lock may change: select the
                    # verified packaged library, never the old registry 0.7 API
                    # or the working tree. Release publication still needs the
                    # matching library published first. No persistent patch.
                    command += ['--config', 'patch.crates-io.irig106-time.path=' + json.dumps(str(lib))]
                subprocess.run(command, cwd=package, env=test_env, check=True)
    finally:
        if (ROOT / 'Cargo.lock').read_bytes() != original_lock:
            raise RuntimeError('package checks modified the workspace lockfile')


if __name__ == '__main__':
    main()
