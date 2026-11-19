"""Lockstep layout checks; metadata only, no dependency build or publication."""
import json
from pathlib import Path
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parent.parent


class CliWorkspace(unittest.TestCase):
    def test_root_and_sibling_share_one_version_and_exact_library_pin(self):
        root = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        cli = tomllib.loads((ROOT / 'irig106-time-cli/Cargo.toml').read_text())
        self.assertEqual(root['workspace']['members'], ['.', 'irig106-time-cli'])
        self.assertIn('fuzz', root['workspace']['exclude'])
        self.assertTrue(root['package']['version']['workspace'])
        self.assertTrue(cli['package']['version']['workspace'])
        version = root['workspace']['package']['version']
        self.assertEqual(cli['dependencies']['irig106-time'],
                         {'path': '..', 'version': '=' + version})
        self.assertFalse((ROOT / 'irig106-time-cli/Cargo.lock').exists())
        self.assertTrue((ROOT / 'Cargo.lock').is_file())

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
