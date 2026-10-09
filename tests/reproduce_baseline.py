#!/usr/bin/env python3
"""Compare isolated reproductions against the original installer (no installation)."""
import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--ref', default='8f26432')
a = p.parse_args()
original = subprocess.check_output(['git', 'show', a.ref + ':install.sh'], cwd=root, text=True)
current = (root / 'install.sh').read_text()
results = {'baseline_ref': a.ref, 'image': 'ubuntu:24.04', 'cases': {}}
with tempfile.TemporaryDirectory(prefix='tfs-reproduce-') as directory:
    case = Path(directory)
    case.chmod(0o755)
    for name, source in [('baseline', original), ('fixed', current)]:
        core = re.search(r"<< 'CORELIB'\n(.*?)\nCORELIB", source, re.S).group(1)
        detect = re.search(r'^detect_distro_base\(\) \{.*?^\}', source, re.M | re.S).group(0)
        (case / (name + '-core.sh')).write_text(core)
        (case / (name + '-detect.sh')).write_text(detect.replace('/etc/os-release', '/tmp/os-release'))
        script = '''mkdir -p "$HOME/config"
CONFIG_DIR="$HOME/config"
source /cases/NAME-core.sh
source /cases/NAME-detect.sh
printf 'ID=opensuse-tumbleweed\nID_LIKE="suse opensuse"\n' > /tmp/os-release
printf 'DETECTION='; detect_distro_base
printf 'setting=before\n' > "$CONFIG_DIR/config.ini"
create_backup
create_backup
printf '\nARCHIVE_ENTRIES\n'
for f in "$BACKUP_DIR"/*.tar.gz; do tar -tzf "$f"; done
set -- "$BACKUP_DIR"/*.tar.gz
mv "$1" "$BACKUP_DIR/backup with spaces.tar.gz"
for f in "$BACKUP_DIR"/backup_*.tar.gz; do rm -f -- "$f"; done
printf '\nRESTORE_INVALID\n'
printf 'oops\n' | restore_backup
printf 'restore_rc=%s\n' "$?"
kitty() { :; }
pgrep() { return 0; }
export TERM=dumb TERM_PROGRAM='' KITTY_WINDOW_ID='' WAYLAND_DISPLAY=''
printf '\nTERMINAL='; check_terminal_image_support
'''.replace('NAME', name)
        (case / (name + '.sh')).write_text(script)
        result = subprocess.run(['docker', 'run', '--rm', '--user', '1000:1000',
                                 '-e', 'HOME=/tmp/test', '-v', f'{case}:/cases:ro',
                                 'ubuntu:24.04', 'bash', '/cases/' + name + '.sh'],
                                text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        results['cases'][name] = {'exit_code': result.returncode, 'transcript': result.stdout}
        (case / (name + '-installer.sh')).write_text(source.rsplit('\nmain "$@"', 1)[0])
        optional = '''source /cases/NAME-installer.sh
fastfetch() { echo 'fastfetch 2.53.0'; }
awk() { :; }
sudo() { echo 'MOCK: optional package installation failed' >&2; return 1; }
if install_dependencies; then :; else :; fi
'''.replace('NAME', name)
        (case / (name + '-optional.sh')).write_text(optional)
        failed_optional = subprocess.run(['docker', 'run', '--rm', '--user', '1000:1000',
                                         '-e', 'HOME=/tmp/test', '-v', f'{case}:/cases:ro',
                                         'ubuntu:24.04', 'bash', '/cases/' + name + '-optional.sh'],
                                        text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        results['cases'][name]['optional_transcript'] = failed_optional.stdout
    before = results['cases']['baseline']['transcript']
    after = results['cases']['fixed']['transcript']
    results['assertions'] = {
        'suse_identity': 'DETECTION=unknown' in before and 'DETECTION=suse' in after,
        'recursive_backup': './backups/' in before and './backups/' not in after,
        'spaced_restore_name': '1) backup with spaces.tar.gz' in after and '2) with' in before,
        'invalid_restore_input': 'integer expression expected' in before and 'Invalid choice' in after,
        'unrelated_kitty': 'TERMINAL=kitty' in before and 'TERMINAL=none' in after,
        'optional_failure_message': ('Optional dependencies installed' in results['cases']['baseline']['optional_transcript']
            and 'MOCK: optional package installation failed' in results['cases']['fixed']['optional_transcript']
            and 'Optional dependencies installed' not in results['cases']['fixed']['optional_transcript']),
    }
output = root / 'tests/baseline-results.json'
output.write_text(json.dumps(results, indent=2) + '\n')
print(json.dumps(results['assertions'], indent=2))
raise SystemExit(0 if all(results['assertions'].values()) else 1)
