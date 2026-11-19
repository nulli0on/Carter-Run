//! Inventory & asset manifest for non-lethal comedy-game defense items.
//! Meshes are kept unchanged; rig/animation compatibility requires a running raylib test.

pub const COUNT:usize=10;
pub const FIRST_SHOP_ID:i32=9;
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Weapon {
    pub name:&'static str,
    pub asset:&'static str,
    pub cost:i32,
    pub charges:u16,
    pub range:f32,
    pub cooldown:f32,
    pub burst:usize,
}

pub const CATALOG:[Weapon;COUNT]=[
    Weapon{name:"Cardboard Blaster",asset:"cardboard_blaster.glb",cost:35,charges:99,range:2.1,cooldown:1.1,burst:1},
    Weapon{name:"Foam Knife",asset:"knife.glb",cost:90,charges:99,range:1.2,cooldown:0.65,burst:1},
    Weapon{name:"Mark 23 (prop)",asset:"mark23.glb",cost:180,charges:32,range:6.0,cooldown:0.45,burst:1},
    Weapon{name:"Vector (prop)",asset:"vector.glb",cost:360,charges:48,range:5.0,cooldown:0.23,burst:1},
    Weapon{name:"M4 (prop)",asset:"m4.glb",cost:470,charges:48,range:7.5,cooldown:0.28,burst:1},
    Weapon{name:"Minigun (prop)",asset:"minigun.glb",cost:1300,charges:120,range:6.0,cooldown:0.09,burst:2},
    Weapon{name:"Confetti Flamethrower",asset:"flamethrower.glb",cost:980,charges:80,range:3.0,cooldown:0.14,burst:3},
    Weapon{name:"T-Shirt RPG (prop)",asset:"rpg7.glb",cost:1100,charges:8,range:9.0,cooldown:2.3,burst:6},
    Weapon{name:"Party Grenade (prop)",asset:"grenade.glb",cost:170,charges:6,range:4.0,cooldown:1.6,burst:4},
    Weapon{name:"Dynamite Pinata (prop)",asset:"dynamite.glb",cost:220,charges:4,range:3.6,cooldown:2.6,burst:5},
];

#[derive(Debug,Clone,PartialEq)]
pub struct Arsenal{
    pub owned:[bool;COUNT],
    pub charges:[u16;COUNT],
    pub selected:usize,
    pub cooldown:f32,
}
impl Default for Arsenal {fn default()->Self{Self::new()}}
impl Arsenal {
    pub fn new()->Self{
        let mut owned=[false;COUNT];owned[0]=true;
        let mut charges=[0;COUNT];charges[0]=99;
        Self{owned,charges,selected:0,cooldown:0.0}
    }
    pub fn buy(&mut self,index:usize,cash:&mut i32)->Result<&'static str,&'static str>{
        let Some(w)=CATALOG.get(index) else{return Err("Unknown AmazonCrime item")};
        let is_refill=self.owned[index];
        let price=if is_refill{(w.cost/4).max(20)}else{w.cost};
        if is_refill && self.charges[index]>=w.charges {return Err("Already fully stocked")}
        if *cash<price{return Err("Insufficient cash for this item")}
        *cash-=price;
        self.owned[index]=true;
        self.charges[index]=self.charges[index].saturating_add(w.charges).min(w.charges.saturating_mul(3));
        self.selected=index;
        Ok(if is_refill{"Refill purchased. Select with number keys 1-0."}else{"Model added to the fictional armory. Select with keys 1-0."})
    }
    pub fn equip(&mut self,index:usize)->bool{
        if index>=COUNT || !self.owned[index]{return false;}
        self.selected=index;true
    }
    pub fn tick(&mut self,dt:f32){self.cooldown=(self.cooldown-dt.max(0.0)).max(0.0);}
    pub fn fire(&mut self)->Option<Weapon>{
        let idx=self.selected;
        if idx>=COUNT || !self.owned[idx] || self.cooldown>0.0 || self.charges[idx]==0{return None}
        let weapon=CATALOG[idx];self.charges[idx]-=1;self.cooldown=weapon.cooldown;
        Some(weapon)
    }
    pub fn all_owned(&self)->bool{self.owned.iter().all(|v|*v)}
    pub fn ownership_mask(&self)->u16{
        self.owned.iter().enumerate().fold(0u16,|bits,(i,have)|if *have{bits | 1u16<<i}else{bits})
    }
    pub fn restore(mask:u16,ammo:[u16;COUNT],selected:usize)->Option<Self>{
        if mask & !((1u16<<COUNT)-1)!=0 || mask&1==0 || selected>=COUNT || mask&(1<<selected)==0{return None}
        let owned=std::array::from_fn(|i|mask&(1u16<<i)!=0);
        if ammo.iter().enumerate().any(|(i,c)|(*c>CATALOG[i].charges.saturating_mul(3))||(!owned[i]&&*c!=0)){return None}
        Some(Self{owned,charges:ammo,selected,cooldown:0.0})
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn starter_and_purchase(){let mut a=Arsenal::new();assert_eq!(a.ownership_mask(),1);let mut cash=300;
        assert!(a.buy(2,&mut cash).is_ok());assert_eq!(cash,120);assert!(a.equip(2));}
    #[test]fn cannot_spam_without_cooldown_or_ammo(){let mut a=Arsenal::new();
        assert!(a.fire().is_some());assert!(a.fire().is_none());a.tick(2.0);assert!(a.fire().is_some());}
    #[test]fn reserve_and_restore(){let mut a=Arsenal::new();let mut money=900;
        a.buy(2,&mut money).unwrap();let b=Arsenal::restore(a.ownership_mask(),a.charges,a.selected).unwrap();assert_eq!(a,b);}
    #[test]fn no_cheat_from_corrupted_save(){assert!(Arsenal::restore(0,[0;COUNT],0).is_none());
        assert!(Arsenal::restore(1,[65535;COUNT],0).is_none());}
    #[test]fn purchase_requires_budget(){let mut a=Arsenal::new();let mut cash=0;
        assert!(a.buy(9,&mut cash).is_err());assert_eq!(cash,0);}
}
