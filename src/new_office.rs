//! Geometry and interaction points for the service district's converted repair shop.
use raylib::prelude::*;

pub const ARRIVAL: (f32, f32) = (0.0, 1.48);
pub const DESK: (f32, f32) = (-0.32, -0.86);
pub const DELIVERY: (f32, f32) = (0.69, 1.94);
pub const CHUTE: (f32, f32) = (0.41, -2.39);
pub const PICKUP: (f32, f32) = (0.32, -1.54);
pub const CAMERA: (f32, f32) = (0.78, 2.23);

// Center, width, depth and height. Rendered furniture and movement use the same volumes.
const BENCH: [f32; 5] = [-0.95, -0.72, 0.63, 2.02, 0.90];
const RACK: [f32; 5] = [0.87, 0.32, 0.44, 1.53, 1.84];

pub fn passable(x: f32, y: f32, z: f32, margin: f32) -> bool {
    (-1.31..=1.09).contains(&x) && (-2.56..=2.57).contains(&z) && (0.0..=2.7).contains(&y)
        && ![BENCH, RACK].iter().any(|&[cx, cz, width, depth, height]| {
            (x-cx).abs()<width/2.0+margin && (z-cz).abs()<depth/2.0+margin && y<=height
        })
}

pub fn walkable(x: f32, z: f32) -> bool { passable(x,0.0,z,0.12) }

pub fn draw<D: RaylibDraw3D>(world: &mut D) {
    let wall=Color::new(66,79,83,255);
    let steel=Color::new(39,52,60,255);
    let mat=Color::new(50,87,94,255);
    let ivory=Color::new(222,216,190,255);
    let amber=Color::new(237,177,97,255);
    let mint=Color::new(106,221,184,255);
    let floor=Color::new(100,107,105,255);
    let cube=|world:&mut D,x,y,z,w,h,d,color| world.draw_cube(Vector3::new(x,y,z),w,h,d,color);
    cube(world,-0.11,-0.10,0.0,2.60,0.19,5.36,floor);
    cube(world,-0.11,2.79,0.0,2.60,0.18,5.36,steel);
    cube(world,-1.41,1.37,0.0,0.20,2.74,5.36,wall);
    cube(world,1.19,1.37,0.0,0.20,2.74,5.36,wall);
    cube(world,-0.11,1.37,-2.65,2.60,2.74,0.18,wall);
    cube(world,-0.11,1.37,2.66,2.60,2.74,0.18,wall);
    cube(world,0.0,1.04,2.55,0.96,2.06,0.10,steel);
    cube(world,0.0,2.13,2.48,1.09,0.16,0.12,mint);
    cube(world,0.29,0.99,2.47,0.17,0.05,0.10,amber);
    cube(world,-0.11,0.95,-2.54,2.36,0.07,0.025,amber);
    cube(world,0.91,0.012,1.94,0.34,0.02,0.84,amber);
    cube(world,0.47,0.012,1.94,0.045,0.02,0.84,amber);
    cube(world,-0.33,0.013,-0.87,0.59,0.02,1.95,mat);
    for z in [-1.56, -0.2, 1.24] {
        cube(world,-0.12,2.68,z,1.55,0.07,0.11,ivory);
    }
    let [x,z,w,d,h]=BENCH;
    cube(world,x,h-0.05,z,w,0.10,d,ivory);
    for z in [z-d*0.42,z+d*0.42] {cube(world,x,0.42,z,w*0.75,0.84,0.11,steel);}
    cube(world,-1.17,1.45,-0.86,0.17,0.56,0.94,steel);
    cube(world,-1.076,1.45,-0.86,0.025,0.49,0.86,mint);
    cube(world,-0.86,0.925,-0.86,0.27,0.045,0.52,steel);
    cube(world,-1.09,1.075,-1.43,0.31,0.35,0.22,steel);
    cube(world,-0.93,1.11,-1.43,0.02,0.10,0.13,amber);
    let [x,z,w,d,h]=RACK;
    for y in [0.13,0.72,1.29,h-0.08] {
        cube(world,x,y,z,w,0.085,d,steel);
    }
    for z in [z-d*0.46,z+d*0.46] {cube(world,x,0.92,z,w,1.78,0.085,steel);}
    for (z,y,color) in [(0.06,0.29,ivory),(0.58,0.89,mat),(0.25,1.46,amber)] {
        cube(world,x,y,z,w*0.80,0.23,0.29,color);
    }
    // Interior glazing above the bench and a frosted view toward the side lot.
    cube(world,-1.298,1.99,0.87,0.015,0.64,1.32,steel);
    cube(world,-1.288,1.99,0.87,0.015,0.55,1.22,Color::new(112,163,181,255));
    cube(world,-1.277,1.99,0.87,0.018,0.55,0.05,steel);
}
