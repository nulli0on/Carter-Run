"""Extract exact, named visual layers from the user-provided CarterOS SVG.
No redrawing, recoloring, resampling beyond lossless SVG rasterization at 1:1.
Crop transparent padding only. Source SVG remains untouched.
"""
from pathlib import Path
from lxml import etree
from copy import deepcopy
from PIL import Image
import subprocess, json, tempfile, re, os
ROOT=Path(__file__).resolve().parents[1]
SRC=ROOT/'design_reference/CarterOS_original_export.svg'
DEST=ROOT/'assets/carteros'
DEST.mkdir(parents=True,exist_ok=True)
SVG_NS='http://www.w3.org/2000/svg'
tree=etree.parse(str(SRC)); root=tree.getroot(); pagegroups=[g for g in root if g.tag.endswith('g') and 'id' in g.attrib and g.get('id')[:2].isdigit()]
assert len(pagegroups)==16
specs=[
 ('desktop_bg',0,['__rect__','CarterOS · Aurora wallpaper']),
 ('menu_bar',0,['Menu bar']),
 ('dock',0,['Floating Dock']),
 ('desktop_widgets',0,['__rest__'],['__rect__','CarterOS · Aurora wallpaper','Menu bar','Floating Dock']),
 ('launchpad',1,['__rest__'],['__rect__','CarterOS · Aurora wallpaper','Menu bar','Floating Dock']),
 ('notification_center',2,['Notification Center']),
 ('context_menu',2,['Desktop context menu']),
 ('files_window',3,['Files window']),
 ('settings_display',4,['Settings window']),
 ('settings_sound',5,['Settings window']),
 ('settings_network',6,['Settings window']),
 ('settings_privacy',7,['Settings window']),
 ('settings_personalization',8,['Settings window']),
 ('carterweb_inactive',9,['CarterWeb · inactive window']),
 ('cartermail_window',9,['CarterMail · active window']),
 ('lock_screen',10,['__all__']),
 ('boot_screen',11,['__all__']),
 ('leak_bg',13,['__rect__','CarterOS · Aurora wallpaper']),
 ('leak_banner',13,['Location exposure banner']),
 ('leak_modal',13,['Critical · Location leaked']),
 ('leak_radar',13,['KidRadar · approaching fans']),
]
# Extract entire background and window contents exactly; the parent group's clip remains.
# Parent group clip is carried into output preserving original layer references.
manifest={}
for index,(name,frameix,targets,*omit) in enumerate(specs):
    group=pagegroups[frameix]
    x=(frameix%4)*1560; y=(frameix//4)*1020
    new=etree.Element('{%s}svg'%SVG_NS,nsmap=root.nsmap)
    new.attrib.update({'width':'1440','height':'900','viewBox':f'{x} {y} 1440 900','fill':'none'})
    main=etree.SubElement(new,'{%s}g'%SVG_NS)
    if group.get('clip-path'): main.set('clip-path',group.get('clip-path'))
    for i,el in enumerate(group):
        label=el.get('id','')
        norm=lambda s:re.sub('[^a-z0-9]','',s.lower())
        wanted=('__all__' in targets or ('__rest__' in targets and norm(label) not in set(map(norm,omit[0])) and not (el.tag.endswith('rect') and i==0)) or ('__rect__' in targets and el.tag.endswith('rect') and i==0) or any(norm(t) in norm(label) for t in targets if not t.startswith('__')))
        if wanted: main.append(deepcopy(el))
    new.append(deepcopy(root[-1])) # defs
    temp=DEST/(name+'_work.svg'); png=DEST/(name+'_full.png')
    temp.write_bytes(etree.tostring(new,encoding='utf-8',xml_declaration=True))
    p=subprocess.run(['inkscape',str(temp),'--export-type=png','--export-filename='+str(png),'--export-width=1440','--export-height=900'],capture_output=True,text=True,timeout=75)
    if p.returncode!=0: print(name,p.stderr[-1000:]);raise RuntimeError(name)
    im=Image.open(png).convert('RGBA')
    bbox=im.getbbox()
    if bbox is None:raise RuntimeError(name+' has no visible image')
    cropped=im.crop(bbox)
    out=DEST/(name+'.png');cropped.save(out,optimize=True)
    manifest[name]={'file':name+'.png','x':bbox[0],'y':bbox[1],'width':cropped.width,'height':cropped.height,'frame':frameix+1}
    temp.unlink();png.unlink()
    print(f'[{index+1:02d}/{len(specs)}] {name}: {bbox}',flush=True)
(DEST/'asset_manifest.json').write_text(json.dumps(manifest,indent=2))
(DEST/'SOURCE_ATTRIBUTION.txt').write_text('Original Figma source layers from design_reference/CarterOS_original_export.svg. No visual redesign. Source artboards: 1440x900.\n')
for frame in [0,1,3,4,5,6,7,8,9,10,11,13]:
    x=(frame%4)*1560; y=(frame//4)*1020
    svg=etree.Element('{%s}svg'%SVG_NS, nsmap=root.nsmap)
    svg.attrib.update({'width':'1440','height':'900','viewBox':f'{x} {y} 1440 900','fill':'none'})
    svg.append(deepcopy(pagegroups[frame]));svg.append(deepcopy(root[-1]))
    temp=DEST/'_tmp_full.svg'
    temp.write_bytes(etree.tostring(svg,encoding='utf-8',xml_declaration=True))
    png=DEST/f'full_frame_{frame+1:02d}.png'
    subprocess.run(['inkscape',str(temp),'--export-filename='+str(png),'--export-width=1440','--export-height=900'],check=True,timeout=75)
    temp.unlink()
print('Complete: 21 transparent source layers and 12 original full-artboard exports')
