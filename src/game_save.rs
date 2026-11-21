//! Versioned, bounded save for RUN CARTER's *fictional* campaign state.
//! File locations are selected by the game at startup.
use std::{fs, io::{self, Read}, path::{Path, PathBuf}};

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
    pub sugar_boost:f32,
    pub inbox_seed: u64,
    pub inbox_text: Vec<String>,
    pub inspected: bool,
    pub business: crate::business::Business,
    pub side: crate::creator::Campaign,
    pub alarm: f32,
    pub shift_closing: bool,
    pub defended_today: bool,
    pub studio_day: u32,
    pub weapon_cooldown: f32,
    pub decoy: Option<(f32, f32, f32)>,
    pub run_progress: Option<crate::achievements::RunProgress>,
    pub bailout_debt:i32,pub bailout_last_day:u32,
    pub weapon_owned:u16,pub weapon_charges:[u16;crate::arsenal::COUNT],pub weapon_selected:u8,
}

impl Snapshot {
    pub fn path() -> PathBuf { crate::storage::path("game_v1.cfg") }
    pub fn validate(&self) -> bool {
        self.x.is_finite() && (-1.31..=1.09).contains(&self.x)
            && self.z.is_finite() && (-2.56..=2.57).contains(&self.z)
            && self.yaw.is_finite() && self.yaw.abs()<=1000.0
            && self.pitch.is_finite() && self.pitch.abs()<=1.4
            && (0..=5_000_000).contains(&self.money)
            && self.exposure.is_finite() && (0.0..=100.0).contains(&self.exposure)
            && self.day > 0 && self.inbox_index <= 13
            && self.scans<=crate::INVENTORY_CAP && self.raspberry_pis<=crate::INVENTORY_CAP
            && self.phone_cooldown.is_finite() && (0.0..=45.0).contains(&self.phone_cooldown)
            && self.airhorns<=crate::INVENTORY_CAP && self.confetti_cannons<=crate::INVENTORY_CAP && self.drones<=crate::INVENTORY_CAP
            && self.sugar_hits<=crate::INVENTORY_CAP
            && self.sugar_timer.is_finite() && (-1.0..=75.0).contains(&self.sugar_timer)
            && self.sugar_boost.is_finite() && (0.0..=25.0).contains(&self.sugar_boost)
            && (0..=100_000).contains(&self.bailout_debt) && self.bailout_last_day<=self.day
            && self.studio_day <= self.day && self.side.fan_day <= self.day
            && (-1.0..=20.0).contains(&self.alarm)
            && (!self.shift_closing || self.alarm >= 0.0)
            && self.inbox_text.len() <= 10 && self.inbox_text.iter().all(|entry|crate::inbox::valid_entry(entry))
            && (0.0..=3.0).contains(&self.weapon_cooldown)
            && self.decoy.is_none_or(|(x, z, remaining)| {
                (-1.31..=1.09).contains(&x) && (-2.56..=2.57).contains(&z) &&
                (0.0..=24.0).contains(&remaining)
            })
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
        out.push_str(&format!("airhorns={}\nconfetti={}\ndrones={}\nsugar_hits={}\nsugar_timer={}\nsugar_visible={}\nsugar_boost={}\n",
            self.airhorns,self.confetti_cannons,self.drones,self.sugar_hits,self.sugar_timer,
            self.sugar_visible as u8,self.sugar_boost));
        out.push_str(&format!("inbox_seed={}\ninbox_text={}\ninspected={}\nbusiness={}\ndefended_today={}\nstudio_day={}\nweapon_cooldown={}\n",
            self.inbox_seed, self.inbox_text.join("|"), u8::from(self.inspected), self.business.encode(),
            u8::from(self.defended_today), self.studio_day, self.weapon_cooldown));
        out.push_str(&format!("side={}\nalarm={}\nshift_closing={}\n", self.side.encode(), self.alarm,u8::from(self.shift_closing)));
        if let Some(run) = &self.run_progress {
            out.push_str(&format!("run_progress={}\n", run.encode()));
        }
        if let Some((x, z, remaining)) = self.decoy {
            out.push_str(&format!("decoy={x},{z},{remaining}\n"));
        }
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
            inbox_seed: value("inbox_seed").unwrap_or("13257610").parse().ok()?,
            inbox_text: value("inbox_text").filter(|text| !text.is_empty())
                .map(|text| text.split('|').map(str::to_owned).collect()).unwrap_or_default(),
            inspected: value("inspected") == Some("1"),
            business: match value("business") {
                Some(record) => crate::business::Business::decode(record)?,
                None => crate::business::Business::default(),
            },
            side: value("side").map_or_else(||Some(crate::creator::Campaign::default()),crate::creator::Campaign::decode)?,
            alarm: value("alarm").unwrap_or("-1").parse().ok()?,
            shift_closing: value("shift_closing") == Some("1"),
            defended_today: value("defended_today") == Some("1"),
            studio_day: value("studio_day").unwrap_or("0").parse().ok()?,
            weapon_cooldown: value("weapon_cooldown").unwrap_or("0").parse().ok()?,
            run_progress: match value("run_progress") {
                Some(text) => Some(crate::achievements::RunProgress::decode(text)?),
                None => None,
            },
            decoy: match value("decoy") {
                Some(data) => {
                    let parts = data.split(',').map(str::parse::<f32>).collect::<Result<Vec<_>, _>>().ok()?;
                    let &[x, z, time] = parts.as_slice() else { return None; };
                    Some((x, z, time))
                }
                None => None,
            },
            bailout_debt:value("bailout_debt").unwrap_or("0").parse().ok()?,
            bailout_last_day:value("bailout_last_day").unwrap_or("0").parse().ok()?,
            weapon_owned:value("weapon_owned").unwrap_or("1").parse().ok()?,
            weapon_selected:value("weapon_selected").unwrap_or("0").parse().ok()?,
            weapon_charges:match value("weapon_charges") {
                Some(list)=>{
                    let parsed:Vec<u16>=list.split(',').map(str::parse).collect::<Result<_,_>>().ok()?;
                    match parsed.len(){
                        crate::arsenal::COUNT => parsed.try_into().ok()?,
                        // v0.14 legacy saves have only ten inventory charge slots.
                        10 => {
                            let mut expanded=[0u16;crate::arsenal::COUNT];
                            expanded[..10].copy_from_slice(&parsed);
                            expanded
                        },
                        _ => return None,
                    }
                },
                None=>{let mut c=[0u16;crate::arsenal::COUNT];c[0]=99;c}
            },
        };
        let mut snap = snap;
        if value("inbox_seed").is_none() {
            snap.inbox_index = snap.inbox_index.min(crate::inbox::count(snap.day));
        }
        snap.validate().then_some(snap)
    }
    fn read_valid(path: &Path) -> Option<(Self, String)> {
        let mut text = String::new();
        fs::File::open(path).ok()?.take(MAX_SAVE_BYTES + 1).read_to_string(&mut text).ok()?;
        if text.len() as u64 > MAX_SAVE_BYTES {
            return None;
        }
        let snapshot = Self::decode(&text)?;
        match crate::integrity::verify_text(&text, path) {
            crate::integrity::Result::Unchanged | crate::integrity::Result::Untracked => Some((snapshot, text)),
            _ => None,
        }
    }

    pub fn load(path: &Path) -> Option<(Self, bool)> {
        Self::read_valid(path).map(|(snapshot, _)| (snapshot, false)).or_else(|| {
            Self::read_valid(&path.with_extension("bak")).map(|(snapshot, _)| (snapshot, true))
        })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if !self.validate() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid campaign snapshot"));
        }
        let next = crate::integrity::protect(&self.encode());
        if let Some((previous, raw)) = Self::read_valid(path) {
            if raw == next {
                return Ok(());
            }
            let backup = crate::integrity::protect(&previous.encode());
            crate::storage::replace(&path.with_extension("bak"), &backup)?;
        }
        crate::storage::replace(path, &next)
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
        sugar_visible:false,sugar_boost:0.0,
        inbox_seed: 42, inbox_text: vec!["An example line".to_owned()], inspected: true,
        business: crate::business::Business::default(), side:crate::creator::Campaign::default(), alarm:-1.0, shift_closing:false, defended_today: true, studio_day: 2,
        weapon_cooldown: 0.2, decoy: Some((0.0, 1.0, 2.0)),
        run_progress: Some(crate::achievements::RunProgress::default()),
        bailout_debt:0,bailout_last_day:0,
        weapon_owned:1,weapon_charges:{let mut c=[0;crate::arsenal::COUNT];c[0]=99;c},weapon_selected:0} }
    #[test] fn round_trip_and_atomic_file() {
        let data=sample();let path=std::env::temp_dir().join(format!("run_carter_snapshot_{}.cfg",std::process::id()));
        data.save(&path).unwrap();assert_eq!(Some((data, false)),Snapshot::load(&path));let _=fs::remove_file(path);
    }
    #[test]
    fn damaged_primary_recovers_last_good_checkpoint() {
        let path = std::env::temp_dir().join(format!("run_carter_recovery_{}.cfg", std::process::id()));
        let backup = path.with_extension("bak");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&backup);
        let previous = sample();
        previous.save(&path).unwrap();
        let mut current = previous.clone();
        current.money += 10;
        current.save(&path).unwrap();
        current.save(&path).unwrap();
        assert_eq!(Snapshot::load(&path), Some((current.clone(), false)));

        let modified = fs::read_to_string(&path).unwrap().replace("money=240", "money=241");
        fs::write(&path, modified).unwrap();
        assert_eq!(Snapshot::load(&path), Some((previous.clone(), true)));
        previous.save(&path).unwrap();
        assert_eq!(Snapshot::load(&path), Some((previous.clone(), false)));
        fs::remove_file(&path).unwrap();
        assert_eq!(Snapshot::load(&path), Some((previous, true)));
        fs::remove_file(backup).unwrap();
        assert!(Snapshot::load(&path).is_none());
    }

    #[test] fn legacy_ten_weapon_save_is_upgraded_without_losing_cash(){
        let mut old=sample();old.money=724;old.weapon_owned=0b10001;
        old.weapon_charges[4]=12;old.weapon_selected=4;
        let legacy=old.encode().replace(
            &format!("weapon_charges={}",old.weapon_charges.iter().map(u16::to_string).collect::<Vec<_>>().join(",")),
            &format!("weapon_charges={}",old.weapon_charges[..10].iter().map(u16::to_string).collect::<Vec<_>>().join(","))
        );
        let got=Snapshot::decode(&legacy).unwrap();
        assert_eq!(got.money,724);assert_eq!(got.weapon_selected,4);
        assert_eq!(got.weapon_charges[4],12);
        assert!(got.weapon_charges[10..].iter().all(|v|*v==0));
        assert_eq!(got.weapon_owned,0b10001);
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
