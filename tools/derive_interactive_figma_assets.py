"""Derive exact UI fragments from Figma export; no new art or recoloring."""
from pathlib import Path
from lxml import etree
from copy import deepcopy
from PIL import Image
import subprocess, tempfile
ROOT=Path(__file__).resolve().parents[1]; A=ROOT/'assets/carteros'
S=Image.open(A/'full_frame_06.png').convert('RGBA')
S.crop((1095,565,1139,595)).save(A/'toggle_on.png')
S=Image.open(A/'full_frame_05.png').convert('RGBA')
S.crop((1095,694,1139,724)).save(A/'toggle_off.png')
# The original Figma boot screen contains the full-resolution After hours
# background *without* any textual UI inside the wallpaper group.
root=etree.parse(str(ROOT/'design_reference/CarterOS_original_export.svg')).getroot()
pages=[g for g in root if g.tag.endswith('g') and g.get('id','')[:2].isdigit()]
page=pages[11];svg_ns='http://www.w3.org/2000/svg'
svg=etree.Element('{%s}svg'%svg_ns,nsmap=root.nsmap,
                  attrib={'width':'1440','height':'900','viewBox':'4680 2040 1440 900','fill':'none'})
svg.append(deepcopy(page[0]));svg.append(deepcopy(page[1]));svg.append(deepcopy(root[-1]))
with tempfile.TemporaryDirectory() as td:
    sp=Path(td)/'art.svg';sp.write_bytes(etree.tostring(svg,encoding='utf-8',xml_declaration=True))
    p=subprocess.run(['inkscape',str(sp),'--export-filename='+str(A/'wallpaper_after_hours.png'),
        '--export-width=1440','--export-height=900'],capture_output=True,text=True)
    if p.returncode: raise RuntimeError(p.stderr)
for filename in ['toggle_on.png','toggle_off.png','wallpaper_after_hours.png']:
    im=Image.open(A/filename)
    print(filename,im.size)
