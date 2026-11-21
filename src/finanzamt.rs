//! Secret finale. Arena loadout is a copy of the office inventory.
use raylib::prelude::*;
use crate::arsenal::{self, Arsenal};

const CENTER: Vector2 = Vector2 {x:0.5,y:0.29};

fn distance(a: Vector2, b: Vector2) -> f32 { (a.x-b.x).hypot(a.y-b.y) }
fn toward(a: Vector2, b: Vector2) -> Vector2 {
    let d = distance(a,b).max(0.0001);
    Vector2::new((b.x-a.x)/d,(b.y-a.y)/d)
}
fn step(a: Vector2, dir: Vector2, amount: f32) -> Vector2 {
    Vector2::new(a.x+dir.x*amount,a.y+dir.y*amount)
}

#[derive(Default)]
pub struct Input {
    pub movement: Vector2, pub aim: Vector2, pub fire: bool, pub held: bool,
    pub select: Option<usize>, pub cycle: i8, pub gadget: Option<usize>, pub sprint: bool,
    pub boost: bool,
}

struct Paper { pos:Vector2, heading:Vector2, surcharge:bool }
struct Agent { pos: Vector2, shield: f32 }
enum Attack { Notice { aim: Vector2, warning:f32, surcharge:bool }, Deadline { target: Vector2, warning: f32 } }

pub struct Arena {
    pub result: Option<bool>,
    pub ending: bool,
    follow_up: bool,
    kit: Arsenal,
    supplies: [u32; 5],
    player: Vector2,
    hp: u8,
    boss_hp: f32,
    cooldown: f32,
    clock: f32,
    wave: f32,
    attack_no: u32,
    attack: Option<Attack>,
    papers: Vec<Paper>,
    agents: Vec<Agent>,
    invulnerable: f32,
    decoy: Option<(Vector2, f32)>,
    burst: Option<(Vector2, f32)>,
    boost_time: f32,
    text: &'static str,
}

impl Arena {
    pub fn new(mut kit: Arsenal, supplies: [u32;5]) -> Self {
        kit.cooldown = 0.0;
        Self {
            result:None,ending:false,follow_up:false,kit, supplies, player: Vector2::new(0.5,0.83), hp: 5, boss_hp: 90.0,
            cooldown: 0.0, clock: 2.0, wave: 6.5, attack_no: 0, attack: None,
            papers: Vec::new(), agents: Vec::new(), invulnerable: 0.0, decoy: None,
            burst: None, boost_time: 0.0, text: "Guilty. Appeal denied. Please fight the paperwork.",
        }
    }

    pub fn close(mut self, won: bool) -> Self { self.result = Some(won); self }

    fn damage(&mut self) {
        if self.invulnerable <= 0.0 && self.result.is_none() {
            self.hp = self.hp.saturating_sub(1);
            self.invulnerable = 1.05;
            self.text = "A filing fee hits Carter.";
            if self.hp == 0 { self.result = Some(false); }
        }
    }

