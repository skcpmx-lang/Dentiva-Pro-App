"""Fail-closed commercial release evidence validator; never builds an installer."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

GATES = (
    'BUILD LINT TYPECHECK UNIT INTEGRATION E2E DATABASE_INTEGRITY SECURITY RBAC PRINT PDF '
    'BANGLA_UNICODE BACKUP RESTORE ATTACHMENTS INSTALLER CLEAN_MACHINE PERFORMANCE HIGH_DPI '
    'LICENSE_AUDIT UI_UX_AUDIT NO_FAKE_FEATURES NO_TODO SECRET_SCAN DESTRUCTIVE_ACTIONS RELEASE_ARTIFACT'
).split()


def validate_report(report: dict, root: Path, commit: str) -> list[str]:
    errors: list[str] = []
    if report.get('source_commit') != commit:
        errors.append('Evidence must match the exact source commit being released.')
    if report.get('version') != '1.0.0' or report.get('release_tag') != 'v1.0.0':
        errors.append('The first commercial release must use consistent version 1.0.0 / tag v1.0.0.')
    gates = report.get('gates', {})
    if not isinstance(gates, dict):
        return errors + ['Gate evidence must be a keyed object.']
    for name in GATES:
        gate = gates.get(name, {})
        if not isinstance(gate, dict) or gate.get('status') != 'PASS':
            errors.append(f'{name}: not verified PASS.')
            continue
        if not gate.get('reviewer') or not gate.get('tested_at') or not gate.get('environment'):
            errors.append(f'{name}: reviewer, environment and execution time are required.')
        evidence = gate.get('evidence_path')
        if not isinstance(evidence, str) or not evidence:
            errors.append(f'{name}: evidence file is missing.')
            continue
        path = (root / evidence).resolve()
        if not path.is_relative_to(root.resolve()) or not path.is_file():
            errors.append(f'{name}: evidence must be a real repository-contained file.')
            continue
        if hashlib.sha256(path.read_bytes()).hexdigest() != gate.get('sha256'):
            errors.append(f'{name}: evidence checksum mismatch.')
    requirements = report.get('requirements', {})
    if not isinstance(requirements, dict):
        return errors + ['Requirements must be a keyed object.']
    missing = [f'R{i:03}' for i in range(1, 148) if requirements.get(f'R{i:03}') != 'PASS']
    if missing:
        errors.append(f'{len(missing)} master requirement groups have not passed: ' + ', '.join(missing))
    tests = report.get('tests', {})
    if not isinstance(tests, dict) or not isinstance(tests.get('passed'), int) or tests.get('passed', 0) < 1:
        errors.append('Executed passing test counts are required.')
    if not isinstance(tests, dict) or tests.get('failed') != 0 or tests.get('skipped') != 0:
        errors.append('Failed or skipped required tests block release.')
    findings = report.get('findings', {})
    if not isinstance(findings, dict) or findings.get('critical') != 0 or findings.get('high') != 0:
        errors.append('Unresolved or unassessed critical/high findings block release.')
    return errors


def validate_artifact(report: dict, root: Path) -> list[str]:
    artifact = report.get('artifact', {})
    if not isinstance(artifact, dict) or not isinstance(artifact.get('path'), str):
        return ['A real Windows installer artifact is required.']
    path = (root / artifact['path']).resolve()
    if not path.is_relative_to(root.resolve()) or not path.is_file() or path.suffix.lower() != '.exe':
        return ['Installer must be an existing repository-contained .exe, not a source file.']
    with path.open('rb') as stream:
        header = stream.read(64)
        if len(header) != 64 or header[:2] != b'MZ':
            return ['Installer lacks a Windows executable header.']
        pe_offset = int.from_bytes(header[60:64], 'little')
        if pe_offset > path.stat().st_size - 4:
            return ['Installer has an invalid executable structure.']
        stream.seek(pe_offset)
        if stream.read(4) != b'PE\0\0':
            return ['Installer lacks a PE signature.']
    with path.open('rb') as stream:
        checksum = hashlib.file_digest(stream, 'sha256').hexdigest()
    if checksum != artifact.get('sha256'):
        return ['Installer checksum does not match tested artifact.']
    return []


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    try:
        report = json.loads(args.report.read_text(encoding='utf-8'))
        if not isinstance(report, dict):
            raise ValueError('Report must be an object')
        commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
        errors = validate_report(report, root, commit) + validate_artifact(report, root)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'RELEASE BLOCKED: Cannot validate evidence: {error}', file=sys.stderr)
        return 1
    if errors:
        print('RELEASE BLOCKED\n' + '\n'.join(f'- {error}' for error in errors), file=sys.stderr)
        return 1
    print('Release evidence and artifact structure verified. Publication still requires tag/version and installer verification in the release workflow.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
