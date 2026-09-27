import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[2] / 'scripts/release_gate.py'
spec = importlib.util.spec_from_file_location('release_gate', SCRIPT)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class ReleaseGateTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        (self.root / 'evidence.txt').write_text('Synthetic validator unit-test evidence, not release evidence.')
        self.report = {
            'source_commit': 'source', 'version': '1.0.0', 'release_tag': 'v1.0.0',
            'gates': {name: {
                'status': 'PASS', 'reviewer': 'unit fixture', 'environment': 'unit fixture',
                'tested_at': '2026-09-27', 'evidence_path': 'evidence.txt',
                'sha256': hashlib.sha256((self.root / 'evidence.txt').read_bytes()).hexdigest(),
            } for name in gate.GATES},
            'requirements': {f'R{i:03}': 'PASS' for i in range(1, 148)},
            'tests': {'passed': 1, 'failed': 0, 'skipped': 0},
            'findings': {'critical': 0, 'high': 0},
        }

    def tearDown(self):
        self.directory.cleanup()

    def test_empty_report_fails_closed(self):
        self.assertGreater(len(gate.validate_report({}, self.root, 'source')), len(gate.GATES))
        self.assertTrue(gate.validate_artifact({}, self.root))

    def test_valid_evidence_contract_is_accepted_not_installer(self):
        self.assertEqual(gate.validate_report(self.report, self.root, 'source'), [])
        self.assertTrue(gate.validate_artifact(self.report, self.root))

    def test_source_mismatch_and_missing_requirement_block(self):
        self.report['requirements'].pop('R147')
        errors = gate.validate_report(self.report, self.root, 'another-source')
        self.assertTrue(any('source commit' in item for item in errors))
        self.assertTrue(any('R147' in item for item in errors))

    def test_skip_is_not_pass(self):
        self.report['gates']['PRINT']['status'] = 'SKIPPED'
        self.report['tests']['skipped'] = 1
        self.assertTrue(gate.validate_report(self.report, self.root, 'source'))

    def test_tampering_and_path_escape_block(self):
        self.report['gates']['PRINT']['sha256'] = 'incorrect'
        self.report['gates']['PDF']['evidence_path'] = '../outside-file'
        errors = gate.validate_report(self.report, self.root, 'source')
        self.assertTrue(any('checksum mismatch' in item for item in errors))
        self.assertTrue(any('repository-contained' in item for item in errors))

    def test_renamed_text_is_not_installer(self):
        (self.root / 'not-an-installer.exe').write_text('Not a Windows installer')
        self.report['artifact'] = {'path': 'not-an-installer.exe'}
        self.assertTrue(gate.validate_artifact(self.report, self.root))


if __name__ == '__main__':
    unittest.main(verbosity=2)
