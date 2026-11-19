//! Small deterministic 2D navigation flow field for prototype office blockers.
//! Not a collision mesh or baked navmesh: uses the same conservative rectangles
//! as the player and avoids kids walking directly through desks and shelving.
use std::collections::VecDeque;

const STEP:f32=0.15;
const MIN_X:f32=-1.31;
const MIN_Z:f32=-2.56;
const NX:usize=17;
const NZ:usize=35;
const UNKNOWN:u16=u16::MAX;

fn idx(x:usize,z:usize)->usize {z*NX+x}
fn to_cell(x:f32,z:f32)->(usize,usize){
    let cx=((x-MIN_X)/STEP).round().clamp(0.0,(NX-1) as f32) as usize;
    let cz=((z-MIN_Z)/STEP).round().clamp(0.0,(NZ-1) as f32) as usize;
    (cx,cz)
}
fn to_world(x:usize,z:usize)->(f32,f32){(MIN_X+x as f32*STEP,MIN_Z+z as f32*STEP)}
fn neighbors(x:usize,z:usize)->impl Iterator<Item=(usize,usize)> {
    [(x.checked_sub(1),Some(z)),(x.checked_add(1).filter(|v|*v<NX),Some(z)),
     (Some(x),z.checked_sub(1)),(Some(x),z.checked_add(1).filter(|v|*v<NZ))]
        .into_iter().filter_map(|(a,b)|Some((a?,b?)))
}

pub struct Flow {costs:Vec<u16>,free:Vec<bool>}
impl Flow {
    pub fn new<F:Fn(f32,f32)->bool>(target_x:f32,target_z:f32,passable:F)->Self {
        let mut free=vec![false;NX*NZ];
        for z in 0..NZ {for x in 0..NX{
            let (wx,wz)=to_world(x,z);free[idx(x,z)]=passable(wx,wz);
        }}
        let mut costs=vec![UNKNOWN;NX*NZ];
        let (gx,gz)=to_cell(target_x,target_z);
        let goal=if free[idx(gx,gz)]{Some((gx,gz))} else {
            // Target near a grid boundary: choose closest traversable grid cell.
            (0..NZ).flat_map(|z|(0..NX).map(move |x|(x,z)))
                .filter(|&(x,z)|free[idx(x,z)])
                .min_by(|&(a,b),&(c,d)|{
                    let (ax,az)=to_world(a,b);let (cx,cz)=to_world(c,d);
                    let aa=(ax-target_x).powi(2)+(az-target_z).powi(2);
                    let bb=(cx-target_x).powi(2)+(cz-target_z).powi(2);
                    aa.total_cmp(&bb)
                })
        };
        if let Some((gx,gz))=goal {
            let mut queue=VecDeque::new();costs[idx(gx,gz)]=0;queue.push_back((gx,gz));
            while let Some((x,z))=queue.pop_front(){
                let cost=costs[idx(x,z)];
                for (nx,nz) in neighbors(x,z){
                    let n=idx(nx,nz);
                    if free[n] && costs[n]==UNKNOWN {
                        costs[n]=cost+1;queue.push_back((nx,nz));
                    }
                }
            }
        }
        Self{costs,free}
    }
    pub fn direction(&self,world_x:f32,world_z:f32)->Option<(f32,f32)> {
        let (x,z)=to_cell(world_x,world_z);
        let (mut bx,mut bz)=(x,z);
        let mut best=self.costs[idx(x,z)];
        for (nx,nz) in neighbors(x,z){
            let n=idx(nx,nz);
            if self.free[n] && self.costs[n]<best {
                best=self.costs[n];bx=nx;bz=nz;
            }
        }
        if best==UNKNOWN {return None;}
        if bx==x && bz==z {
            let (tx,tz)=to_world(bx,bz);
            let d=(world_x-tx).hypot(world_z-tz);
            return if d>0.04 {Some((tx-world_x,tz-world_z))}else{None};
        }
        let (tx,tz)=to_world(bx,bz);
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
