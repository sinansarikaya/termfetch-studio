#!/usr/bin/env python3
"""Check rejection of Debian 13's actual older distro package as a normal user."""
import hashlib
import json
import tempfile
import uuid
from pathlib import Path
from docker_matrix import ROOT, run, root_run, user_run

name = 'tfs-old-version-' + uuid.uuid4().hex[:12]
result = {'image': 'debian:13', 'installer_sha256': hashlib.sha256((ROOT/'install.sh').read_bytes()).hexdigest()}
phase = 'preparation'
with tempfile.TemporaryDirectory(prefix='tfs-old-version-') as directory:
    source = Path(directory)
    source.chmod(0o755)
    (source/'install.sh').write_bytes((ROOT/'install.sh').read_bytes())
    try:
        result['digest'] = json.loads(run('docker','image','inspect','debian:13').stdout)[0]['RepoDigests'][0]
        run('docker','run','-d','--name',name,'-v',f'{source}:/src:ro','debian:13','sleep','3600')
        root_run(name,'apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq fastfetch sudo curl ca-certificates')
        root_run(name,"useradd -m -s /bin/bash test && echo 'test ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/test && chmod 0440 /etc/sudoers.d/test")
        before = user_run(name,'fastfetch --version').stdout.strip()
        phase = 'behavior'
        output = user_run(name,'bash /src/install.sh',check=False)
        after = user_run(name,'fastfetch --version; test ! -e ~/.local/bin/termfetch-studio')
        result.update(fastfetch=before, exit_code=output.returncode, transcript=output.stdout)
        result['result'] = ('passed: old distro package rejected without replacement' if
            before.startswith('fastfetch 2.40.4') and output.returncode != 0 and
            'fastfetch >= 2.42.0 is required' in output.stdout and before == after.stdout.strip()
            else 'failed: rejection or preservation assertion did not hold')
    except Exception as error:
        result.update(result=('blocked: environment/package preparation' if phase == 'preparation'
                              else 'failed: application/version behavior'), error=str(error))
    finally:
        run('docker','rm','-f',name,check=False)
(ROOT/'tests/old-fastfetch-results.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
raise SystemExit(0 if result['result'].startswith('passed:') else 1)
