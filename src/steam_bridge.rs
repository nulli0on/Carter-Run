//! Optional real Steam achievement integration.
//! Without `--features steam`, the game keeps a clearly labelled local pending unlock.
//! For an actual Steam unlock, configure and publish achievement SHAME_SHAME_SHAME
//! for the game's own App ID in Steamworks Partner, then run the Steam build through Steam.
#[cfg(all(feature="steam",not(any(target_os="linux",target_os="windows"))))]
compile_error!("Steam bridge currently supports Linux and Windows with the supplied SDK 1.65 redistributables.");

#[cfg(feature="steam")]
mod native {
    use std::ffi::{c_char,c_void,CString};
    #[cfg_attr(target_os="windows",link(name="steam_api64"))]
    #[cfg_attr(not(target_os="windows"),link(name="steam_api"))]
    extern "C" {
        fn SteamAPI_InitFlat(out_error:*mut c_char)->i32;
        fn SteamAPI_RunCallbacks();
        fn SteamAPI_Shutdown();
        fn SteamAPI_SteamUserStats_v013()->*mut c_void;
        fn SteamAPI_ISteamUserStats_SetAchievement(self_ptr:*mut c_void,name:*const c_char)->bool;
        fn SteamAPI_ISteamUserStats_StoreStats(self_ptr:*mut c_void)->bool;
    }
    pub struct Bridge {ready:bool}
    impl Bridge {
        pub fn new()->Self {
            let ready=unsafe{SteamAPI_InitFlat(std::ptr::null_mut())==0};
            if !ready{eprintln!("Steamworks not initialized: achievement will remain locally pending.");}
            Self{ready}
        }
        pub fn tick(&self){if self.ready{unsafe{SteamAPI_RunCallbacks()};}}
        pub fn unlock(&self,name:&str)->bool {
            if !self.ready{return false;}
            let Ok(name)=CString::new(name)else{return false;};
            unsafe {
                let stats=SteamAPI_SteamUserStats_v013();
                !stats.is_null() && SteamAPI_ISteamUserStats_SetAchievement(stats,name.as_ptr()) && SteamAPI_ISteamUserStats_StoreStats(stats)
            }
        }
    }
    impl Drop for Bridge{fn drop(&mut self){if self.ready{unsafe{SteamAPI_Shutdown()};}}}
}
#[cfg(not(feature="steam"))]
mod native {
    pub struct Bridge;
    impl Bridge {
        pub fn new()->Self{Self}
        pub fn tick(&self){}
        pub fn unlock(&self,_name:&str)->bool{false}
    }
}
pub use native::Bridge;