    pub fn tick(&mut self, dt: f32, input: Input) {
        if self.result.is_some() { return; }
        self.cooldown -= dt;
        self.invulnerable -= dt;
        self.kit.tick(dt);
        self.boost_time = (self.boost_time - dt).max(0.0);
        if let Some(i) = input.select { self.kit.equip(i); }
        if input.cycle != 0 { self.kit.cycle(input.cycle > 0); }
        let speed = if input.sprint { 0.43 } else { 0.32 };
        let next = step(self.player, input.movement, speed * if self.boost_time>0.0 { 1.32 } else { 1.0 } * dt);
        self.player = Vector2::new(next.x.clamp(0.055,0.945), next.y.clamp(0.22,0.91));

        if input.boost && self.supplies[4] > 0 && self.boost_time <= 0.0 {
            self.supplies[4] -= 1;
            self.boost_time = 5.0;
            self.text = "Fictional rush: movement boosted.";
        }
        if let Some(i) = input.gadget.filter(|&i| self.supplies[i] > 0) {
            self.supplies[i] -= 1;
            match i {
                0 | 3 => { self.decoy = Some((input.aim, if i==3 {6.0} else {3.0})); self.text = "The agents investigate the decoy."; }
                1 => { self.agents.retain(|a| distance(a.pos,self.player)>0.24); self.text = "A loud objection clears nearby agents."; }
                2 => { self.agents.clear(); self.text = "Confetti clears the active agents."; }
                _ => {}
            }
        }
        if let Some((_, time)) = &mut self.decoy { *time -= dt; if *time<=0.0 { self.decoy=None; } }
        if let Some((_, time)) = &mut self.burst { *time -= dt; if *time<=0.0 { self.burst=None; } }

        // Empty equipment falls back to a push-stamp, so the finale never requires a specific purchase.
        let automatic = matches!(self.kit.selected,3..=6|13);
        if self.cooldown <= 0.0 && (input.fire || (automatic && input.held)) {
            let slot = self.kit.selected;
            let native = self.kit.fire();
            let weapon = native.unwrap_or(arsenal::CATALOG[0]);
            let power = if native.is_some() { 3.0 + weapon.burst as f32 * 0.7 } else { 1.8 };
            self.cooldown = if native.is_some() { weapon.cooldown.clamp(0.18,1.0)*0.75 } else { 0.48 };
            let radius = if native.is_some() && matches!(slot,6..=10|14) { 0.10 } else { 0.054 };
            self.burst = Some((input.aim,0.14));
            if distance(input.aim,CENTER)<0.105 && self.agents.is_empty() {
                self.boss_hp = (self.boss_hp-power).max(0.0);
                self.text = if native.is_some() { "Objection sustained." } else { "Emergency stamp applied." };
            } else if distance(input.aim,CENTER)<0.105 {
                self.text = "Clear the clerks to drop the paperwork shield.";
            }
            for agent in &mut self.agents {
                if distance(input.aim,agent.pos)<radius { agent.shield -= if native.is_some() {2.0} else {1.0}; }
            }
            self.agents.retain(|a| a.shield>0.0);
            if self.boss_hp<=0.0 { self.result=Some(true); return; }
        }

        self.wave -= dt;
        if self.wave <= 0.0 {
            self.wave = if self.boss_hp<=45.0 { 9.0 } else { 11.0 };
            if self.agents.is_empty() {
                let positions = [Vector2::new(0.16,0.42),Vector2::new(0.84,0.42),Vector2::new(0.5,0.51)];
                let count = if self.boss_hp<=45.0 {3} else {2};
                self.agents.extend(positions.into_iter().take(count).map(|pos| Agent{pos,shield:2.0}));
                self.text = "The Finanzamt summons a wave of clerks.";
            }
        }
        let mut contact = false;
        for a in &mut self.agents {
            let target = self.decoy.map_or(self.player, |(p,_)| p);
            if distance(a.pos,target)>0.025 { a.pos = step(a.pos,toward(a.pos,target),dt*0.15); }
            if distance(a.pos,self.player)<0.045 { contact = true; }
        }
        if contact { self.damage(); }
        if self.result.is_some() { return; }

        self.clock -= dt;
        if self.clock<=0.0 && self.attack.is_none() {
            self.attack = Some(if self.follow_up {
                self.follow_up=false;
                self.text="NACHFORDERUNG! The second notice is aimed at your new position.";
                Attack::Notice {aim:toward(CENTER,self.player),warning:0.48,surcharge:true}
            } else {
                self.attack_no+=1;
                if self.attack_no%3==0 {
                self.text = "FRIST ABGELAUFEN! Leave the giant red stamp circle.";
                Attack::Deadline {target:self.player,warning:1.1}
                } else { Attack::Notice {aim:toward(CENTER,self.player),warning:0.58,surcharge:false} }
            });
        }
        match self.attack.take() {
            Some(Attack::Notice {aim,warning,surcharge}) if warning<=dt => {
                self.papers.push(Paper {pos:step(CENTER,aim,0.08),heading:aim,surcharge});
                self.follow_up=!surcharge && self.boss_hp<=45.0;
                self.clock=if self.follow_up {0.72} else {1.65};
            }
            Some(Attack::Deadline {target,warning}) if warning<=dt => {
                if distance(target,self.player)<0.16 { self.damage(); }
                self.burst=Some((target,0.32));
                self.clock=1.8;
            }
            Some(Attack::Notice {aim,warning,surcharge}) => self.attack=Some(Attack::Notice{aim,warning:warning-dt,surcharge}),
            Some(Attack::Deadline {target,warning}) => self.attack=Some(Attack::Deadline{target,warning:warning-dt}),
            None=>{}
        }
        let mut hit = false;
        self.papers.retain_mut(|paper| {
            paper.pos = step(paper.pos,paper.heading,dt*0.65);
            if distance(paper.pos,self.player)<0.05 { hit=true; return false; }
            (0.0..=1.0).contains(&paper.pos.x) && (0.0..=1.0).contains(&paper.pos.y)
        });
        if hit { self.damage(); }
    }

