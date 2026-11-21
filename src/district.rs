//! The playable block between the chute and the New Office entrance.
use raylib::prelude::*;

pub const START: (f32, f32) = (0.0, -17.0);
pub const ENTRANCE: (f32, f32) = (-2.5, 14.3);
pub const BOUNDS: (f32, f32, f32, f32) = (-16.0, 16.0, -20.0, 18.0);

// Center x, center z, width, depth, height. Used for sight, collision and rendering.
const BLOCKS: [(f32, f32, f32, f32, f32); 7] = [
    (-2.5, 17.0, 8.0, 3.5, 5.5),
    (-11.0, 3.0, 6.0, 9.0, 4.4),
    (11.0, 7.0, 5.0, 8.0, 4.2),
    (1.8, -6.0, 6.5, 2.4, 1.4),
    (-3.8, 2.0, 4.8, 2.4, 1.4),
    (2.3, 8.0, 5.0, 1.7, 1.1),
    (-4.8, -18.4, 6.0, 2.1, 4.0),
];

pub fn bounded(x: f32, z: f32) -> bool {
    let (min_x,max_x,min_z,max_z) = BOUNDS;
    (min_x..=max_x).contains(&x) && (min_z..=max_z).contains(&z)
}

pub fn passable(x: f32, y: f32, z: f32, margin: f32) -> bool {
    bounded(x,z) && y>=0.0 && !BLOCKS.iter().any(|&(cx,cz,w,d,h)| {
        (x-cx).abs() < w*0.5+margin && (z-cz).abs() < d*0.5+margin && y<=h
    })
}

pub fn walkable(x: f32, z: f32) -> bool { passable(x,0.0,z,0.22) }

