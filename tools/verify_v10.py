#!/usr/bin/env python3
"""Source and asset audit only. Does NOT compile Rust or execute gameplay."""
from pathlib import Path
import json,hashlib,re,subprocess,sys
root=Path(__file__).resolve().parents[1]
fail=[]
def require(condition,message):
 if not condition:fail.append(message)
src={p.stem:p.read_text() for p in (root/'src').glob('*.rs')}
for module in ['main','carteros','achievements','debug_console','integrity','game_save','web_search','cheats']:
 require(module in src,'missing module '+module)
 require(src[module].count('{')==src[module].count('}'),'raw brace imbalance in '+module)
all_ids=re.findall(r'\("([A-Z][A-Z_0-9]+)",\s*"',src['achievements'])
require(len(all_ids)==20 and len(set(all_ids))==20,f'expected 20 unique achievements; got {len(all_ids)}')
for s in ['fn sugar(', 'fn emergency_save(', 'fn wave_survived(', 'fn victory(', 'fn purchase(', 'fn cheat(', 'fn tamper(']:
 require(s in src['achievements'],'missing achievement event '+s)
for s in ['fn emergency_bailout(', 'fn take_sugar(', 'fn use_airhorn(', 'fn use_confetti(', 'fn deploy_drone(', 'fn run_debug(', 'fn finish_shift(', 'Action::SearchOutcome', 'KEY_F12', 'Screen::Hearing', 'integrity::verify(']:
 require(s in src['main'],'missing integration '+s)
for s in ['Action::SearchOutcome', 'pending_ai_result', 'fn dynamic_web', 'fn handle_custom_app']:
 require(s in src['carteros'],'missing UI integration '+s)
for domain in ['iamapussypleasegivememoney.com','pleasedonthurtmeimfragile.com', 'thekidsareunionizing.example','calltheitraccoon.example','wheredidmydoorgo.example']:
 require(domain in src['cheats'],'missing Easter egg '+domain)
for s in ['fn funny_mail(', 'jobapplication@raccoon-hr.example']:
 require(s in src['cheats'],'missing email Easter egg '+s)
for s in ['fn protect(', 'fn verify(', 'RUN_CARTER_CHECK_V1']:
 require(s in src['integrity'],'missing integrity helper '+s)
manifest=json.loads((root/'tools/source_asset_hashes.json').read_text())
protected=0
for path,checksum in manifest.items():
 asset=root/path
 require(asset.exists(),'missing protected Figma/asset file '+path)
 if asset.exists():
  require(hashlib.sha256(asset.read_bytes()).hexdigest()==checksum,'changed protected original '+path)
  protected+=1
require(protected>=36,'expected original protected images')
if fail:
 print('FAIL: source/asset checks')
 for x in fail:print(' -',x)
 sys.exit(1)
print(f'PASS: {len(all_ids)} achievement IDs, gameplay/OS hooks, debug commands, new easter eggs')
print(f'PASS: {protected} protected source artwork/design assets identical to the v0.9 manifest')
print('LIMITATION: Cargo compilation, dynamic gameplay testing and Steam unlocking UNVERIFIED')
