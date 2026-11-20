//! Playback for *existing* GLB clips; no synthetic skeletal keyframes.
//! 60 samples/sec matches raylib 6's glTF animation importer.
//! Named clips use their exact authored glTF time ranges. Single `allanims`
//! clips contain unlabelled sequences, so they are played only as full tracks
//! on use; assigning fake draw/reload boundaries would misrepresent the files.
use raylib::prelude::*;
use raylib::core::models::{ModelAnimations,RaylibModel};
use crate::arsenal;

pub const SAMPLE_HZ:f32=60.0;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Motion{Idle,Draw,Fire,Reload,Holster}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Clip{pub index:usize,pub start:f32,pub end:f32,pub looping:bool}
const fn c(index:usize,start:f32,end:f32)->Clip{Clip{index,start,end,looping:false}}
const fn lp(index:usize,start:f32,end:f32)->Clip{Clip{index,start,end,looping:true}}

#[derive(Debug,Clone,Copy)]
pub struct Profile{
    pub idle:Option<Clip>,pub draw:Option<Clip>,pub fire:Option<Clip>,
    pub reload:Option<Clip>,pub holster:Option<Clip>,pub merged:bool,
}
impl Profile{
    pub fn clip(self,kind:Motion)->Option<Clip>{match kind{
        Motion::Idle=>self.idle,Motion::Draw=>self.draw,Motion::Fire=>self.fire,
        Motion::Reload=>self.reload,Motion::Holster=>self.holster,
    }}
}
const fn merged(end:f32)->Profile{Profile{idle:Some(c(0,0.0,0.0)),draw:None,
    fire:Some(c(0,0.0,end)),reload:None,holster:None,merged:true}}
const STATIC:Profile=Profile{idle:None,draw:None,fire:None,reload:None,holster:None,merged:false};

/// Index/trim ranges are from the animation metadata inside the user's v0.15 GLBs.
/// These ranges are **seconds on the original glTF timeline**, not made-up keyframes.
pub const fn profile(slot:usize)->Profile{match slot{
    0=>Profile{idle:Some(c(0,0.042,0.042)),draw:None,fire:Some(c(0,0.042,3.75)),reload:None,holster:None,merged:true},
    1=>merged(4.833),
    2=>Profile{idle:Some(c(3,5.300,5.300)),draw:Some(c(3,4.50,5.333)),fire:Some(c(1,3.767,4.10)),
        reload:Some(c(0,0.033,3.767)),holster:Some(c(2,4.10,4.467)),merged:false},
    3=>Profile{idle:Some(c(0,4.633,4.633)),draw:Some(c(0,4.167,4.667)),fire:Some(c(1,3.367,3.567)),
        reload:Some(c(2,0.033,3.333)),holster:Some(c(3,3.60,4.133)),merged:false},
    4=>Profile{idle:Some(c(0,9.967,9.967)),draw:Some(c(0,8.967,10.0)),fire:Some(c(2,7.533,8.333)),
        reload:Some(c(1,0.033,3.667)),holster:Some(c(4,8.367,8.933)),merged:false},
    5=>merged(6.0),6=>merged(6.5),
    7|8|9=>STATIC,
    10=>merged(5.967),11=>merged(6.5),12=>merged(7.733),13=>merged(15.567),14=>merged(6.0),
    15=>Profile{idle:Some(lp(3,0.0,1.333)),draw:Some(c(6,0.0,1.033)),
        fire:Some(c(1,0.0,0.417)),reload:Some(c(4,0.0,1.133)),
        holster:Some(c(2,0.0,0.333)),merged:false},
    _=>STATIC,
}}

