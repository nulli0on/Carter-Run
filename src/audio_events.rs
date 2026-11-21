//! Gameplay/UI sound event queue. Independently testable without raylib or audio hardware.
//! Sound effects are advisory; a missing/undecodable sample never changes game state.
use std::collections::{HashMap,VecDeque};

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum Cue {
    ComputerOpen, Keyboard, MailHandled, ShopPurchase, ShopDenied,
    Delivery, Pickup, WeaponDraw, WeaponFire(u8), WeaponEmpty, Hit,
    Footstep, DoorPound, Airhorn, Confetti, Drone,
    SugarPickup, SugarWarning, Detonation, WaveStart, WaveEnd, PlayerCaught,
}
impl Cue {
    pub fn min_interval_ms(self)->u64 {
        match self {
            Cue::Keyboard=>100, Cue::Footstep=>310, Cue::DoorPound=>2400,
            Cue::WeaponDraw=>140, Cue::WeaponEmpty=>500,
            Cue::ShopDenied=>250, Cue::SugarWarning=>4500,
            Cue::WeaponFire(_)=>0, _=>90,
        }
    }
}
#[derive(Debug,Default)]
pub struct Queue {
    time_ms:u64,
    last:HashMap<Cue,u64>,
    pending:VecDeque<Cue>,
}
impl Queue {
    pub fn advance(&mut self,dt:f32){
        if dt.is_finite() && dt>0.0 {
            self.time_ms=self.time_ms.saturating_add((dt.min(1.0)*1000.0).round() as u64);
        }
    }
    pub fn push(&mut self,cue:Cue)->bool {
        if self.last.get(&cue).is_some_and(|old| self.time_ms.saturating_sub(*old)<cue.min_interval_ms()) {
            return false;
        }
        // Never let a flood of input events accumulate while a sound device is absent.
        if self.pending.len()>=64{self.pending.pop_front();}
        self.last.insert(cue,self.time_ms);
        self.pending.push_back(cue);
        true
    }
    pub fn take(&mut self)->Vec<Cue>{self.pending.drain(..).collect()}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn first_event_always_passes(){let mut q=Queue::default();assert!(q.push(Cue::Keyboard));assert_eq!(q.take(),vec![Cue::Keyboard]);}
    #[test]fn keyboard_does_not_machinegun(){let mut q=Queue::default();assert!(q.push(Cue::Keyboard));assert!(!q.push(Cue::Keyboard));q.advance(0.105);assert!(q.push(Cue::Keyboard));}
    #[test]fn weapon_cooldowns_are_not_overridden(){let mut q=Queue::default();assert!(q.push(Cue::WeaponFire(4)));assert!(q.push(Cue::WeaponFire(4)));assert_eq!(q.take().len(),2);}
    #[test]fn footsteps_throttled(){let mut q=Queue::default();assert!(q.push(Cue::Footstep));q.advance(0.2);assert!(!q.push(Cue::Footstep));q.advance(0.2);assert!(q.push(Cue::Footstep));}
    #[test]fn clock_never_moves_backwards(){let mut q=Queue::default();q.advance(f32::NAN);q.advance(-99.0);q.push(Cue::DoorPound);q.advance(3.0);assert!(q.push(Cue::DoorPound)==false);q.advance(1.5);assert!(!q.push(Cue::DoorPound));q.advance(0.5);assert!(q.push(Cue::DoorPound));}
    #[test]fn queue_is_bounded_and_drains(){let mut q=Queue::default();for _ in 0..80{q.push(Cue::WeaponFire(5));}assert_eq!(q.take().len(),64);assert!(q.take().is_empty());}
}
