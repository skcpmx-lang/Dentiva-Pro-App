"""Expose small redacted test transcripts via GitHub's API, not just artifact CDN."""
import json
import os
from pathlib import Path
from urllib.request import Request, urlopen

log_path = Path('native-tests.log')
log = log_path.read_text(encoding='utf-8', errors='replace') if log_path.exists() else 'Native tests did not execute; consult preceding build steps.'
result = os.environ.get('TEST_OUTCOME', 'skipped')
payload = {
    'name': f"Native test evidence ({os.environ['RUNNER_OS']})",
    'head_sha': os.environ['GITHUB_SHA'],
    'status': 'completed',
    'conclusion': 'success' if result == 'success' else 'failure',
    'output': {
        'title': f'Native test execution: {result}',
        'summary': 'Engineering evidence only, not commercial release certification.\n\n```text\n' + log[-58000:] + '\n```',
    },
}
request = Request(
    f"https://api.github.com/repos/{os.environ['GITHUB_REPOSITORY']}/check-runs",
    data=json.dumps(payload).encode('utf-8'),
    headers={
        'Authorization': f"Bearer {os.environ['GH_TOKEN']}",
        'Accept': 'application/vnd.github+json',
        'Content-Type': 'application/json',
        'X-GitHub-Api-Version': '2022-11-28',
    },
    method='POST',
)
with urlopen(request, timeout=30) as response:
    if response.status != 201:
        raise RuntimeError('Unable to publish native test evidence')
