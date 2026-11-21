#!/usr/bin/env python3
"""Static/source/asset audit for v0.8. Does NOT compile Rust or run gameplay."""
from pathlib import Path
from collections import deque
import sys

root=Path(__file__).resolve().parents[1]
errors=[]
for rel, keys in {
    'src/game_save.rs':['RUN_CARTER_SAVE_V1','fn validate(', 'fn decode(', 'pub fn load(', 'pub fn save(', '#[cfg(test)]'],
    'src/nav.rs':['pub struct Flow','pub fn new<','pub fn direction(', 'fn flow_reaches_player_around_desk'],
    'src/main.rs':['mod game_save;','mod nav;','fn from_save(', 'fn save_campaign(',
        'fn collect_deliveries(', 'fn end_shift(', 'nav::Flow::new(',
        'KEY_F5','KEY_F9','KEY_F','orders_pending:', 'deliveries_ready:', 'front_visitors:'],
    'src/carteros.rs':['pub fn notify(', 'ctx.orders_pending','ctx.deliveries_ready',
        'ctx.nearest_kid','ctx.front_visitors','END SHIFT (ENTER)'],
}.items():
    file=root/rel
    if not file.exists():errors.append(f'Missing source: {rel}');continue
    code=file.read_text()
    for term in keys:
        if term not in code:errors.append(f'{rel}: no {term}')

# Independent numerical reproduction of the small-grid algorithm's assumptions,
# not a Rust test or guarantee of runtime integration.
step=.15; nx=17; nz=35; minx=-1.31;minz=-2.56
free=lambda x,z: (-1.31<=x<=1.09 and -2.56<=z<=2.57 and not(x>.33 and -2.19<=z<=1.16)
                  and not(x<-.69 and -2.35<=z<=2.53))
tocell=lambda x,z:(round((x-minx)/step),round((z-minz)/step))
toworld=lambda x,z:(minx+x*step,minz+z*step)
def neighbors(x,z):
    return [(a,b) for a,b in ((x-1,z),(x+1,z),(x,z-1),(x,z+1)) if 0<=a<nx and 0<=b<nz]
passable={(x,z) for z in range(nz) for x in range(nx) if free(*toworld(x,z))}
goal=tocell(0,-.8);cost={goal:0};q=deque([goal])
while q:
    cell=q.popleft()
    for cand in neighbors(*cell):
        if cand in passable and cand not in cost:
            cost[cand]=cost[cell]+1;q.append(cand)
pos=(1.0,2.42)
for i in range(500):
    cell=tocell(*pos); options=[p for p in neighbors(*cell) if p in passable and cost.get(p,9999)<cost.get(cell,9999)]
    if not options:break
    target=toworld(*min(options,key=cost.get))
    dx=target[0]-pos[0];dz=target[1]-pos[1];length=(dx*dx+dz*dz)**.5
    velocity=min(.065,length)
    candidate=pos[0]+dx/length*velocity,pos[1]+dz/length*velocity
    if not free(*candidate):errors.append('Numerical navigation check crosses approximate blocker');break
    pos=candidate
if ((pos[0])**2+(pos[1]+.8)**2)**.5>=.5:
    errors.append('Numerical navigation check failed to reach playable target')

if errors:
    print('FAIL\n'+'\n'.join(errors),file=sys.stderr);sys.exit(1)
print('PASS: v0.8 source/asset wiring and independent numerical nav sanity check')
print('NOTE: Rust compiler/type checking, raylib runtime, input UX remain unverified')
