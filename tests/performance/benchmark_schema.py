"""Disposable SQL-only benchmark. Not evidence of desktop/render/attachment performance."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import sqlite3
import statistics
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]


def run(sizes):
    migration = (ROOT / 'src-tauri/migrations/0001_foundation.sql').read_text()
    source = (ROOT / 'crates/dentiva-core/src/patients.rs').read_text()
    projection = re.search(r'const PROJECTION: &str = "(.*?)";', source, re.S).group(1)
    where = re.search(r'let filter\s*=\s*r"(.*?)";', source, re.S).group(1)
    page_query = f'{projection} {where} ORDER BY p.registered_at DESC,p.id DESC LIMIT ?5 OFFSET ?6'
    count_query = f'SELECT COUNT(*) FROM patients p {where}'
    result = {
        'scope': 'Actual demographic list SQL extracted from native source; not full application performance',
        'environment': {'os': platform.system(), 'architecture': platform.machine(), 'logical_cpus': os.cpu_count(), 'sqlite_version': sqlite3.sqlite_version},
        'measurements': [],
    }
    with tempfile.TemporaryDirectory() as temporary:
        database = Path(temporary) / 'benchmark.db'
        conn = sqlite3.connect(database)
        conn.execute('PRAGMA foreign_keys=ON')
        conn.execute('PRAGMA journal_mode=WAL')
        conn.execute('PRAGMA synchronous=FULL')
        conn.executescript(migration)
        conn.execute("INSERT INTO roles VALUES('owner','Owner',1)")
        conn.execute("INSERT INTO users(id,username,full_name,password_hash,role_id,created_at) VALUES('u','synthetic','Synthetic fixture','not-a-usable-login-hash','owner','2026-09-27T00:00:00.000Z')")
        previous = 0
        for size in sizes:
            start = time.perf_counter()
            with conn:
                conn.executemany("INSERT INTO patients(id,code,name,gender,address,registered_at,registered_by,updated_at) VALUES(?,?,?,'Not specified',?,'2026-09-27T00:00:00.000Z','u','2026-09-27T00:00:00.000Z')",
                                 ((f'p{i:06}', f'DP-2026-{i:06}', f'Synthetic patient {i:06}', 'Synthetic address') for i in range(previous, size)))
                conn.executemany("INSERT INTO patient_contacts VALUES(?,?,'Primary','',?)",
                                 ((f'c{i:06}', f'p{i:06}', f'017{i:08}') for i in range(previous, size)))
            load_ms = (time.perf_counter() - start) * 1000
            previous = size
            scenarios = []
            for label, search in [('newest_page', ''), ('name_search', f'{size-1:06}'), ('phone_search', f'017{size-1:08}')]:
                samples = []
                count = None
                for _ in range(11):
                    start = time.perf_counter()
                    args = (search, f'%{search}%', None, None)
                    count = conn.execute(count_query, args).fetchone()[0]
                    rows = conn.execute(page_query, (*args, 50, 0)).fetchall()
                    samples.append((time.perf_counter() - start) * 1000)
                    assert len(rows) <= 50
                scenarios.append({'name': label, 'matched_rows': count, 'page_limit': 50,
                                  'first_query_ms': round(samples[0], 3),
                                  'warm_median_ms': round(statistics.median(samples[1:]), 3),
                                  'warm_p95_ms': round(sorted(samples[1:])[-1], 3),
                                  'query_plan': [row[3] for row in conn.execute('EXPLAIN QUERY PLAN ' + page_query, (*args, 50, 0))]})
            integrity = conn.execute('PRAGMA integrity_check').fetchone()[0]
            foreign_keys = conn.execute('PRAGMA foreign_key_check').fetchall()
            result['measurements'].append({'patients': size, 'contacts': size, 'incremental_load_ms': round(load_ms, 3),
                                           'integrity': integrity, 'foreign_key_violations': len(foreign_keys), 'scenarios': scenarios})
            assert integrity == 'ok' and not foreign_keys
        conn.close()
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    options = parser.parse_args()
    report = run([1000, 10000, 50000, 100000])
    options.output.parent.mkdir(parents=True, exist_ok=True)
    options.output.write_text(json.dumps(report, indent=2) + '\n')
    for measurement in report['measurements']:
        print(measurement['patients'], 'patients:', [(item['name'], item['warm_p95_ms']) for item in measurement['scenarios']])
