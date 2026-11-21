#!/usr/bin/env python3
"""Source, Figma and SDK contract audit; NOT a Rust compiler or runtime test."""
from pathlib import Path
import hashlib,json,subprocess,sys
r=Path(__file__).resolve().parents[1]
base_hashes=json.loads((r/'tools/source_asset_hashes.json').read_text())
issues=[]
def fail(msg):issues.append(msg)
for n in ['main.rs','carteros.rs','cheats.rs','web_search.rs','steam_bridge.rs']:
 p=r/'src'/n
 if not p.is_file():fail(f'missing Rust source: {n}')
 else:
  s=p.read_text()
  if s.count('{')!=s.count('}'):fail(f'unbalanced raw brace count: {n}')
source=(r/'src/cheats.rs').read_text(); main=(r/'src/main.rs').read_text(); os=(r/'src/carteros.rs').read_text();web=(r/'src/web_search.rs').read_text();steam=(r/'src/steam_bridge.rs').read_text()
provided={
 'iamapussypleasegivememoney.com','pleasedonthurtmeimfragile.com','mommytheyreshootingatme.org',
 'icantdodgeanything.net','idontwanttoplaythegame.com','skipstraighttothecredits.org',
 'carrymepleaseimtrash.gg','ineedaimbotandwallhacks.net','giveittomebecauseicantaim.com',
 'skillissuecompensation.org','itwasjustaprankbro.gov','officerididntdoanything.com',
 'witnessprotectionforidiots.org','helpimstuckinawallagain.com','getmeoutofhereimscared.net'
}
for domain in provided:
 if f'"{domain}"' not in source:fail('missing provided URL '+domain)
for needle in ['fn website(','fn mail(','fn browser_easter_egg(','struct CheatState','SHAME_SHAME_SHAME','fn applied(']:
 if needle not in source:fail('missing cheat logic '+needle)
for needle in ['Cheat(CheatKind)','pub fn tick','self.web.submit','self.web.show_secret','self.mail_compose','draw_compose_overlay','fn dynamic_web']:
 if needle not in os:fail('missing CarterOS integration '+needle)
for needle in ['fn use_cheat(','CheatKind::Money','CheatKind::GodMode','CheatKind::MaxStats','CheatKind::Items','CheatKind::ClearAggro','CheatKind::Noclip','mod web_search;','mod steam_bridge;']:
 if needle not in main:fail('missing gameplay integration '+needle)
if 'KEY_ESCAPE)||rl.is_key_pressed(KeyboardKey::KEY_E)' in main:fail('E key still breaks text input')
for needle in ['CARTEROS_AI_MODEL','Command::new("ollama")','std::thread::spawn','AnswerKind::Ai','AnswerKind::Comedic','AnswerKind::NoResults','AnswerKind::OfflineFallback']:
 if needle not in web:fail('missing AI/search behavior '+needle)
for needle in ['SteamAPI_InitFlat','SteamAPI_SteamUserStats_v013','SteamAPI_ISteamUserStats_SetAchievement','SteamAPI_ISteamUserStats_StoreStats']:
 if needle not in steam:fail('Steam interface missing '+needle)
for filename in ['steam/linux64/libsteam_api.so','steam/win64/steam_api64.dll','steam/win64/steam_api64.lib']:
 if not (r/filename).exists():fail('missing redistributable '+filename)
# Verify all protected originals against the historical SHA-256 manifest.
count=0
for rel,expected in base_hashes.items():
    path=r/rel
    if not path.is_file():fail('protected asset missing: '+rel);continue
    actual=hashlib.sha256(path.read_bytes()).hexdigest()
    if actual!=expected:fail('original protected asset changed: '+rel)
    if rel.startswith('assets/carteros/') and rel.endswith('.png'):count+=1
if count!=36:fail(f'expected 36 protected PNG art assets, got {count}')
# Ensure every Steam symbol in binding is actually exported by user's linux binary.
try:
 nm=subprocess.run(['nm','-D',str(r/'steam/linux64/libsteam_api.so')],capture_output=True,text=True,check=True).stdout
 for n in ['SteamAPI_InitFlat','SteamAPI_SteamUserStats_v013','SteamAPI_ISteamUserStats_SetAchievement','SteamAPI_ISteamUserStats_StoreStats']:
  if n not in nm:fail('SDK export absent '+n)
except Exception as e:fail('could not inspect SDK binary: '+str(e))
if issues:
 print('FAIL')
 for issue in issues:print(' -',issue)
 sys.exit(1)
print('PASS: all 15 user cheat URL names, typed search and mail wiring, local AI modes, Steam SDK exports')
print('PASS: original Figma .fig, SVG, 36 PNGs and repaired office model are byte-for-byte unchanged')
print('WARNING: no Rust compiler available; compilation, runtime UI and live Steam unlock are UNVERIFIED')
