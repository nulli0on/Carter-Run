//! CarterOS state is a simulation belonging to RUN CARTER, never the host computer.
//! Uses a strict key/value format so no extra Rust dependencies are required.
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct SystemState {
    pub night_comfort: bool,
    pub master_volume: u8,
    pub system_sounds: bool,
    pub speakers: bool,
    pub microphone: bool,
    pub wifi: bool,
    pub public_shield: bool,
    pub doxxblock: bool,
    pub diagnostics: bool,
    pub location_permission: bool,
    pub camera_permission: bool,
    pub wallpaper: u8,
    pub dark_appearance: bool,
    pub accent: u8,
    pub focus_mode: bool,
    pub display_preset: u8,
    pub interface_scale: u8,
    pub mouse_sensitivity: u8,
    pub invert_mouse: bool,
}
impl Default for SystemState {
    fn default() -> Self { Self {
        night_comfort:false, master_volume:64, system_sounds:true,
        speakers:true,microphone:true,wifi:true,public_shield:true,doxxblock:true,
        diagnostics:false,location_permission:true,camera_permission:true,
        wallpaper:0,dark_appearance:false,accent:0,focus_mode:true,
        display_preset:0,interface_scale:100,mouse_sensitivity:100,invert_mouse:false,
    }}
}
impl SystemState {
    pub fn path() -> PathBuf {crate::storage::path("carteros_settings.cfg")}
    pub fn load(path: &Path) -> Self {
        let mut state=Self::default();
        let Ok(file)=fs::read_to_string(path) else {return state};
        for line in file.lines() {
            let Some((k,v))=line.split_once('=') else {continue};
            let b = v=="1";
            match k {
                "night_comfort"=>state.night_comfort=b,
                "master_volume"=>if let Ok(n)=v.parse::<u8>(){state.master_volume=n.min(100)},
                "system_sounds"=>state.system_sounds=b,
                "speakers"=>state.speakers=b,
                "microphone"=>state.microphone=b,
                "wifi"=>state.wifi=b,
                "public_shield"=>state.public_shield=b,
                "doxxblock"=>state.doxxblock=b,
                "diagnostics"=>state.diagnostics=b,
                "location_permission"=>state.location_permission=b,
                "camera_permission"=>state.camera_permission=b,
                "wallpaper"=>if let Ok(n)=v.parse::<u8>(){state.wallpaper=n.min(2)},
                "dark_appearance"=>state.dark_appearance=b,
                "accent"=>if let Ok(n)=v.parse::<u8>(){state.accent=n.min(5)},
                "focus_mode"=>state.focus_mode=b,
                "display_preset"=>if let Ok(n)=v.parse::<u8>(){state.display_preset=n.min(1)},
                "mouse_sensitivity" => if let Ok(n) = v.parse::<u8>() { state.mouse_sensitivity = n.clamp(30, 200); },
                "invert_mouse" => state.invert_mouse = b,
                "interface_scale"=>if let Ok(n)=v.parse::<u8>(){state.interface_scale=if n==125 {125} else {100}},
                _=>{},
            }
        }
        state
    }
    pub fn save(&self,path:&Path)->std::io::Result<()> {
        let flags = [
            ("night_comfort",self.night_comfort),
            ("system_sounds",self.system_sounds),("speakers",self.speakers),
            ("microphone",self.microphone),("wifi",self.wifi),
            ("public_shield",self.public_shield),("doxxblock",self.doxxblock),
            ("diagnostics",self.diagnostics),
            ("location_permission",self.location_permission),
            ("camera_permission",self.camera_permission),
            ("dark_appearance",self.dark_appearance),("focus_mode",self.focus_mode),("invert_mouse",self.invert_mouse),
        ];
        let mut content=String::from("# RUN CARTER simulated CarterOS settings v1\n");
        for (k,v) in flags {content.push_str(&format!("{k}={}\n",u8::from(v)));}
        for (k,v) in [("master_volume",self.master_volume),
            ("wallpaper",self.wallpaper),("accent",self.accent),
            ("display_preset",self.display_preset),("interface_scale",self.interface_scale),("mouse_sensitivity",self.mouse_sensitivity)] {
            content.push_str(&format!("{k}={v}\n"));
        }
        // Atomic replacement to avoid truncating a previous save after an interrupted write.
        crate::storage::replace(path, &content)
    }
    pub fn internet_ready(&self)->bool{self.wifi}
    pub fn phishing_multiplier(&self)->f32 {
        if self.doxxblock && self.public_shield && self.wifi {0.60}
        else if self.doxxblock || self.public_shield {0.80}
        else {1.0}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn defaults_match_original_figma() {
        let x=SystemState::default();
        assert!(!x.night_comfort); assert_eq!(x.master_volume,64);
        assert!(x.wifi);assert!(x.public_shield);assert!(x.doxxblock);
        assert!(!x.diagnostics);assert_eq!(x.wallpaper,0);
    }
    #[test] fn round_trip_and_clamps() {
        let base=std::env::temp_dir().join(format!("run_carter_state_test_{}",std::process::id()));
        let path=base.join("state.cfg");
        let mut x=SystemState::default();x.night_comfort=true;x.master_volume=19;x.wallpaper=2;x.wifi=false;
        x.save(&path).expect("save");assert_eq!(SystemState::load(&path),x);
        fs::write(&path,"master_volume=999\nwallpaper=200\ninterface_scale=78\n").expect("write");
        let y=SystemState::load(&path);assert_eq!(y.master_volume,64);assert_eq!(y.wallpaper,2);assert_eq!(y.interface_scale,100);
        let _=fs::remove_dir_all(base);
    }
    #[test] fn protection_affects_fictional_phishing(){
        let mut x=SystemState::default();assert!(x.phishing_multiplier()<1.0);
        x.doxxblock=false;x.public_shield=false;assert_eq!(x.phishing_multiplier(),1.0);
    }
}