    pub fn draw(&self, d: &mut impl RaylibDraw) {
        let (w, h) = (crate::carteros::W, crate::carteros::H);
        let locate=|p:Vector2| ((p.x*w as f32) as i32,(p.y*h as f32) as i32);
        d.clear_background(Color::new(18,24,25,255));
        if self.ending {
            d.draw_rectangle(w/2-343,h/2-150,686,292,Color::new(238,226,197,255));
            d.draw_rectangle_lines(w/2-327,h/2-134,654,260,Color::MAROON);
            d.draw_text("FINANZAMT  /  SECRET ENDING",w/2-277,h/2-101,24,Color::MAROON);
            d.draw_text("THE FINAL NOTICE",w/2-277,h/2-53,40,Color::BLACK);
            d.draw_text("The clerk stamps the giant file: CLOSED.",w/2-277,h/2+21,22,Color::BLACK);
            d.draw_text("One last envelope slides under Carter's door. It's blank.",w/2-277,h/2+58,18,Color::DARKGRAY);
            d.draw_text("Enter  /  Return to title",w/2-151,h/2+173,18,Color::SKYBLUE);
            return;
        }
        d.draw_rectangle(w/20,h/5,w*9/10,h*73/100,Color::new(41,50,51,255));
        for x in (w/20..w*19/20).step_by(54) {
            d.draw_line(x,h/5,x,h*93/100,Color::new(49,60,60,255));
        }
        d.draw_text("FINANZAMT",32,24,32,Color::RAYWHITE);
        d.draw_text("Secret finale  /  Office of unreasonable correspondence",32,64,18,Color::LIGHTGRAY);
        d.draw_text(&format!("CARTER  {} / 5",self.hp),32,102,22,Color::SKYBLUE);
        d.draw_text(&format!("FILE INTEGRITY  {:.0} / 90",self.boss_hp),w-315,31,22,Color::WHITE);
        d.draw_rectangle(w-315,63,270,10,Color::DARKGRAY);
        d.draw_rectangle(w-315,63,(3.0*self.boss_hp) as i32,10,Color::ORANGE);
        let (bx,by)=locate(CENTER);
        if !self.agents.is_empty() { d.draw_circle_lines(bx,by,74.0,Color::SKYBLUE); }
        d.draw_rectangle(bx-57,by-31,114,59,Color::BEIGE);
        d.draw_rectangle(bx-44,by-17,88,17,Color::BROWN);
        d.draw_text("FINANZAMT",bx-46,by-12,15,Color::WHITE);
        d.draw_text("AKTE 001",bx-35,by+10,13,Color::MAROON);
        if let Some(attack)=&self.attack {
            match attack {
                Attack::Notice{aim,surcharge,..}=>{
                    let (x,y)=locate(step(CENTER,*aim,1.15));
                    d.draw_line(bx,by,x,y,if *surcharge {Color::PINK} else {Color::GOLD});
                }
                Attack::Deadline{target,..}=>{
                    let (x,y)=locate(*target);
                    let radius=0.16*w as f32;
                    d.draw_ellipse(x,y,radius,0.16*h as f32,Color::new(191,46,52,106));
                    d.draw_ellipse_lines(x,y,radius,0.16*h as f32,Color::PINK);
                    d.draw_text("FRIST ABGELAUFEN",x-91,y-8,17,Color::WHITE);
                }
            }
        }
        for paper in &self.papers {
            let (x,y)=locate(paper.pos);
            d.draw_rectangle(x-57,y-13,114,27,if paper.surcharge {Color::PINK}else{Color::RAYWHITE});
            d.draw_text(if paper.surcharge {"NACHFORDERUNG"}else{"STEUERBESCHEID"},x-50,y-6,12,Color::MAROON);
        }
        for agent in &self.agents {
            let (x,y)=locate(agent.pos);
            d.draw_circle(x,y,15.0,Color::ORANGE);
            d.draw_text("§",x-6,y-9,16,Color::BLACK);
        }
        if let Some((point,_))=self.decoy {
            let (x,y)=locate(point); d.draw_circle_lines(x,y,26.0,Color::LIME);
        }
        if let Some((point,_))=self.burst {
            let (x,y)=locate(point); d.draw_circle_lines(x,y,14.0,Color::YELLOW);
        }
        let (x,y)=locate(self.player);
        d.draw_circle(x,y,16.0,if self.invulnerable>0.0 {Color::PINK} else {Color::SKYBLUE});
        let it=&arsenal::CATALOG[self.kit.selected];
        d.draw_text(&format!("{}  /  {} charges  /  Q Pi {}   Z horn {}   X confetti {}   C drone {}   R boost {}",
            it.name,self.kit.charges[self.kit.selected],self.supplies[0],self.supplies[1],self.supplies[2],self.supplies[3],self.supplies[4]),
            32,134,16,Color::RAYWHITE);
        d.draw_text(self.text,32,h-53,18,Color::RAYWHITE);
        d.draw_text("WASD move  /  Shift sprint  /  aim & click  /  wheel equipment  /  Esc return to title",32,h-26,15,Color::LIGHTGRAY);
        if let Some(won)=self.result {
            d.draw_rectangle(w/2-285,h/2-85,570,170,Color::new(15,22,27,248));
            d.draw_text(if won {"TAX OFFICE OVERRULED"} else {"APPEAL DENIED"},w/2-242,h/2-55,32,Color::WHITE);
            d.draw_text(if won {"The Finanzamt has run out of forms."} else {"The office stays sealed. The paperwork wins."},w/2-242,h/2,18,Color::LIGHTGRAY);
            d.draw_text(if won {"Enter  /  Read the final notice"} else {"R  /  Retry the appeal       Enter  /  Title"},w/2-242,h/2+35,17,Color::SKYBLUE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn second_notice_targets_after_carter_moves() {
        let mut arena=Arena::new(Arsenal::new(),[0;5]);
        arena.boss_hp=40.0;
        arena.attack=Some(Attack::Notice {aim:Vector2::new(0.0,1.0),warning:0.01,surcharge:false});
        arena.tick(0.016,Input::default());
        assert!(arena.follow_up);
        arena.player=Vector2::new(0.72,0.8);
        arena.clock=0.0;
        arena.tick(0.016,Input::default());
        assert!(matches!(arena.attack,Some(Attack::Notice {aim,surcharge:true,..}) if aim.x>0.3));
    }

    #[test]
    fn office_inventory_is_not_consumed_and_finishing_hit_wins() {
        let office = Arsenal::new();
        let mut arena = Arena::new(office.clone(),[1,1,1,1,1]);
        arena.boss_hp=1.0;
        arena.tick(0.016,Input { aim:CENTER,fire:true,..Input::default() });
        assert_eq!(arena.result,Some(true));
        assert_eq!(office.charges[0],99);
        assert_eq!(arena.kit.charges[0],98);
    }
}
