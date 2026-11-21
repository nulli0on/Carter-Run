#!/usr/bin/env python3
"""Measure actual weapon visibility by toggling B to compare against unchanged world frame."""
from pathlib import Path
import subprocess,time,os,ctypes,json,sys
from PIL import ImageGrab,Image,ImageDraw
import numpy as np
ROOT=Path(__file__).resolve().parents[1]; OUT=ROOT/'visual_qa'/('visibility_'+os.getenv('QA_LABEL','default'))
OUT.mkdir(parents=True,exist_ok=True); BIN=ROOT/'target/release/run-carter';DISPLAY=':192'
base={**os.environ,'DISPLAY':DISPLAY,'LIBGL_ALWAYS_SOFTWARE':'1','ALSOFT_DRIVERS':'null'}
xserver=subprocess.Popen(['Xvfb',DISPLAY,'-screen','0','1600x1000x24','-nolisten','tcp'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
try:
 time.sleep(.65)
 x11=ctypes.CDLL('libX11.so.6');xtst=ctypes.CDLL('libXtst.so.6')
 x11.XOpenDisplay.argtypes=[ctypes.c_char_p];x11.XOpenDisplay.restype=ctypes.c_void_p;x11.XFlush.argtypes=[ctypes.c_void_p]
 x11.XStringToKeysym.argtypes=[ctypes.c_char_p];x11.XStringToKeysym.restype=ctypes.c_ulong
 x11.XKeysymToKeycode.argtypes=[ctypes.c_void_p,ctypes.c_ulong];x11.XKeysymToKeycode.restype=ctypes.c_ubyte
 xtst.XTestFakeKeyEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
 dx=x11.XOpenDisplay(DISPLAY.encode());assert dx
 key=int(x11.XKeysymToKeycode(dx,x11.XStringToKeysym(b'b')));print('X keycode B',key)
 def press_b():
  xtst.XTestFakeKeyEvent(dx,key,1,0);x11.XFlush(dx);time.sleep(.05)
  xtst.XTestFakeKeyEvent(dx,key,0,0);x11.XFlush(dx)
 def cap():return ImageGrab.grab(xdisplay=DISPLAY).crop((80,50,1520,950))
 names=['Cardboard','Knife','Mark23','Vector','M4','Minigun','Flamethrower','RPG7','Grenade','Dynamite','Shotgun','Sniper','Persuader','LMG','Grenade Launcher','Pistol']
 slots=[int(a) for a in sys.argv[1:]] if len(sys.argv)>1 else range(16)
 results=[];tiles=[]
 for i in slots:
  with open(OUT/f'{i:02d}.log','wb') as log:
   p=subprocess.Popen([str(BIN),'--smoke-frames=600',f'--qa-weapon-slot={i}',*os.getenv('QA_VM_ARGS','').split()],cwd=ROOT,env=base,stdout=log,stderr=subprocess.STDOUT)
   try:
    time.sleep(1.9)
    if p.poll() is not None:raise RuntimeError('game exited early')
    with_weapon=cap()
    press_b();time.sleep(.2)
    no_weapon=cap()
    # Weapon-only mask. Restrict to main 3D view, excluding HUD and crosshair.
    a=np.array(with_weapon.crop((0,72,1440,780)),dtype='int16')
    b=np.array(no_weapon.crop((0,72,1440,780)),dtype='int16')
    diff=np.max(np.abs(a-b),axis=2)>45
    # Ignore tiny antialiasing, retain strong differences; possible crowd movement errors small.
    n=int(diff.sum()); ys,xs=np.where(diff)
    box=([int(xs.min()),int(ys.min()+72),int(xs.max()),int(ys.max()+72)] if n>0 else None)
    with_weapon.save(OUT/f'{i:02d}_with.png')
    no_weapon.save(OUT/f'{i:02d}_without.png')
    image=with_weapon.resize((480,300));draw=ImageDraw.Draw(image)
    draw.rectangle((0,0,480,27),fill=(8,16,29));draw.text((8,5),f'{i:02d} {names[i]} visible={n:,} px',fill='white')
    tiles.append(image)
    results.append({'slot':i,'name':names[i],'visible_pixels':n,'box':box})
    print(i,n,box,flush=True)
   except Exception as e:
    results.append({'slot':i,'name':names[i],'error':str(e)});print('ERROR',i,e,flush=True)
   finally:
    p.terminate()
    try:p.wait(timeout=4)
    except subprocess.TimeoutExpired:p.kill();p.wait(timeout=4)
 if tiles:
  sheet=Image.new('RGB',(480*4,300*((len(tiles)+3)//4)))
  for n,img in enumerate(tiles):sheet.paste(img,((n%4)*480,(n//4)*300))
  sheet.save(OUT/'visual_contact.jpg',quality=90)
 (OUT/'visibility.json').write_text(json.dumps(results,indent=2))
finally:
 xserver.terminate()
 try:xserver.wait(timeout=2)
 except subprocess.TimeoutExpired:xserver.kill()
