//! Versioned, bounded save for RUN CARTER's *fictional* campaign state.
//! No external file paths are taken from game content. Saved in .run_carter only.
use std::{fs, path::{Path, PathBuf}};

const SIGNATURE: &str = "RUN_CARTER_SAVE_V1";
const MAX_SAVE_BYTES: u64 = 8192;
const MAX_ORDERS: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub struct SavedOrder {
    pub item: u8,
    pub eta: f32,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub x: f32, pub z: f32, pub yaw: f32, pub pitch: f32,
    pub money: i32, pub exposure: f32, pub day: u32,
    pub inbox_index: usize, pub scans: u32, pub raspberry_pis: u32,
    pub door_upgrade: bool, pub phone_cooldown: f32, pub omni_last_day: u32,
    pub orders: Vec<SavedOrder>,
    pub airhorns:u32,pub confetti_cannons:u32,pub drones:u32,
    pub sugar_hits:u32,pub sugar_timer:f32,pub sugar_visible:bool,
    pub sugar_boost:f32,pub campaign_won:bool,
    pub bailout_debt:i32,pub bailout_last_day:u32,
    pub weapon_owned:u16,pub weapon_charges:[u16;10],pub weapon_selected:u8,
}

impl Snapshot {
    pub fn path() -> PathBuf { PathBuf::from(".run_carter/game_v1.cfg") }
    pub fn validate(&self) -> bool {
        self.x.is_finite() && (-1.31..=1.09).contains(&self.x)
            && self.z.is_finite() && (-2.56..=2.57).contains(&self.z)
            && self.yaw.is_finite() && self.yaw.abs()<=1000.0
            && self.pitch.is_finite() && self.pitch.abs()<=1.4
            && (0..=5_000_000).contains(&self.money)
            && self.exposure.is_finite() && (0.0..=100.0).contains(&self.exposure)
            && (1..=9999).contains(&self.day) && self.inbox_index<=10
            && self.scans<=100_000 && self.raspberry_pis<=100_000
            && self.phone_cooldown.is_finite() && (0.0..=45.0).contains(&self.phone_cooldown)
            && self.airhorns<=100_000 && self.confetti_cannons<=100_000 && self.drones<=100_000
            && self.sugar_hits<=100_000
            && self.sugar_timer.is_finite() && (-1.0..=75.0).contains(&self.sugar_timer)
            && self.sugar_boost.is_finite() && (0.0..=25.0).contains(&self.sugar_boost)
            && (0..=100_000).contains(&self.bailout_debt) && self.bailout_last_day<=self.day
            && self.omni_last_day<=self.day && self.orders.len()<=MAX_ORDERS
            && crate::arsenal::Arsenal::restore(self.weapon_owned,self.weapon_charges,self.weapon_selected as usize).is_some()
            && self.orders.iter().all(|o| (o.item==1||o.item==3)
                && o.eta.is_finite() && (0.0..=120.0).contains(&o.eta))
    }
    fn encode(&self) -> String {
        let mut out=format!("{SIGNATURE}\nx={}\nz={}\nyaw={}\npitch={}\nmoney={}\nexposure={}\nday={}\ninbox_index={}\nscans={}\nraspberry_pis={}\ndoor_upgrade={}\nphone_cooldown={}\nomni_last_day={}\n",
            self.x,self.z,self.yaw,self.pitch,self.money,self.exposure,self.day,
            self.inbox_index,self.scans,self.raspberry_pis,u8::from(self.door_upgrade),
            self.phone_cooldown,self.omni_last_day);
        out.push_str(&format!("airhorns={}\nconfetti={}\ndrones={}\nsugar_hits={}\nsugar_timer={}\nsugar_visible={}\nsugar_boost={}\ncampaign_won={}\n",
            self.airhorns,self.confetti_cannons,self.drones,self.sugar_hits,self.sugar_timer,
            self.sugar_visible as u8,self.sugar_boost,self.campaign_won as u8));
        out.push_str(&format!("bailout_debt={}\nbailout_last_day={}\n",self.bailout_debt,self.bailout_last_day));
        out.push_str(&format!("weapon_owned={}\nweapon_selected={}\nweapon_charges={}\n",
            self.weapon_owned,self.weapon_selected,self.weapon_charges.iter().map(u16::to_string).collect::<Vec<_>>().join(",")));
        for o in &self.orders {
            out.push_str(&format!("order={},{},{}\n",o.item,o.eta,u8::from(o.ready)));
        }
        out
    }
    fn decode(text: &str) -> Option<Self> {
        let mut lines=text.lines();
        if lines.next()? != SIGNATURE {return None;}
        let mut values=std::collections::HashMap::new();
        let mut orders=Vec::new();
        for line in lines {
            let (key,value)=line.split_once('=')?;
            if key=="order" {
                if orders.len()>=MAX_ORDERS{return None;}
                let parts:Vec<_>=value.split(',').collect();
                if parts.len()!=3{return None;}
                orders.push(SavedOrder{
                    item:parts[0].parse().ok()?,
                    eta:parts[1].parse().ok()?,
                    ready:match parts[2]{"0"=>false,"1"=>true,_=>return None},
                });
            } else if values.insert(key,value).is_some() {return None;}
        }
        let value=|key| values.get(key).copied();
        let snap=Self {
            x:value("x")?.parse().ok()?, z:value("z")?.parse().ok()?,
            yaw:value("yaw")?.parse().ok()?,pitch:value("pitch")?.parse().ok()?,
            money:value("money")?.parse().ok()?, exposure:value("exposure")?.parse().ok()?,
            day:value("day")?.parse().ok()?, inbox_index:value("inbox_index")?.parse().ok()?,
            scans:value("scans")?.parse().ok()?,raspberry_pis:value("raspberry_pis")?.parse().ok()?,
            door_upgrade:match value("door_upgrade")?{"0"=>false,"1"=>true,_=>return None},
            phone_cooldown:value("phone_cooldown")?.parse().ok()?,
            omni_last_day:value("omni_last_day")?.parse().ok()?,orders,
            airhorns:value("airhorns").unwrap_or("0").parse().ok()?,
            confetti_cannons:value("confetti").unwrap_or("0").parse().ok()?,
            drones:value("drones").unwrap_or("0").parse().ok()?,
            sugar_hits:value("sugar_hits").unwrap_or("0").parse().ok()?,
            sugar_timer:value("sugar_timer").unwrap_or("-1").parse().ok()?,
            sugar_visible:match value("sugar_visible").unwrap_or("0"){"0"=>false,"1"=>true,_=>return None},
            sugar_boost:value("sugar_boost").unwrap_or("0").parse().ok()?,
            campaign_won:match value("campaign_won").unwrap_or("0"){"0"=>false,"1"=>true,_=>return None},
            bailout_debt:value("bailout_debt").unwrap_or("0").parse().ok()?,
            bailout_last_day:value("bailout_last_day").unwrap_or("0").parse().ok()?,
            weapon_owned:value("weapon_owned").unwrap_or("1").parse().ok()?,
            weapon_selected:value("weapon_selected").unwrap_or("0").parse().ok()?,
            weapon_charges:match value("weapon_charges") {
                Some(list)=>{
                    let parsed:Vec<u16>=list.split(',').map(str::parse).collect::<Result<_,_>>().ok()?;
                    parsed.try_into().ok()?
                },
                None=>{let mut c=[0u16;10];c[0]=99;c}
            },
        };
        if snap.validate() {Some(snap)}else{None}
    }
    pub fn load(path:&Path) -> Option<Self> {
        if fs::metadata(path).ok()?.len()>MAX_SAVE_BYTES{return None;}
        Self::decode(&fs::read_to_string(path).ok()?)
    }
    pub fn save(&self,path:&Path) -> std::io::Result<()> {
        if !self.validate(){return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"Invalid campaign snapshot"));}
        if let Some(parent)=path.parent(){fs::create_dir_all(parent)?;}
        let tmp=path.with_extension("tmp");
        fs::write(&tmp,self.encode())?;
        fs::rename(&tmp,path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample()->Snapshot {Snapshot{x:0.0,z:-0.8,yaw:1.0,pitch:0.0,money:230,
        exposure:25.0,day:2,inbox_index:3,scans:4,raspberry_pis:2,
        door_upgrade:false,phone_cooldown:10.0,omni_last_day:1,
        orders:vec![SavedOrder{item:1,eta:12.5,ready:false}],
        airhorns:2,confetti_cannons:1,drones:0,sugar_hits:0,sugar_timer:-1.0,
        sugar_visible:false,sugar_boost:0.0,campaign_won:false,
        bailout_debt:0,bailout_last_day:0,
        weapon_owned:1,weapon_charges:[99,0,0,0,0,0,0,0,0,0],weapon_selected:0} }
    #[test] fn round_trip_and_atomic_file() {
        let data=sample();let path=std::env::temp_dir().join(format!("run_carter_snapshot_{}.cfg",std::process::id()));
        data.save(&path).unwrap();assert_eq!(Some(data),Snapshot::load(&path));let _=fs::remove_file(path);
    }
    #[test] fn reject_tampered_values_and_unknown_orders() {
        let data=sample();let text=data.encode();
        assert!(Snapshot::decode(&text.replace("money=230","money=-1000")).is_none());
        assert!(Snapshot::decode(&text.replace("12.5,0","NaN,0")).is_none());
        assert!(Snapshot::decode(&text.replace("order=1","order=7")).is_none());
        assert!(Snapshot::decode(&text.replace(SIGNATURE,"ANOTHER_GAME")).is_none());
    }
    #[test] fn save_is_only_allowed_for_valid_state() {
        let mut data=sample();data.x=30.0;assert!(!data.validate());
        data=sample();data.orders=vec![SavedOrder{item:1,eta:5.0,ready:false};13];
        assert!(!data.validate());
    }
}