/// Pure animation state machine. Charges are gameplay, not animation frames.
/// Unsupported skeletal actions use controlled procedural viewmodel movement.
pub struct Player{slot:usize,motion:Motion,elapsed:f32}
impl Player{
    pub fn new(slot:usize)->Self{Self{slot,motion:Motion::Draw,elapsed:0.0}}
    pub fn state(&self)->Motion{self.motion}
    pub fn elapsed(&self)->f32{self.elapsed}
    pub fn restart(&mut self){self.motion=Motion::Draw;self.elapsed=0.0;}
    fn fallback_seconds(m:Motion)->f32{match m{
        Motion::Draw=>0.30,Motion::Fire=>0.22,Motion::Reload=>0.65,
        Motion::Holster=>0.24,Motion::Idle=>f32::INFINITY,
    }}
    pub fn is_native(&self)->bool{profile(self.slot).clip(self.motion).is_some()}
    pub fn trigger(&mut self,motion:Motion)->bool{
        if motion==Motion::Fire && self.motion==Motion::Fire{return true;}
        self.motion=motion;self.elapsed=0.0;true
    }
    pub fn procedural_pose(&self)->(f32,f32,f32){
        if self.is_native(){return (0.0,0.0,0.0);}
        let duration=Self::fallback_seconds(self.motion);
        let progress=if duration.is_finite(){(self.elapsed/duration).clamp(0.0,1.0)}else{0.0};
        match self.motion{
            Motion::Draw=>(-0.28*(1.0-progress),0.08*(1.0-progress),0.0),
            Motion::Holster=>(-0.28*progress,0.06*progress,0.0),
            Motion::Fire=>(-0.025*(1.0-progress),0.075*(1.0-progress),0.0),
            Motion::Reload=>(-0.12*(std::f32::consts::PI*progress).sin(),0.02,0.0),
            Motion::Idle=>(0.0,0.0,0.0),
        }
    }
    pub fn step(&mut self,dt:f32)->Option<(usize,f32)>{
        if dt.is_finite() && dt>0.0{self.elapsed+=dt.min(0.1);}
        let Some(clip)=profile(self.slot).clip(self.motion) else {
            if self.motion!=Motion::Idle && self.elapsed>=Self::fallback_seconds(self.motion){
                self.motion=Motion::Idle;self.elapsed=0.0;
            }
            return None;
        };
        let span=(clip.end-clip.start).max(0.0);
        let t=if span<=0.0{0.0}else if clip.looping{self.elapsed.rem_euclid(span)}else{self.elapsed.min(span)};
        let frame=(clip.start+t)*SAMPLE_HZ;
        if !clip.looping && self.elapsed>=span && self.motion!=Motion::Idle{
            self.motion=Motion::Idle;self.elapsed=0.0;
        }
        Some((clip.index,frame))
    }
}

