#!/usr/bin/env python3
"""Read-only evidence collection from an isolated test VM via SSH.
Never point this script at the user's daily-use installation.
"""
import argparse
import json
import subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--port',required=True,type=int)
p.add_argument('--identity',required=True,type=Path)
p.add_argument('--known-hosts',required=True,type=Path)
p.add_argument('--output',required=True,type=Path)
a=p.parse_args()
code=r"""import hashlib,json,os,re,subprocess
from pathlib import Path
h=Path.home()
def run(args):
 r=subprocess.run(args,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 return {'rc':r.returncode,'output':r.stdout.strip()}
release=dict(line.split('=',1) for line in Path('/etc/os-release').read_text().splitlines() if '=' in line)
sessions=[]
for sid in run(['loginctl','list-sessions','--no-legend'])['output'].splitlines():
 parts=sid.split()
 if len(parts)>2 and parts[2]=='tfs':
  sessions.append(run(['loginctl','show-session',parts[0],'-p','Type','-p','Desktop','-p','Seat','-p','Remote']))
config=h/'.config/termfetch-studio'
validation=run(['fastfetch','--config',str(config/'fastfetch.jsonc'),'--show-errors','--pipe','true'])
errors=[line for line in validation['output'].splitlines() if re.search('Unknown JSON key|JsonConfig Error|Invalid enum',line)]
print(json.dumps({'os_release':{k:release.get(k) for k in ['ID','VERSION_ID','BUILD_ID','PRETTY_NAME']},'sessions':sessions,
 'fastfetch':run(['fastfetch','--version']),
 'installer_sha256':hashlib.sha256((h/'install.sh').read_bytes()).hexdigest(),
 'settings':(config/'config.ini').read_text(),
 'config_validation':{'rc':validation['rc'],'errors':errors},
 'runtime_logo_fields':[line.strip() for line in (config/'.fastfetch.current.jsonc').read_text().splitlines()[:20] if re.search('"(type|source|width|height)"',line)]},indent=2))
"""
r=subprocess.run(['ssh','-o','BatchMode=yes','-o','ConnectTimeout=10','-o',
 'UserKnownHostsFile='+str(a.known_hosts),'-i',str(a.identity),'-p'+str(a.port),
 'tfs@127.0.0.1','python3 -'],input=code,text=True,stdout=subprocess.PIPE,check=True)
data=json.loads(r.stdout)
a.output.parent.mkdir(parents=True,exist_ok=True)
a.output.write_text(json.dumps(data,indent=2)+'\n')
print(a.output)
