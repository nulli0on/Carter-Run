//! Small deterministic 2D navigation flow field for prototype office blockers.
//! Not a collision mesh or baked navmesh: uses the same conservative rectangles
//! as the player and avoids kids walking directly through desks and shelving.
use std::collections::VecDeque;

const UNKNOWN: u16 = u16::MAX;

#[derive(Clone, Copy)]
struct Grid { min_x: f32, min_z: f32, step: f32, nx: usize, nz: usize }

impl Grid {
    fn index(self,x:usize,z:usize)->usize { z*self.nx+x }
    fn cell(self,x:f32,z:f32)->(usize,usize) {
        (((x-self.min_x)/self.step).round().clamp(0.0,(self.nx-1) as f32) as usize,
         ((z-self.min_z)/self.step).round().clamp(0.0,(self.nz-1) as f32) as usize)
    }
    fn world(self,x:usize,z:usize)->(f32,f32) {
        (self.min_x+x as f32*self.step,self.min_z+z as f32*self.step)
    }
    fn neighbors(self,x:usize,z:usize)->impl Iterator<Item=(usize,usize)> {
        [(x.checked_sub(1),Some(z)),(x.checked_add(1).filter(|v|*v<self.nx),Some(z)),
         (Some(x),z.checked_sub(1)),(Some(x),z.checked_add(1).filter(|v|*v<self.nz))]
            .into_iter().filter_map(|(a,b)|Some((a?,b?)))
    }
}

pub fn clear_segment(from: (f32, f32), to: (f32, f32), passable: fn(f32, f32) -> bool) -> bool {
    let dx = to.0 - from.0;
    let dz = to.1 - from.1;
    let steps = (dx.hypot(dz) / 0.04).ceil().max(1.0) as u32;
    (1..=steps).all(|i| {
        let t = i as f32 / steps as f32;
        passable(from.0 + t * dx, from.1 + t * dz)
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Area { Office, RepairShop, District }

#[derive(Default)]
pub struct Pursuit {
    cached: Option<(Area, (f32, f32), Flow)>,
}

impl Pursuit {
    pub fn flow(&mut self, area: Area, target: (f32, f32), passable: fn(f32, f32) -> bool) -> &Flow {
        let reusable = self.cached.as_ref().is_some_and(|(previous_area, previous, flow)| {
            if *previous_area != area { return false; }
            let cell = flow.grid.cell(target.0, target.1);
            *previous == target || (flow.free[flow.grid.index(cell.0, cell.1)]
                && cell == flow.grid.cell(previous.0, previous.1))
        });
        if !reusable {
            let flow = match area {
                Area::District => Flow::district(target.0, target.1),
                _ => Flow::new(target.0, target.1, passable),
            };
            self.cached = Some((area, target, flow));
        }
        &self.cached.as_ref().unwrap().2
    }
}

pub struct Flow { costs: Vec<u16>, free: Vec<bool>, grid: Grid, passable: fn(f32, f32) -> bool }
impl Flow {
    pub fn new(target_x:f32,target_z:f32,passable:fn(f32,f32)->bool)->Self {
        Self::build(Grid{min_x:-1.31,min_z:-2.56,step:0.15,nx:17,nz:35},target_x,target_z,passable)
    }
    pub fn district(target_x:f32,target_z:f32)->Self {
        let (min_x,max_x,min_z,max_z)=crate::district::BOUNDS;
        let step=0.5;
        let grid=Grid{min_x,min_z,step,nx:((max_x-min_x)/step) as usize+1,nz:((max_z-min_z)/step) as usize+1};
        Self::build(grid,target_x,target_z,crate::district::walkable)
    }
    fn build(grid:Grid,target_x:f32,target_z:f32,passable:fn(f32,f32)->bool)->Self {
        let (nx,nz)=(grid.nx,grid.nz);
        let mut free=vec![false;nx*nz];
        for z in 0..nz {for x in 0..nx{
            let (wx,wz)=grid.world(x,z);free[grid.index(x,z)]=passable(wx,wz);
        }}
        let mut costs=vec![UNKNOWN;nx*nz];
        let (gx,gz)=grid.cell(target_x,target_z);
        let goal=if free[grid.index(gx,gz)]{Some((gx,gz))} else {
            // Target near a grid boundary: choose closest traversable grid cell.
            (0..nz).flat_map(|z|(0..nx).map(move |x|(x,z)))
                .filter(|&(x,z)|free[grid.index(x,z)])
                .min_by(|&(a,b),&(c,d)|{
                    let (ax,az)=grid.world(a,b);let (cx,cz)=grid.world(c,d);
                    let aa=(ax-target_x).powi(2)+(az-target_z).powi(2);
                    let bb=(cx-target_x).powi(2)+(cz-target_z).powi(2);
                    aa.total_cmp(&bb)
                })
        };
        if let Some((gx,gz))=goal {
            let mut queue=VecDeque::new();costs[grid.index(gx,gz)]=0;queue.push_back((gx,gz));
            while let Some((x,z))=queue.pop_front(){
                let cost=costs[grid.index(x,z)];
                for (nx,nz) in grid.neighbors(x,z){
                    let n=grid.index(nx,nz);
                    if free[n] && costs[n]==UNKNOWN {
                        costs[n]=cost+1;queue.push_back((nx,nz));
                    }
                }
            }
        }
        Self { costs, free, grid, passable }
    }
    pub fn direction(&self,world_x:f32,world_z:f32)->Option<(f32,f32)> {
        let grid=self.grid;
        let (x,z)=grid.cell(world_x,world_z);
        let (mut bx,mut bz)=(x,z);
        let mut best=self.costs[grid.index(x,z)];
        for (nx,nz) in grid.neighbors(x,z){
            let n=grid.index(nx,nz);
            if self.free[n] && self.costs[n] < best
                && clear_segment((world_x, world_z), grid.world(nx, nz), self.passable)
            {
                best = self.costs[n];
                (bx, bz) = (nx, nz);
            }
        }
        if best==UNKNOWN {return None;}
        if bx==x && bz==z {
            let (tx,tz)=grid.world(bx,bz);
            let d=(world_x-tx).hypot(world_z-tz);
            return if d>0.04 {Some((tx-world_x,tz-world_z))}else{None};
        }
        let (tx,tz)=grid.world(bx,bz);
        Some((tx-world_x,tz-world_z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn office(x:f32,z:f32)->bool {
        (-1.31..=1.09).contains(&x) && (-2.56..=2.57).contains(&z)
            && !(x>0.33 && (-2.19..=1.16).contains(&z))
            && !(x< -0.69 && (-2.35..=2.53).contains(&z))
    }
    #[test] fn flow_reaches_player_around_desk() {
        let flow=Flow::new(0.0,-0.8,office);
        let (mut x,mut z)=(1.0,2.42);
        for _ in 0..400 {
            let Some((dx,dz))=flow.direction(x,z)else{break};
            let dist=dx.hypot(dz);let step=dist.min(0.065);
            let nx=x+dx/dist*step;let nz=z+dz/dist*step;
            assert!(office(nx,nz),"Flow entered obstacle at {nx},{nz}");
            x=nx;z=nz;
        }
        assert!(x.hypot(z+0.8)<0.5,"Kid never reached Carter: {x},{z}");
    }
    #[test] fn target_in_blocker_is_snapped_to_reachable_grid() {
        let f=Flow::new(0.8,0.0,office);
        assert!(f.direction(0.0,1.8).is_some());
    }
}
