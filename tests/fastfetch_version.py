#!/usr/bin/env python3
"""Exercise the version guard with synthetic fastfetch --version output."""
import hashlib
import json
import re
import subprocess
from pathlib import Path
root = Path(__file__).resolve().parents[1]
source = (root / 'install.sh').read_text()
function = re.search(r'^validate_fastfetch_version\(\) \{.*?^\}', source, re.M | re.S).group(0)
cases = []
for value, expected in [('fastfetch 2.40.4', False), ('fastfetch 2.41.9', False),
                        ('fastfetch 2.42.0', True), ('fastfetch 2.53.0 (x86_64)', True),
                        ('fastfetch 2.69.0', True), ('fastfetch 3.0.0', True),
                        ('unparseable', False)]:
    result = subprocess.run(['bash', '-c', 'print_error() { printf "%s\\n" "$*"; };\n' + function +
                             '\nvalue=$1; fastfetch() { printf "%s\\n" "$value"; }; validate_fastfetch_version',
                             'bash', value], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    passed = (result.returncode == 0) == expected
    cases.append({'synthetic_output': value, 'accepted': result.returncode == 0,
                  'result': 'passed' if passed else 'failed', 'message': result.stdout.strip()})
report = {'installer_sha256': hashlib.sha256(source.encode()).hexdigest(), 'cases': cases}
(root / 'tests/version-results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(cases, indent=2))
raise SystemExit(0 if all(c['result'] == 'passed' for c in cases) else 1)
