#!/usr/bin/env python3
import argparse,socket,time,json,subprocess,collections
from pathlib import Path
parser=argparse.ArgumentParser(description="Capture real QEMU preview transition samples; ROI color check requires a known colorful fixture.")
parser.add_argument('--monitor', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--region', nargs=4, type=int, required=True, metavar=('X','Y','WIDTH','HEIGHT'))
parser.add_argument('--text-region', nargs=4, type=int, metavar=('X','Y','WIDTH','HEIGHT'), help='Also verify stable menu text remains visible during upload')
a=parser.parse_args()
root=a.output;root.mkdir(parents=True,exist_ok=False)
x0,y0,rw,rh=a.region
if min(x0,y0)<0 or min(rw,rh)<1:parser.error('Invalid region')
with socket.socket(socket.AF_UNIX) as s:
 s.connect(str(a.monitor));s.settimeout(.1)
 try:s.recv(4096)
 except TimeoutError:pass
 def command(x):s.sendall((x+'\n').encode())
 command('sendkey down')
 records=[]
 for n in range(12):
  p=root/f'frame-{n:02}.ppm';t=time.monotonic();command('screendump '+json.dumps(str(p)))
  while True:
   if time.monotonic()-t>3:raise RuntimeError('frame capture timed out')
   if p.exists():
    header=p.read_bytes().split(b'\n',3)
    if len(header)==4:
     w,h=map(int,header[1].split());pixels=header[3]
     if len(pixels)==w*h*3:break
   time.sleep(.005)
  if x0+rw>w or y0+rh>h:raise ValueError('ROI outside framebuffer')
  colored=0
  for y in range(y0,y0+rh):
   line=pixels[(y*w+x0)*3:(y*w+x0+rw)*3]
   colored+=sum(max(line[i:i+3])-min(line[i:i+3])>35 for i in range(0,len(line),3))
  text_pixels=None
  if a.text_region:
   tx,ty,tw,th=a.text_region
   if min(tx,ty)<0 or min(tw,th)<1 or tx+tw>w or ty+th>h:raise ValueError('Text ROI outside framebuffer')
   colors=collections.Counter()
   for y in range(ty,ty+th):
    line=pixels[(y*w+tx)*3:(y*w+tx+tw)*3]
    colors.update(tuple(line[i:i+3]) for i in range(0,len(line),3))
   background=colors.most_common(1)[0][0]
   text_pixels=sum(count for rgb,count in colors.items() if max(abs(rgb[i]-background[i]) for i in range(3))>40)
  records.append({'text_pixels':text_pixels,'frame':n,'elapsed_ms':round((time.monotonic()-t)*1000),'preview_colored_pixels':colored})
  time.sleep(.05)
 assert all(x['preview_colored_pixels']>500 for x in records),records
 if a.text_region:assert all(x['text_pixels']>500 for x in records),records
 (root/'temporal.json').write_text(json.dumps({'method':'12 real QEMU framebuffer samples after down key; saturated fixture pixels >500 in every frame','region':a.region,'text_region':a.text_region,'frames':records},indent=2))
 for n in [0,5,11]:subprocess.run(['magick',str(root/f'frame-{n:02}.ppm'),str(root/f'frame-{n:02}.png')],check=True)
 print(json.dumps(records))
