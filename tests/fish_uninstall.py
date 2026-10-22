#!/usr/bin/env python3
"""Exercise generated uninstaller against exact legacy Fish aliases, in Docker only."""
import argparse
import hashlib
import json
import re
import tempfile
import uuid
from pathlib import Path
from docker_matrix import ROOT, run, root_run, user_run, docker_user

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--installer',type=Path,default=ROOT/'install.sh')
parser.add_argument('--output',type=Path,required=True)
args=parser.parse_args()
source=args.installer.read_text()
script=re.search(r"<< 'UNINSTALL'\n(.*?)\nUNINSTALL\n",source,re.S).group(1)
result={'installer_sha256':hashlib.sha256(args.installer.read_bytes()).hexdigest(),'checks':{}}
name='tfs-fish-uninstall-'+uuid.uuid4().hex[:10]
with tempfile.TemporaryDirectory(prefix='tfs-fish-uninstall-') as temporary:
    folder=Path(temporary);folder.chmod(0o755);(folder/'uninstall.sh').write_text(script);(folder/'uninstall.sh').chmod(0o644)
    run('docker','run','-d','--name',name,'-v',f'{folder}:/src:ro','ubuntu:24.04','sleep','infinity')
    try:
        result['digest']=run('docker','image','inspect','ubuntu:24.04','--format','{{index .RepoDigests 0}}').stdout.strip()
        root_run(name,'apt-get update -qq && apt-get install -y -qq fish; useradd -m -s /bin/bash test')
        user_run(name,"mkdir -p ~/.local/bin ~/.local/share/termfetch-studio ~/.config/termfetch-studio ~/.config/fish; printf '#!/bin/sh\\nprintf NATIVE_FETCH\\n' > ~/.local/bin/fastfetch; chmod +x ~/.local/bin/fastfetch")
        fixture='''# personal settings
set -gx PERSONAL preserved
alias fastfetch="termfetch-studio --preview"
function clear
    command clear
    fastfetch
end
function personal_function
    echo preserved
end
'''
        user_run(name,"python3 - <<'PY'\nfrom pathlib import Path\n(Path.home()/'.config/fish/config.fish').write_text("+repr(fixture)+")\nPY")
        removed=user_run(name,'bash /src/uninstall.sh -y')
        argv=docker_user(name,'fish','-i','-c','clear; personal_function; printf CLEAR_READY');argv.remove('-i');argv.insert(2,'-t')
        output=run(*argv).stdout
        assert 'Unknown command' not in output and 'NATIVE_FETCH' in output and 'CLEAR_READY' in output and 'preserved' in output,output
        user_run(name,"python3 - <<'PY'\nfrom pathlib import Path\np=Path.home()/'.config/fish/config.fish'\ns=p.read_text()\nassert s=="+repr(fixture.replace('alias fastfetch="termfetch-studio --preview"\n',''))+"\nbackups=list(p.parent.glob('config.fish.termfetch-backup.*'))\nassert len(backups)==1 and backups[0].read_text()=="+repr(fixture)+"\nPY")
        assert 'functions --erase fastfetch' in removed.stdout
        result['checks']['legacy_alias_uninstall_clear']='passed: exact alias removed, original backed up, personal clear/function preserved, new Fish clear uses native fetch'
        # A child uninstaller cannot erase functions already loaded by its parent.
        user_run(name,"python3 - <<'PY'\nfrom pathlib import Path\n(Path.home()/'.config/fish/config.fish').write_text("+repr(fixture)+")\nPY")
        argv=docker_user(name,'fish','-i','-c','bash /src/uninstall.sh -y; functions --erase fastfetch; clear; printf LIVE_CLEAR_READY');argv.remove('-i');argv.insert(2,'-t')
        output=run(*argv).stdout
        assert 'Unknown command' not in output and 'NATIVE_FETCH' in output and 'LIVE_CLEAR_READY' in output,output
        result['checks']['existing_session_recovery']='passed: parent Fish keeps loaded alias until documented functions --erase fastfetch; clear then uses native fetch'
        # A custom alias with an extra command is personal code and must remain unchanged.
        custom='alias fastfetch="termfetch-studio --preview; echo personal"'
        user_run(name,"python3 - <<'PY'\nfrom pathlib import Path\n(Path.home()/'.config/fish/config.fish').write_text("+repr(custom)+")\nPY")
        user_run(name,'bash /src/uninstall.sh -y')
        user_run(name,"python3 - <<'PY'\nfrom pathlib import Path\nassert (Path.home()/'.config/fish/config.fish').read_text()=="+repr(custom)+"\nPY")
        result['checks']['custom_alias_preserved']='passed: compound custom alias is not removed'
    finally:run('docker','rm','-f',name,check=False)
args.output.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
