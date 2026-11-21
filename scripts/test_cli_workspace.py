"""Lockstep layout checks; metadata only, no dependency build or publication."""
import json
import re
from pathlib import Path
import subprocess
import tomllib
import unittest
import tempfile
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent


class CliWorkspace(unittest.TestCase):
    def test_virtual_members_share_one_version_and_workspace_library_pin(self):
        root = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        self.assertEqual(set(root), {'workspace'})
        self.assertEqual(root['workspace']['members'],
                         ['crates/irig106-time', 'apps/irig106-time-cli'])
        self.assertIn('fuzz', root['workspace']['exclude'])
        self.assertEqual(root['workspace']['resolver'], '3')
        lib = tomllib.loads((ROOT / 'crates/irig106-time/Cargo.toml').read_text())
        cli = tomllib.loads((ROOT / 'apps/irig106-time-cli/Cargo.toml').read_text())
        for package in [lib, cli]:
            self.assertTrue(package['package']['version']['workspace'])
            self.assertTrue(package['package']['rust-version']['workspace'])
            self.assertTrue(package['package']['license']['workspace'])
            self.assertTrue(package['package']['repository']['workspace'])
        version = root['workspace']['package']['version']
        self.assertEqual(root['workspace']['dependencies']['irig106-time'],
                         {'path': 'crates/irig106-time', 'version': '=' + version})
        self.assertEqual(cli['dependencies']['irig106-time'], {'workspace': True})
        self.assertFalse((ROOT / 'apps/irig106-time-cli/Cargo.lock').exists())
        self.assertTrue((ROOT / 'Cargo.lock').is_file())

    def test_literal_rust_includes_are_package_local(self):
        for member in ['crates/irig106-time', 'apps/irig106-time-cli']:
            package = (ROOT / member).resolve()
            for directory in ['src', 'tests', 'examples', 'benches']:
                for source in (package / directory).rglob('*.rs'):
                    for target in re.findall(r'include_(?:str|bytes)!\(\s*"([^"\n]+)"', source.read_text()):
                        resolved = (source.parent / target).resolve()
                        self.assertTrue(resolved.is_relative_to(package),
                                        f'{source.relative_to(ROOT)} includes outside its package: {target}')
                        self.assertTrue(resolved.is_file(), str(resolved))

    def test_include_and_path_guard_negative_probes(self):
        for attribute in ['include!("../../../outside.rs");',
                          '#[path = "../../../outside.rs"] mod external;']:
            with self.subTest(attribute=attribute), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                source = root / 'crates/irig106-time/src/probe.rs'
                source.parent.mkdir(parents=True)
                source.write_text(attribute)
                (root / 'outside.rs').write_text('// external source')
                with patch.dict(globals(), ROOT=root):
                    with self.assertRaisesRegex(AssertionError, 'outside its package'):
                        self.test_literal_rust_includes_are_package_local()

    def test_cli_schema_docs_snapshot_matches_packaged_source(self):
        self.assertEqual(
            (ROOT / 'apps/irig106-time-cli/schema/irigtime-report-v1.schema.json').read_bytes(),
            (ROOT / 'docs/schema/irigtime-report-v1.schema.json').read_bytes())

    def test_guide_network_snippets_match_packaged_fixture(self):
        guide = (ROOT / 'docs/usage.md').read_text().split('## 13. Processing Network Time Packets')[1].split('## 14.')[0]
        packaged = (ROOT / 'crates/irig106-time/tests/fixtures/format2_usage.md').read_text()
        def snippets(text):
            return re.findall(r'```rust\n(.*?)\n```', text, re.S)
        self.assertEqual(snippets(guide), snippets(packaged))
        self.assertEqual(len(snippets(packaged)), 2)

    def test_fuzz_remains_a_separate_workspace_with_its_own_lock(self):
        result = subprocess.run(['cargo', 'metadata', '--manifest-path',
                                 str(ROOT / 'fuzz/Cargo.toml'), '--no-deps',
                                 '--format-version', '1', '--locked'],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        meta = json.loads(result.stdout)
        self.assertEqual(Path(meta['workspace_root']), ROOT / 'fuzz')
        self.assertEqual(len(meta['workspace_members']), 1)
        self.assertTrue((ROOT / 'fuzz/Cargo.lock').is_file())

    def test_cargo_resolves_both_members_and_only_the_irigtime_binary(self):
        result = subprocess.run(['cargo', 'metadata', '--no-deps', '--format-version', '1',
                                 '--locked'], cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        meta = json.loads(result.stdout)
        packages = {p['name']: p for p in meta['packages']}
        self.assertEqual(set(packages), {'irig106-time', 'irig106-time-cli'})
        self.assertEqual(len(meta['workspace_members']), 2)
        self.assertEqual(packages['irig106-time']['version'],
                         packages['irig106-time-cli']['version'])
        binaries = [t['name'] for t in packages['irig106-time-cli']['targets']
                    if 'bin' in t['kind']]
        self.assertEqual(binaries, ['irigtime'])
        self.assertEqual(Path(meta['workspace_root']), ROOT)
        self.assertEqual(Path(meta['target_directory']), ROOT / 'target')


if __name__ == '__main__':
    unittest.main()