pub fn at_entrance(x: f32, z: f32) -> bool {
    (x-ENTRANCE.0).hypot(z-ENTRANCE.1)<1.1
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pickup { Spare, Decoys }

impl Pickup {
    fn location(self) -> (f32, f32) {
        match self { Self::Spare => (-10.5,-4.5), Self::Decoys => (11.5,-2.0) }
    }

    pub fn nearby(x: f32, z: f32) -> Option<Self> {
        [Self::Spare, Self::Decoys].into_iter().find(|s| {
            let (sx,sz) = s.location();
            (sx-x).hypot(sz-z) <= 1.6
        })
    }
}

// The apron east of the chute is open across all three reinforcement rows.
pub fn arrival(i: usize) -> (f32, f32) {
    (-0.8 + (i % 9) as f32 * 0.70, -19.55 + (i / 9) as f32 * 0.68)
}

pub fn draw<D: RaylibDraw3D>(world: &mut D, claimed: bool) {
    let asphalt=Color::new(69,78,88,255);
    let concrete=Color::new(144,152,153,255);
    let back=Color::new(58,76,88,255);
    let blue=Color::new(63,217,207,255);
    world.draw_cube(Vector3::new(0.0,-0.14,-1.0),33.0,0.26,39.0,asphalt);
    world.draw_cube(Vector3::new(-13.7,-0.065,-1.0),4.0,0.12,38.0,concrete);
    world.draw_cube(Vector3::new(14.4,-0.065,-1.0),2.8,0.12,38.0,concrete);
    for z in (-16..14).step_by(3) {
        world.draw_cube(Vector3::new(0.0,0.01,z as f32),0.14,0.015,1.1,Color::new(216,205,156,255));
    }
    for (i,&(x,z,w,d,h)) in BLOCKS.iter().enumerate() {
        let tint = [back,Color::new(135,100,88,255),Color::new(110,127,143,255),
            Color::new(39,86,94,255),Color::new(132,80,53,255),Color::DARKGREEN,back][i];
        world.draw_cube(Vector3::new(x,h/2.0,z),w,h,d,tint);
        world.draw_cube(Vector3::new(x,h+0.07,z),w+0.16,0.14,d+0.16,if h>2.0 {concrete} else {back});
        if h>2.0 {
            for offset in [-0.31,0.0,0.31] {
                world.draw_cube(Vector3::new(x+w*offset,h*0.70,z-d/2.0-0.02),w*0.19,h*0.19,0.05,Color::new(106,164,184,255));
            }
        }
    }
    for pickup in [Pickup::Spare,Pickup::Decoys] {
        let (x,z)=pickup.location();
        let tint=if claimed {Color::GRAY} else {Color::GOLD};
        world.draw_cube(Vector3::new(x,0.03,z),1.7,0.06,1.7,tint);
        world.draw_cube(Vector3::new(x,0.47,z),0.96,0.85,0.78,back);
        world.draw_cube(Vector3::new(x,0.68,z),0.98,0.12,0.80,tint);
        if !claimed {
            world.draw_cube(Vector3::new(x,1.25,z),0.06,0.8,0.06,concrete);
            world.draw_sphere(Vector3::new(x,1.72,z),0.18,tint);
        }
    }
    let (x,z)=ENTRANCE;
    world.draw_cube(Vector3::new(x,0.015,z),2.9,0.03,2.1,blue);
    world.draw_cube(Vector3::new(x,1.19,15.20),1.65,2.38,0.16,Color::new(23,50,61,255));
    world.draw_cube(Vector3::new(x,2.6,15.13),2.8,0.14,0.25,blue);
    world.draw_cube(Vector3::new(x,6.75,16.8),0.16,2.2,0.15,blue);
    world.draw_sphere(Vector3::new(x,7.93,16.8),0.36,blue);
    world.draw_cube(Vector3::new(0.0,0.5,-19.7),32.0,1.0,0.25,back);
    for x in [-16.1,16.1] {
        world.draw_cube(Vector3::new(x,0.83,-1.0),0.20,1.66,38.0,back);
    }
}

pub fn minimap<D: RaylibDraw>(d: &mut D, player: (f32,f32), fans: impl Iterator<Item=(f32,f32)>, yaw: f32, claimed: bool, w: i32) {
    let (left,top)=(w-178,79);
    d.draw_rectangle(left,top,159,185,Color::new(12,25,39,238));
    let project=|x:f32,z:f32| (left+9+((x+16.0)/32.0*140.0) as i32,top+9+((18.0-z)/38.0*149.0) as i32);
    for &(x,z,w,dz,_) in &BLOCKS {
        let (px,py)=project(x-w/2.0,z+dz/2.0);
        d.draw_rectangle(px,py,(w*4.375) as i32,(dz*3.92) as i32,Color::new(82,105,119,255));
    }
    let (x,z)=project(ENTRANCE.0,ENTRANCE.1);
    d.draw_circle(x,z,5.0,Color::LIME);
    if !claimed {
        for (pickup,label) in [(Pickup::Spare,"PC"),(Pickup::Decoys,"Pi")] {
            let (sx,sz)=pickup.location();
            let (x,z)=project(sx,sz);
            d.draw_circle(x,z,3.0,Color::GOLD);
            d.draw_text(label,x+4,z-6,12,Color::GOLD);
        }
    }
    for (x,z) in fans {
        let (x,z)=project(x,z);
        d.draw_circle(x,z,2.0,Color::ORANGE);
    }
    let (x,z)=project(player.0,player.1);
    d.draw_line(x,z,x+(yaw.sin()*10.0) as i32,z-(yaw.cos()*10.0) as i32,Color::WHITE);
    d.draw_circle(x,z,4.0,Color::WHITE);
    d.draw_text("NEW OFFICE",left+14,top+163,15,Color::LIME);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_around_the_block_connects_chute_and_new_office() {
        let waypoints=[START,(6.5,-17.0),(6.5,13.6),(-2.5,13.6),ENTRANCE];
        for pair in waypoints.windows(2) {
            let [(ax,az),(bx,bz)]=[pair[0],pair[1]];
            for step in 0..=100 {
                let t=step as f32/100.0;
                assert!(walkable(ax+(bx-ax)*t,az+(bz-az)*t));
            }
        }
        assert!(at_entrance(ENTRANCE.0,ENTRANCE.1));
        assert!(!at_entrance(START.0,START.1));
    }
}