pub struct WeaponVisual{
    pub model:Model,
    animations:Option<ModelAnimations>,
    player:Player,
    center:Vector3,
    scale:f32,
}
impl WeaponVisual{
    pub fn load(rl:&mut RaylibHandle,thread:&RaylibThread,slot:usize)->Option<Self>{
        let weapon=arsenal::CATALOG.get(slot)?;
        let path=format!("assets/arsenal/{}",weapon.asset);
        let model=match rl.load_model(thread,&path){
            Ok(m)=>m,
            Err(e)=>{eprintln!("Weapon mesh slot={slot} asset={} unavailable: {e:?}",weapon.asset);return None;}
        };
        let bbox=model.get_model_bounding_box();
        let extent=(bbox.max.x-bbox.min.x).max(bbox.max.y-bbox.min.y).max(bbox.max.z-bbox.min.z);
        let scale=if extent.is_finite() && extent>1e-5{(0.38/extent).clamp(0.0001,0.9)}else{0.005};
        let center=Vector3::new(
            (bbox.min.x+bbox.max.x)*0.5,(bbox.min.y+bbox.max.y)*0.5,(bbox.min.z+bbox.max.z)*0.5
        );
        let p=profile(slot);
        let animations=rl.load_model_animations(thread,&path).ok().filter(|set|{
            [p.idle,p.draw,p.fire,p.reload,p.holster].iter().flatten().all(|clip|{
                let Some(anim)=set.as_slice().get(clip.index) else{return false;};
                model.is_model_animation_valid(anim) && (anim.as_ref().keyframeCount as f32)>=clip.end*SAMPLE_HZ-1.1
            })
        });
        if animations.is_none() && p.fire.is_some(){
            eprintln!("Weapon slot={slot} asset={path}: incompatible/missing animation; procedural fallback");
        }
        Some(Self{model,animations,player:Player::new(slot),center,scale})
    }
    pub fn trigger(&mut self,motion:Motion)->bool{self.player.trigger(motion)}
    pub fn restart(&mut self){self.player.restart();}
    pub fn update(&mut self,rl:&mut RaylibHandle,thread:&RaylibThread,dt:f32){
        let frame=self.player.step(dt);
        let (Some(anims),Some((index,frame)))=(self.animations.as_ref(),frame) else{return;};
        let Some(anim)=anims.as_slice().get(index) else{return;};
        let count=anim.as_ref().keyframeCount;
        if count>0{rl.update_model_animation(thread,&self.model,anim,frame.clamp(0.0,(count-1) as f32));}
    }
    pub fn status(&self)->(&'static str,usize){
        let mode=match self.player.state(){Motion::Idle=>"idle",Motion::Draw=>"draw",
            Motion::Fire=>"fire",Motion::Reload=>"reload",Motion::Holster=>"holster"};
        (mode,self.animations.as_ref().map_or(0,ModelAnimations::len))
    }
    pub fn animation_count(&self)->usize{self.animations.as_ref().map_or(0,ModelAnimations::len)}
    pub fn auto_scale(&self)->f32{self.scale}
    pub fn center(&self)->Vector3{self.center}
    pub fn view_offsets(&self)->(f32,f32,f32){
        if self.animations.is_some() && self.player.is_native(){(0.0,0.0,0.0)}
        else{self.player.procedural_pose()}
    }
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn named_clip_indices(){
        assert_eq!(profile(4).draw.unwrap().index,0);
        assert_eq!(profile(4).fire.unwrap().index,2);
        assert_eq!(profile(3).fire.unwrap().index,1);
        assert_eq!(profile(15).idle.unwrap().index,3);
    }
    #[test]fn static_prop_slots(){
        for s in [7,8,9]{assert!(profile(s).fire.is_none());}
    }
    #[test]fn no_negative_clip_durations(){
        for s in 0..arsenal::COUNT{
            let p=profile(s);
            for c in [p.idle,p.draw,p.fire,p.reload,p.holster].into_iter().flatten(){
                assert!(c.start>=0.0&&c.end>=c.start);
            }
        }
    }
    #[test]fn named_action_transitions(){
        let mut p=Player::new(4);assert_eq!(p.state(),Motion::Draw);
        assert!(p.step(0.05).unwrap().1>8.967*SAMPLE_HZ);
        for _ in 0..70{p.step(1.0/60.0);}
        assert_eq!(p.state(),Motion::Idle);
        p.trigger(Motion::Fire);assert_eq!(p.step(0.03).unwrap().0,2);
        p.trigger(Motion::Reload);assert_eq!(p.step(0.03).unwrap().0,1);
    }
    #[test]fn merged_track_not_restarted_by_auto_fire(){
        let mut p=Player::new(5);p.trigger(Motion::Fire);
        let a=p.step(0.1).unwrap().1;p.trigger(Motion::Fire);
        let b=p.step(0.1).unwrap().1;assert!(b>a);
    }
    #[test]fn pistol_idle_loops(){
        let mut p=Player::new(15);p.trigger(Motion::Idle);
        for _ in 0..80{p.step(1.0/60.0);}
        assert_eq!(p.step(1.0/60.0).unwrap().0,3);
        assert_eq!(p.state(),Motion::Idle);
    }
    #[test]fn cardboard_has_derived_skin(){
        assert_eq!(profile(0).fire.unwrap().index,0);
        assert_eq!(arsenal::CATALOG[0].asset,"cardboard_blaster_rigged.glb");
    }
    #[test]fn native_held_idle_poses(){
        for s in [2,3,4]{let p=profile(s).idle.unwrap();assert_eq!(p.start,p.end);}
    }
    #[test]fn procedural_holster_and_draw_complete(){
        let mut p=Player::new(7);
        for _ in 0..19{p.step(0.02);}
        assert_eq!(p.state(),Motion::Idle);
        p.trigger(Motion::Holster);
        for _ in 0..13{p.step(0.02);}
        assert_eq!(p.state(),Motion::Idle);
    }
    #[test]fn static_prop_firing_has_procedural_recoil(){
        let mut p=Player::new(8);p.trigger(Motion::Fire);
        assert!(p.step(0.05).is_none());
        assert_eq!(p.state(),Motion::Fire);
        for _ in 0..4{p.step(0.05);}
        assert_eq!(p.state(),Motion::Idle);
    }
}
