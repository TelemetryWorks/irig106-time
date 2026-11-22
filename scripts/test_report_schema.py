"""CI schema boundary probes; partial CLI-106 evidence, not a whole marker."""
import copy
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent


class ReportSchema(unittest.TestCase):
    def test_ci_installs_hash_locked_tools_and_checks_live_views(self):
        workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        # The colon-space in :all: must stay inside a YAML block scalar.
        self.assertIn('run: |\n          python -m pip install --require-hashes --only-binary=:all:', workflow)
        self.assertIn('scripts/requirements-schema.txt', workflow)
        self.assertIn('python scripts/validate_cli_reports.py', workflow)
        self.assertIn('test_report_schema.py', workflow)

    def test_all_supported_json_views_are_in_the_live_gate(self):
        from validate_cli_reports import VIEWS
        self.assertEqual(set(VIEWS), {'summary', 'channels', 'jumps', 'timeline', 'correlate'})

    def test_existing_full_goldens_validate(self):
        from validate_cli_reports import validator, validate_report
        check = validator()
        for name in ['s17_missing_year.json', 's17_protocol_inventory.json']:
            value = json.loads((ROOT / 'apps/irig106-time-cli/tests/golden' / name).read_text())
            validate_report(value, check)

    def test_recursive_refs_wide_values_and_current_revision_are_enforced(self):
        from jsonschema import ValidationError
        from validate_cli_reports import validator, validate_report
        check = validator()
        original = json.loads((ROOT / 'apps/irig106-time-cli/tests/golden/s17_missing_year.json').read_text())
        for path, replacement in [
            (['schema_revision'], 999),
            (['unexpected'], True),
            (['report', 'total_packets'], 1),
            (['report', 'channels', 0, 'first', 'basis'], 'invented'),
            (['report', 'channels', 0, 'first', 'clock_count_ns'], 123),
            (['report', 'policy', 'ntp_era_inference'], 'enabled'),
        ]:
            value = copy.deepcopy(original)
            target = value
            for key in path[:-1]:
                target = target[key]
            target[path[-1]] = replacement
            with self.subTest(path=path), self.assertRaises(ValidationError):
                validate_report(value, check)

    def test_invalid_producer_schema_fails_metaschema(self):
        from jsonschema import SchemaError
        from validate_cli_reports import validator, SCHEMA
        import tempfile
        schema = json.loads(SCHEMA.read_text())
        schema['type'] = 'not-a-json-schema-type'
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'schema.json'
            path.write_text(json.dumps(schema))
            with self.assertRaises(SchemaError):
                validator(path)


if __name__ == '__main__':
    unittest.main()
