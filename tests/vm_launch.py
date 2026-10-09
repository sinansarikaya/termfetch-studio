#!/usr/bin/env python3
"""Launch an isolated existing QCOW2 overlay and cloud-init ISO using host KVM.
Does not install packages, attach host homes, or modify an existing VM.
"""
import argparse
import shutil
import subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--disk', required=True, type=Path)
p.add_argument('--seed', required=True, type=Path)
p.add_argument('--state', required=True, type=Path)
p.add_argument('--ssh-port', type=int, default=2222)
p.add_argument('--vnc-display', type=int, default=4)
a=p.parse_args()
if not a.disk.is_file() or not a.seed.is_file():
    p.error('Disk and seed must exist; create an overlay before launching')
a.state.mkdir(parents=True, exist_ok=True)
a.state=a.state.resolve()
if (a.state/'qemu.pid').exists():
    p.error('Use a fresh state directory; an existing PID file is present')
variables=a.state/'OVMF_VARS.fd'
if not variables.exists():
    shutil.copy('/usr/share/OVMF/OVMF_VARS_4M.fd',variables)
subprocess.run(['qemu-system-x86_64','-machine','q35,accel=kvm','-cpu','host',
    '-smp','4','-m','4096','-drive',
    'if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd',
    '-drive','if=pflash,format=raw,file='+str(variables),
    '-drive','file='+str(a.disk.resolve())+',if=virtio,format=qcow2',
    '-drive','file='+str(a.seed.resolve())+',media=cdrom,readonly=on',
    '-netdev',f'user,id=net0,hostfwd=tcp:127.0.0.1:{a.ssh_port}-:22',
    '-device','virtio-net-pci,netdev=net0','-device','virtio-vga',
    '-vnc',f'127.0.0.1:{a.vnc_display}',
    '-monitor','unix:'+str(a.state/'monitor.sock')+',server,nowait',
    '-serial','file:'+str(a.state/'serial.log'),
    '-pidfile',str(a.state/'qemu.pid'),'-daemonize'],check=True)
print(a.state/'monitor.sock')
