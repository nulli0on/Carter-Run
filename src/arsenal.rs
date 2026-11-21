//! Inventory & asset manifest for non-lethal comedy-game defense items.
//! Meshes are kept unchanged; rig/animation compatibility requires a running raylib test.

pub const COUNT:usize=16;
pub const FIRST_SHOP_ID:i32=9;
pub const GADGETS: [(i32, &str, i32); 10] = [
    (1, "Raspberry Pi decoy", 85),
    (2, "AntiDoxx patch", 600),
    (3, "3 email scans", 75),
    (4, "Reinforced door (+7s warning)", 1_550),
    (5, "Airhorn [Z]", 28),
    (6, "Confetti cannon [X]", 175),
    (7, "Decoy drone [C]", 475),
    (8, "Colombian Powder [R]", 240),
    (25, "Delivery shelf (+8 slots)", 425),
    (26, "Studio encoder (+25% upload pay)", 1_200),
];

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
    Weapon{name:"Cardboard Blaster",asset:"cardboard_blaster_rigged.glb",cost:35,charges:99,range:2.1,cooldown:1.1,burst:1},
    Weapon{name:"Foam Knife",asset:"knife.glb",cost:90,charges:99,range:1.2,cooldown:0.65,burst:1},
    Weapon{name:"Mark 23 (prop)",asset:"mark23.glb",cost:180,charges:32,range:6.0,cooldown:0.45,burst:1},
    Weapon{name:"Vector (prop)",asset:"vector.glb",cost:360,charges:48,range:5.0,cooldown:0.23,burst:1},
    Weapon{name:"M4 (prop)",asset:"m4.glb",cost:470,charges:48,range:7.5,cooldown:0.28,burst:1},
    Weapon{name:"Minigun (prop)",asset:"minigun.glb",cost:1300,charges:120,range:6.0,cooldown:0.09,burst:2},
    Weapon{name:"Confetti Flamethrower",asset:"flamethrower.glb",cost:980,charges:80,range:3.0,cooldown:0.14,burst:3},
    Weapon{name:"T-Shirt RPG (prop)",asset:"rpg7.glb",cost:1100,charges:8,range:9.0,cooldown:2.3,burst:6},
    Weapon{name:"Party Grenade (prop)",asset:"grenade.glb",cost:170,charges:6,range:4.0,cooldown:1.6,burst:4},
    Weapon{name:"Dynamite Pinata (prop)",asset:"dynamite.glb",cost:220,charges:4,range:3.6,cooldown:2.6,burst:5},
    Weapon{name:"Foam Shotgun",asset:"shotgun.glb",cost:520,charges:28,range:5.5,cooldown:0.95,burst:4},
    Weapon{name:"T-Shirt Sniper",asset:"sniper.glb",cost:840,charges:15,range:12.0,cooldown:1.6,burst:1},
    Weapon{name:"Persuader Megaphone",asset:"persuader.glb",cost:650,charges:32,range:5.5,cooldown:0.60,burst:3},
    Weapon{name:"Sticker LMG",asset:"lmg.glb",cost:1180,charges:140,range:7.5,cooldown:0.12,burst:2},
    Weapon{name:"Confetti Launcher",asset:"grenadelauncher.glb",cost:750,charges:18,range:8.0,cooldown:1.80,burst:5},
    Weapon{name:"Pistol (prop)",asset:"pistol.glb",cost:250,charges:36,range:6.0,cooldown:0.40,burst:1},
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
    pub fn price(&self, index: usize) -> i32 {
        let price = CATALOG[index].cost;
        if self.owned[index] { (price / 4).max(20) } else { price }
    }

    pub fn buy(&mut self,index:usize,cash:&mut i32)->Result<&'static str,&'static str>{
        let Some(w)=CATALOG.get(index) else{return Err("Unknown AmazonCrime item")};
        let is_refill=self.owned[index];
        let price=self.price(index);
        if is_refill && self.charges[index]>=w.charges {return Err("Already fully stocked")}
        if *cash<price{return Err("Insufficient cash for this item")}
        *cash-=price;
        self.owned[index]=true;
        self.charges[index]=self.charges[index].saturating_add(w.charges).min(w.charges.saturating_mul(3));
        self.selected=index;
        Ok(if is_refill{"Refill purchased and equipped. Scroll to switch items."}else{"Added to your armory and equipped. Scroll to switch items."})
    }
    pub fn equip(&mut self,index:usize)->bool{
        if index>=COUNT || !self.owned[index]{return false;}
        self.selected=index;true
    }
    /// Cycle through owned items, including slots 11–16 unavailable on the number keys.
    /// Returns false without changing selection when the starter is the only owned item.
    pub fn cycle(&mut self,forward:bool)->bool{
        let start=self.selected;
        for step in 1..COUNT {
            let index=if forward{(start+step)%COUNT}else{(start+COUNT-step)%COUNT};
            if self.owned[index]{self.selected=index;return true;}
        }
        false
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
        if mask & !(((1u32<<COUNT)-1) as u16)!=0 || mask&1==0 || selected>=COUNT || mask&(1<<selected)==0{return None}
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
    #[test]fn six_new_models_have_stable_unique_ids(){
        assert_eq!(COUNT,16);
        for w in CATALOG.iter(){assert!(w.cost>=0);assert!(!w.asset.is_empty());}
        let unique=CATALOG.iter().map(|w|w.asset).collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(),COUNT);
        assert_eq!(CATALOG[8].asset,"grenade.glb");
        assert_eq!(CATALOG[9].asset,"dynamite.glb");
        assert_eq!(CATALOG[15].asset,"pistol.glb");
    }
    #[test]fn cycling_skips_unowned_and_wraps(){
        let mut a=Arsenal::new();assert!(!a.cycle(true));
        let mut cash=4000;a.buy(15,&mut cash).unwrap();
        assert_eq!(a.selected,15);
        assert!(a.cycle(true));assert_eq!(a.selected,0);
        assert!(a.cycle(false));assert_eq!(a.selected,15);
    }
    #[test]fn high_weapon_bit_roundtrips(){
        let mut a=Arsenal::new();let mut cash=1000;
        a.buy(15,&mut cash).unwrap();
        assert_eq!(a.ownership_mask(),(1<<15)|1);
        let b=Arsenal::restore(a.ownership_mask(),a.charges,a.selected).unwrap();
        assert_eq!(a,b);
    }
    #[test]fn purchase_requires_budget(){let mut a=Arsenal::new();let mut cash=0;
        assert!(a.buy(9,&mut cash).is_err());assert_eq!(cash,0);}
}
