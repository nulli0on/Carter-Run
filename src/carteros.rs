//! CarterOS visual integration — original user-provided Figma SVG artwork only.
//! The exported RGBA layers are exact 1:1 1440x900 artboard elements, not redrawn UI.
//! Newly required app bodies are overlaid inside the Figma window chrome;
//! source-designed artboards remain unchanged on disk.
//! RUN CARTER game behavior remains in main.rs.
use raylib::prelude::*;
use std::collections::HashMap;
use crate::system_state::SystemState;
use crate::drive::{Drive,Kind};
use crate::cheats::{self, CheatKind};
use crate::web_search::{SearchEngine,AnswerKind};
use std::fs;

pub const W: i32 = 1440;
pub const H: i32 = 900;

// These x/y coordinates are the alpha-crop origins on the original Figma frame.
// They were measured from the SVG and MUST NOT be normalized independently.
const LAYERS: &[(&str,i32,i32)] = &[
    ("desktop_bg",0,0), ("menu_bar",0,0), ("dock",257,786),
    ("desktop_widgets",22,57), ("launchpad",0,0),
    ("notification_center",990,28), ("context_menu",511,321),
    ("files_window",109,89),
    ("settings_display",192,73), ("settings_sound",192,73),
    ("settings_network",192,73), ("settings_privacy",192,73),
    ("settings_personalization",192,73),
    ("carterweb_inactive",89,84), ("cartermail_window",480,232),
    ("lock_screen",0,0), ("boot_screen",0,0),
    ("leak_bg",0,0), ("leak_banner",24,58),
    ("leak_modal",378,198), ("leak_radar",1023,656),
    ("toggle_on",0,0),("toggle_off",0,0),
    ("wallpaper_after_hours",0,0),
    // The complete source artboards are drawn when a screen is in its exact
    // default state; independent layers take over only for movable windows.
    ("full_frame_01",0,0),("full_frame_02",0,0),("full_frame_04",0,0),
    ("full_frame_05",0,0),("full_frame_06",0,0),("full_frame_07",0,0),
    ("full_frame_08",0,0),("full_frame_09",0,0),("full_frame_10",0,0),
    ("full_frame_11",0,0),("full_frame_12",0,0),("full_frame_14",0,0),
];
struct Layer { tex: Texture2D, x:i32, y:i32 }
pub struct Assets {layers: HashMap<&'static str,Layer>}
impl Assets {
    pub fn load(rl:&mut RaylibHandle, thread:&RaylibThread)->Self {
        let mut layers=HashMap::new();
        for &(name,x,y) in LAYERS {
            let filename=format!("assets/carteros/{name}.png");
            match rl.load_texture(thread,&filename) {
                Ok(tex)=>{layers.insert(name,Layer{tex,x,y});},
                Err(e)=>eprintln!("CarterOS Figma layer missing: {filename} ({e:?})"),
            }
        }
        Self {layers}
    }
    fn draw<D: RaylibDraw>(&self,d:&mut D,name:&str,dx:i32,dy:i32){
        if let Some(layer)=self.layers.get(name){
            d.draw_texture_pro(&layer.tex,
                Rectangle::new(0.,0.,layer.tex.width as f32,layer.tex.height as f32),
                Rectangle::new((layer.x+dx) as f32,(layer.y+dy) as f32,layer.tex.width as f32,layer.tex.height as f32),
                Vector2::zero(),0.0,Color::WHITE);
        }
    }
    fn paste<D: RaylibDraw>(&self,d:&mut D,name:&str,x:i32,y:i32) {
        if let Some(layer)=self.layers.get(name){d.draw_texture(&layer.tex,x,y,Color::WHITE);}
    }
}

#[derive(Default)]
pub struct Input {
    pub x:i32,pub y:i32,pub click:bool,pub right_click:bool,pub down:bool,
    pub wheel:f32,pub chars:String,pub backspace:bool,pub enter:bool,
    pub escape:bool,pub lock:bool,pub up:bool,pub down_key:bool,pub left:bool,pub right:bool,
    pub save:bool,pub find:bool,pub new_file:bool,pub new_folder:bool,pub rename:bool,
    pub toggle_trash:bool,pub restore:bool,pub cycle_window:bool,
    pub minimize_window:bool,pub close_window:bool,pub maximize_window:bool,
    pub debug_toggle:bool,
}
pub struct MailView<'a> {
    pub pending:bool,pub index:usize,pub sender:&'a str,pub subject:&'a str,pub body_a:&'a str,
    pub body_b:&'a str,pub clue:&'a str,
}
pub struct Context<'a> {
    pub cash:i32,pub exposure:f32,pub day:u32,pub scans:u32,pub pis:u32,
    pub alarm:f32,pub active_wave:f32,pub mail:MailView<'a>,
    pub inspected:bool,pub message:&'a str,pub kids:usize,
    pub orders_pending:u32,pub deliveries_ready:u32,pub next_delivery_eta:f32,
    pub nearest_kid:f32,pub front_visitors:usize,
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Action {
    Exit, AcceptMail, DeleteMail, InspectMail, BuyItem(i32),
    Spend(i32), SecurePhone, OmniProtect, PayWage(i32),
    VideoUpload, FanPost, FakeSpoof, Cheat(CheatKind), SearchOutcome(AnswerKind), DebugEnable,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Page{Boot,Lock,Desktop}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
enum App {Mail,AmazonCrime,AppleStore,Web,Files,Settings,KidRadar,DoxxBlock,
          Studio,CamWatch,FanCord,FakeLocation}
const DOCK_APPS:[App;12]=[
    App::Mail,App::AmazonCrime,App::AppleStore,App::Web,App::Files,App::Settings,
    App::KidRadar,App::DoxxBlock,App::Studio,App::CamWatch,App::FanCord,App::FakeLocation,
];
impl App {
    fn name(self)-> &'static str {match self {
        App::Mail=>"CarterMail",App::AmazonCrime=>"AmazonCrime",App::AppleStore=>"Apple Store",
        App::Web=>"CarterWeb",App::Files=>"Files",App::Settings=>"Settings",
        App::KidRadar=>"KidRadar",App::DoxxBlock=>"DoxxBlock",App::Studio=>"CarterStudio",
        App::CamWatch=>"CamWatch",App::FanCord=>"FanCord",App::FakeLocation=>"FakeMyLocation",
    }}
    fn default_origin(self)->(i32,i32) {match self{
        App::Mail=>(480,232),App::Web=>(89,84),App::Files=>(109,89),
        App::Settings=>(192,73),_=>(192,73),
    }}
}
#[derive(Clone,Copy)]
struct Window {app:App,x:i32,y:i32}
#[derive(Clone,Copy)]
struct Drag {mx:i32,my:i32,x:i32,y:i32}

pub struct CarterOs {
    page:Page, boot_left:f32, launchpad:bool, notifications:bool,context_menu:bool,
    window:Option<Window>, parked:Vec<Window>, minimized:Vec<App>,
    positions:HashMap<App,(i32,i32)>,
    settings_tab:usize, drag:Option<Drag>,
    unavailable:Option<App>, unavailable_left:f32,
    pub system:SystemState, installed:[bool;12], toast:String,toast_left:f32,
    threat_visible:bool, notifications_cleared:bool, persist_changes:bool,
    modal:Option<&'static str>, drive:Drive, files:Vec<String>, selected_file:usize,
    file_search:String, file_search_active:bool, show_trash:bool, sort_desc:bool,
    launchpad_search:String, launchpad_search_active:bool,
    editor_target:Option<String>, editor_text:String, editor_dirty:bool,
    rename_target:Option<String>, rename_text:String, notices:Vec<String>,
    web:SearchEngine, mail_compose:bool, mail_to:String, pending_ai_result:bool,
    video_cooldown:f32, fake_cooldown:f32, shop_page:u8,
    background_seconds:f32, background_count:u32,
    search:String,search_active:bool,compromised_unlocked:bool,
}
impl CarterOs {
    pub fn new()->Self {
        let system=SystemState::load(&SystemState::path());
        let mut installed=[true,true,true,true,true,true,false,false,false,false,false,false];
        if let Ok(flags)=fs::read_to_string(".run_carter/installed_apps.cfg") {
            for (i,c) in flags.chars().enumerate().take(12) {installed[i]=c=='1';}
            installed[2]=true;installed[5]=true;
        }
        // Migrate a legacy v0.6 name-only file list only if the v2 drive does not yet exist.
        let mut drive=Drive::load(&Drive::path());
        if !Drive::path().exists(){
            if let Ok(old)=fs::read_to_string(".run_carter/virtual_files.txt"){
                for name in old.lines().take(30){
                    if drive.get(name).is_none(){let _=drive.create(name,Kind::Text);}
                }
            }
        }
        let files=drive.names(false,"");
        let positions=Self::load_positions();
        Self {page:Page::Boot,boot_left:1.4,launchpad:false,notifications:false,
              context_menu:false,window:None,parked:Vec::new(),minimized:Vec::new(),positions,settings_tab:0,drag:None,
              unavailable:None,unavailable_left:0.0,system,installed,toast:String::new(),
              toast_left:0.0,threat_visible:false,notifications_cleared:false,persist_changes:true,
              modal:None,drive,files,selected_file:0,
              file_search:String::new(),file_search_active:false,show_trash:false,sort_desc:false,
              launchpad_search:String::new(),launchpad_search_active:false,
              editor_target:None,editor_text:String::new(),editor_dirty:false,
              rename_target:None,rename_text:String::new(),notices:Vec::new(),
              web:SearchEngine::new(),mail_compose:false,mail_to:String::new(),pending_ai_result:false,video_cooldown:0.0,fake_cooldown:0.0,
              background_seconds:0.0,background_count:0,shop_page:0,
              search:String::new(),search_active:false,compromised_unlocked:false}
    }
    pub fn tick(&mut self,dt:f32)->Vec<Action> {
        self.web.poll();
        let mut events=Vec::new();
        if self.pending_ai_result {
            if let Some(ref result)=self.web.current {
                if result.kind!=AnswerKind::Loading {
                    events.push(Action::SearchOutcome(result.kind));
                    self.pending_ai_result=false;
                }
            }
        }
        if self.page==Page::Boot {self.boot_left-=dt;
            if self.boot_left<=0.0 {self.page=Page::Lock;}}
        self.unavailable_left=(self.unavailable_left-dt).max(0.0);
        self.toast_left=(self.toast_left-dt).max(0.0);
        self.video_cooldown=(self.video_cooldown-dt).max(0.0);
        self.fake_cooldown=(self.fake_cooldown-dt).max(0.0);
        if self.page==Page::Desktop && self.system.wifi {
            self.background_seconds+=dt;
            if self.background_seconds>=45.0 {
                self.background_seconds-=45.0;
                self.background_count+=1;
                let message=if self.installed[10] && self.background_count%2==0 {
                    "FanCord: another fan is asking for a free PC."
                }else{"CarterMail: unread messages need attention."};
                self.inform_background(message);
            }
        }
        events
    }
    fn inform_background(&mut self,s:&str){
        self.notices.push(s.to_owned());
        if self.notices.len()>20{self.notices.remove(0);}
        self.notifications_cleared=false;
        if !self.system.focus_mode{self.toast=s.to_owned();self.toast_left=3.5;}
    }
    fn inform(&mut self,s:&str){
        if self.notices.last().map(String::as_str)!=Some(s){
            self.notices.push(s.to_owned());
            if self.notices.len()>20{self.notices.remove(0);}
        }
        // Creator Focus silences automatic notifications, not feedback on user actions.
        self.toast=s.to_owned();self.toast_left=3.5;
    }
    fn save_settings(&mut self){
        if self.persist_changes {
            if let Err(e)=self.system.save(&SystemState::path()){eprintln!("Settings save failed: {e}");}
        }
    }
    fn save_virtual_files(&self){
        if !self.persist_changes {return;}
        if let Err(e)=self.drive.save(&Drive::path()){eprintln!("Carter Drive save failed: {e}");}
    }
    fn refresh_files(&mut self){
        self.files=self.drive.names(self.show_trash,&self.file_search);
        self.files.sort_by_key(|n|n.to_lowercase());
        if self.sort_desc{self.files.reverse();}
        self.selected_file=self.selected_file.min(self.files.len().saturating_sub(1));
    }
    fn current_file(&self)->Option<String>{self.files.get(self.selected_file).cloned()}
    fn create_file(&mut self,kind:Kind){
        let root=if kind==Kind::Folder {"New Folder"}else{"New Document"};
        let mut name=root.to_owned();let mut suffix=2;
        while self.drive.get(&name).is_some() {name=format!("{root} {suffix}");suffix+=1;}
        if let Err(err)=self.drive.create(&name,kind){self.inform(err);}else{
            self.save_virtual_files();self.refresh_files();
            self.selected_file=self.files.iter().position(|x|x==&name).unwrap_or(0);
            self.inform("New item created in Carter Drive.");
        }
    }
    fn open_current_file(&mut self){
        let Some(name)=self.current_file()else{return;};
        if let Some(entry)=self.drive.get(&name){
            if entry.kind==Kind::Folder{self.inform("Folders can be renamed or moved to Trash.");return;}
            self.editor_target=Some(name);self.editor_text=entry.content.clone();self.editor_dirty=false;
        }
    }
    fn save_editor(&mut self){
        if let Some(name)=self.editor_target.clone(){
            match self.drive.set_content(&name,&self.editor_text){
                Ok(())=>{self.editor_dirty=false;self.save_virtual_files();self.inform("Document saved to Carter Drive.");}
                Err(e)=>self.inform(e),
            }
        }
    }
    fn close_active(&mut self){
        self.window=self.parked.pop();self.drag=None;
        if self.window.is_none(){self.modal=None;}
    }
    fn minimize_active(&mut self){
        if let Some(w)=self.window.take(){
            if !self.minimized.contains(&w.app){self.minimized.push(w.app);}
        }
        self.window=self.parked.pop();self.drag=None;
    }
    fn cycle_active(&mut self){
        if let Some(previous)=self.parked.pop(){
            if let Some(current)=self.window.replace(previous){self.parked.insert(0,current);}
        }
    }
    fn load_positions()->HashMap<App,(i32,i32)>{
        let mut positions=HashMap::new();
        if let Ok(content)=fs::read_to_string(".run_carter/window_layout.cfg"){
            for line in content.lines().take(12){
                let parts:Vec<_>=line.split('|').collect();
                if parts.len()!=3 {continue;}
                if let (Some(app),Ok(x),Ok(y))=(DOCK_APPS.iter().find(|app|app.name()==parts[0]),
                        parts[1].parse::<i32>(),parts[2].parse::<i32>()){
                    if (-40..=W-200).contains(&x) && (36..=H-180).contains(&y){
                        positions.insert(*app,(x,y));
                    }
                }
            }
        }
        positions
    }
    fn save_positions(&self){
        if !self.persist_changes{return;}
        if let Err(e)=fs::create_dir_all(".run_carter"){eprintln!("Layout directory: {e}");return;}
        let mut data=String::new();
        for app in DOCK_APPS {
            if let Some((x,y))=self.positions.get(&app){data.push_str(&format!("{}|{}|{}\n",app.name(),x,y));}
        }
        let path=".run_carter/window_layout.cfg";
        let tmp=".run_carter/window_layout.tmp";
        if let Err(e)=fs::write(tmp,data).and_then(|()|fs::rename(tmp,path)){
            eprintln!("Window layout save: {e}");
        }
    }
    fn save_installed(&mut self) {
        if !self.persist_changes{return;}
        let path=".run_carter/installed_apps.cfg";
        if let Err(e)=fs::create_dir_all(".run_carter"){eprintln!("Save directory: {e}");return;}
        let flags:String=self.installed.iter().map(|b|if *b {'1'}else{'0'}).collect();
        if let Err(e)=fs::write(path,flags){eprintln!("Installed apps: {e}");}
    }
    /// Notification from gameplay; Creator Focus filters the toast but not history.
    pub fn notify(&mut self,text:&str){self.inform_background(text);}
    pub fn network_online(&self)->bool{self.system.internet_ready()}
    pub fn set_threat_visible(&mut self,visible:bool){
        if visible && !self.threat_visible {self.inform("CRITICAL: LOCATION LEAKED - kids approaching!");}
        self.threat_visible=visible;
        if visible{self.compromised_unlocked=true;}
    }
    pub fn phishing_multiplier(&self)->f32{self.system.phishing_multiplier()}
    fn open(&mut self,app:App) {
        self.launchpad=false;self.notifications=false;self.context_menu=false;self.drag=None;
        let idx=DOCK_APPS.iter().position(|a|*a==app).unwrap_or(0);
        if !self.installed[idx]{self.inform("Install this app through Apple Store first.");return;}
        if self.window.map(|w|w.app)==Some(app){return;}
        let old=self.window.take();
        let existing=self.parked.iter().position(|w|w.app==app)
            .map(|i|self.parked.remove(i));
        if let Some(w)=old {self.parked.push(w);}
        let (x,y)=self.positions.get(&app).copied().unwrap_or_else(||app.default_origin());
        self.window=Some(existing.unwrap_or(Window{app,x,y}));
        self.minimized.retain(|a|*a!=app);
        // Avoid excessive simultaneous windows on the original 1440x900 desktop.
        if self.parked.len()>5 {self.parked.remove(0);}
        self.unavailable=None;self.modal=None;self.search_active=false;self.search.clear();
        self.mail_compose=false;self.mail_to.clear();
        self.launchpad_search.clear();self.launchpad_search_active=false;
    }
    fn icon_at(x:i32,y:i32)->Option<App> {
        if !(804..=869).contains(&y){return None;}
        for (i,app) in DOCK_APPS.iter().enumerate(){
            let left=298+i as i32*64;
            if x>=left && x<left+50 {return Some(*app);}
        }
        None
    }
    fn launchpad_app(x:i32,y:i32)->Option<App>{
        // Original 4x3 icon grid in the 02 Launchpad frame.
        let xs=[309,478,647,816,985,1154];
        let ys=[341,522];
        for (row,cy) in ys.iter().enumerate(){for (col,cx) in xs.iter().enumerate(){
            if (x-*cx).abs()<52 && (y-*cy).abs()<70 {
                return DOCK_APPS.get(row*6+col).copied();
            }
        }}
        None
    }
    fn window_sprite(&self,app:App)->&'static str {
        match app {
            App::Mail=>"cartermail_window",App::Web=>"carterweb_inactive",
            App::Files=>"files_window",App::Settings=>match self.settings_tab {
                1=>"settings_sound",2=>"settings_network",3=>"settings_privacy",
                4=>"settings_personalization",_=>"settings_display"
            },_=>"settings_display",
        }
    }
    fn window_size(app:App)->(i32,i32){match app {
        App::Mail=>(852,591),App::Web=>(938,584),App::Files=>(1222,742),
        App::Settings=>(1056,762),_=>(1056,762),
    }}
    fn rect_contains(x:i32,y:i32,left:i32,top:i32,right:i32,bottom:i32)->bool {
        x>=left && x<right && y>=top && y<bottom
    }
    pub fn handle(&mut self,input:&Input,_cash:i32)->Vec<Action>{
        let mut out=Vec::new();
        if input.escape && self.mail_compose {self.mail_compose=false;self.mail_to.clear();return out;}
        if input.escape && self.search_active {self.search_active=false;return out;}
        if input.escape && self.editor_target.is_some() {
            if self.editor_dirty {self.inform("Unsaved changes: Ctrl+S to save, then Esc to close.");}
            else {self.editor_target=None;}
            return out;
        }
        if input.escape && self.rename_target.is_some(){self.rename_target=None;return out;}
        if input.escape && self.file_search_active {self.file_search_active=false;return out;}
        if input.escape && self.launchpad {self.launchpad=false;return out;}
        if input.escape {out.push(Action::Exit);return out;}
        if input.lock {
            if self.editor_dirty {self.save_editor();}
            self.page=Page::Lock;
            self.window=None;
            self.parked.clear();self.minimized.clear();
            self.editor_target=None;self.rename_target=None;
            self.drag=None;
            self.launchpad=false;
            self.notifications=false;
            self.inform("CarterOS locked. Press Enter to unlock.");
            return out;
        }
        if self.page==Page::Boot {if input.enter||input.click{self.page=Page::Lock;}return out;}
        if self.page==Page::Lock {
            if input.enter||input.click {self.page=Page::Desktop;}
            return out;
        }
        // Hidden System Settings developer control. No visual Figma elements are changed.
        if input.debug_toggle && self.window.map(|w|w.app)==Some(App::Settings) {
            self.inform("Developer Features ON. Debug cheats disable clean-run trophies.");
            out.push(Action::DebugEnable);return out;
        }
        // Compose is an actual editable recipient field. Nothing is sent over a network.
        if self.mail_compose {
            if input.escape {self.mail_compose=false;self.mail_to.clear();return out;}
            if input.backspace {self.mail_to.pop();}
            for c in input.chars.chars(){
                if self.mail_to.len()<110 && (c.is_ascii_alphanumeric() || "@._-+".contains(c)) {
                    self.mail_to.push(c);
                }
            }
            if input.enter {
                if let Some(secret)=cheats::mail(&self.mail_to){
                    out.push(Action::Cheat(secret.kind));self.inform(secret.message);
                } else if let Some(reply)=cheats::funny_mail(&self.mail_to){self.inform(reply);}
                else if !self.system.wifi {self.inform("CarterMail is offline. No message sent.");}
                else if self.mail_to.contains('@') {self.inform("Fictional message queued. No real email was sent.");}
                else {self.inform("Enter a valid fictional email address.");return out;}
                self.mail_compose=false;self.mail_to.clear();
            }
            return out;
        }
        // In-game document editing: no host file reads, writes or execution.
        if self.editor_target.is_some(){
            if input.save {self.save_editor();return out;}
            if input.enter && self.editor_text.len()<8192 {self.editor_text.push('\n');self.editor_dirty=true;}
            if input.backspace {self.editor_text.pop();self.editor_dirty=true;}
            if !input.chars.is_empty() && self.editor_text.len()+input.chars.len()<=8192 {
                self.editor_text.push_str(&input.chars);self.editor_dirty=true;
            }
            if input.click {
                let (wx,wy)=self.window.map(|w|(w.x,w.y)).unwrap_or((109,89));
                if Self::rect_contains(input.x,input.y,wx+610,wy+593,wx+790,wy+638){self.save_editor();}
                if Self::rect_contains(input.x,input.y,wx+820,wy+593,wx+1000,wy+638){
                    if self.editor_dirty {self.inform("Save before closing this document.");}
                    else{self.editor_target=None;}
                }
            }
            return out;
        }
        if self.rename_target.is_some(){
            if input.backspace{self.rename_text.pop();}
            for c in input.chars.chars(){if self.rename_text.len()<72 && (c.is_ascii_alphanumeric() || c==' ' || c=='_' || c=='-' || c=='.') {self.rename_text.push(c);}}
            if input.enter {
                if let Some(old)=self.rename_target.take(){
                    let new=self.rename_text.trim().to_owned();
                    match self.drive.rename(&old,&new) {
                        Ok(())=>{self.save_virtual_files();self.refresh_files();self.inform("Item renamed.");}
                        Err(err)=>{self.rename_target=Some(old);self.inform(err);},
                    }
                }
            }
            return out;
        }
        if self.file_search_active {
            if input.backspace{self.file_search.pop();self.refresh_files();}
            for c in input.chars.chars(){
                if self.file_search.len()<40 && (c.is_ascii_alphanumeric()||c==' '||c=='_'||c=='-'||c=='.') {
                    self.file_search.push(c);
                }
            }
            if !input.chars.is_empty(){self.refresh_files();}
            if input.enter{self.file_search_active=false;}
            return out;
        }
        if input.cycle_window{self.cycle_active();return out;}
        if input.minimize_window{self.minimize_active();return out;}
        if input.close_window{self.close_active();return out;}
        if input.maximize_window{
            self.inform("Original Figma windows keep their native size for visual fidelity.");return out;
        }
        if self.window.map(|w|w.app)==Some(App::Mail) && input.new_file {
            self.mail_compose=true;self.mail_to.clear();return out;
        }
        if self.window.map(|w|w.app)==Some(App::Files){
            if input.find{self.file_search_active=true;return out;}
            if input.new_file {self.create_file(Kind::Text);return out;}
            if input.new_folder {self.create_file(Kind::Folder);return out;}
            if input.toggle_trash {self.show_trash=!self.show_trash;self.refresh_files();return out;}
            if input.restore && self.show_trash {
                if let Some(name)=self.current_file(){self.drive.trash(&name,false);self.save_virtual_files();self.refresh_files();self.inform("Item restored.");}
                return out;
            }
            if input.rename{
                if let Some(name)=self.current_file(){self.rename_target=Some(name.clone());self.rename_text=name;}
                return out;
            }
        }
        if self.threat_visible && input.click {
            if Self::rect_contains(input.x,input.y,458,534,982,574){
                if self.installed[11] && self.system.location_permission && self.system.wifi {
                    if self.fake_cooldown==0.0 {out.push(Action::FakeSpoof);self.fake_cooldown=55.0;}
                    else {self.inform("Location decoy is recharging.");}
                }else{self.inform("Install FakeMyLocation and enable network/location access first.");}
                return out;
            }
            if Self::rect_contains(input.x,input.y,458,582,982,620){
                self.system.wifi=false;self.save_settings();
                self.inform("Network disconnected. Existing visitors may still arrive.");
                return out;
            }
        }
        if let Some(drag)=self.drag {
            if input.down {
                if let Some(w)=self.window.as_mut(){
                    let (width,height)=Self::window_size(w.app);
                    let _=height;
                    w.x=(drag.x+input.x-drag.mx).clamp(-40,W-200);
                    w.y=(drag.y+input.y-drag.my).clamp(36,H-180);
                }
                return out;
            } else {
                if let Some(w)=self.window{self.positions.insert(w.app,(w.x,w.y));self.save_positions();}
                self.drag=None;
            }
        }
        if !input.click && !input.right_click {
            if self.launchpad {
                if input.backspace {self.launchpad_search.pop();}
                if !input.chars.is_empty() {
                    for c in input.chars.chars(){if self.launchpad_search.len()<55 && (c.is_ascii_alphanumeric()||c==' '){self.launchpad_search.push(c);}}
                }
                if input.enter {
                    let q=self.launchpad_search.trim().to_lowercase();
                    if let Some(app)=DOCK_APPS.iter().find(|a|a.name().to_lowercase().contains(&q)&&!q.is_empty()) {
                        self.open(*app);
                    } else {self.inform("No matching application.");}
                    self.launchpad_search.clear();self.launchpad_search_active=false;
                }
                return out;
            }
            // A held mouse button updates the volume slider continuously.
            if input.down && self.window.map(|w|w.app)==Some(App::Settings) && self.settings_tab==1 {
                if let Some(w)=self.window {
                    let lx=input.x-w.x;let ly=input.y-w.y;
                    if (605..920).contains(&lx) && (305..362).contains(&ly) {
                        let v=((lx-625) as f32 / 230.0 * 100.0).clamp(0.0,100.0) as u8;
                        if v!=self.system.master_volume {self.system.master_volume=v;self.save_settings();}
                        return out;
                    }
                }
            }
            if self.search_active {
                if !input.chars.is_empty() {
                    let is_web=self.window.map(|w|w.app)==Some(App::Web);
                    for c in input.chars.chars() {
                        if self.search.len()>=180 {break;}
                        if !c.is_control() && (is_web || c.is_ascii_alphanumeric() || c==' ' || c=='.') {
                            self.search.push(c);
                        }
                    }
                }
                if input.backspace {self.search.pop();}
                if input.enter {
                    if self.window.map(|w|w.app)==Some(App::Settings) {
                        let q=self.search.to_ascii_lowercase();
                        let tabs=["display","sound","network","privacy","personalization"];
                        if let Some((i,_))=tabs.iter().enumerate().find(|(_,x)|x.contains(q.as_str())&&!q.is_empty()) {
                            self.settings_tab=i;
                        }
                    }
                    if self.window.map(|w|w.app)==Some(App::Web) {
                        let q=self.search.trim().to_owned();
                        // Hidden pages are routed internally and never fetched, even offline.
                        if let Some(secret)=cheats::website(&q) {
                            out.push(Action::Cheat(secret.kind));
                            self.web.show_secret(&q,secret.message);
                            self.pending_ai_result=false;
                        } else if let Some(message)=cheats::browser_easter_egg(&q) {
                            self.web.show_secret(&q,message);
                            self.pending_ai_result=false;
                        } else if !self.system.wifi {
                            self.web.show_secret(&q,"CarterWeb is offline. Reconnect Wi-Fi in Settings for ordinary searches.");
                            self.pending_ai_result=false;
                        } else {
                            self.web.submit(&q);
                            self.pending_ai_result=true;
                            if let Some(ref result)=self.web.current {
                                if result.kind!=AnswerKind::Loading {
                                    out.push(Action::SearchOutcome(result.kind));
                                    self.pending_ai_result=false;
                                }
                            }
                        }
                    }
                    self.search_active=false;
                }
                return out;
            }
            if self.window.map(|w|w.app)==Some(App::Files) {
                if input.enter{self.open_current_file();}
                if input.backspace {
                    if let Some(name)=self.current_file(){
                        self.drive.trash(&name,true);self.refresh_files();self.save_virtual_files();
                        self.inform("Moved item to the fictional Trash. Ctrl+T shows Trash.");
                    }
                }
                if input.up {self.selected_file=self.selected_file.saturating_sub(1);}
                if input.down_key && !self.files.is_empty(){self.selected_file=(self.selected_file+1).min(self.files.len()-1);}
            }
            if input.enter && self.window.map(|w|w.app)==Some(App::Mail){out.push(Action::AcceptMail);}
            if input.backspace && self.window.map(|w|w.app)==Some(App::Mail){out.push(Action::DeleteMail);}
            if input.right && self.window.map(|w|w.app)==Some(App::Mail){out.push(Action::InspectMail);}
            return out;
        }
        if input.right_click && self.window.is_none(){
            self.context_menu=true;self.notifications=false;self.launchpad=false;
            return out;
        }
        let (x,y)=(input.x,input.y);
        if self.launchpad {
            if (520..950).contains(&x) && (68..155).contains(&y){
                self.launchpad_search_active=true;return out;
            }
            if let Some(app)=Self::launchpad_app(x,y){self.open(app);}
            else if y<45 || x<200 || x>1250 {self.launchpad=false;self.launchpad_search.clear();}
            return out;
        }
        if self.context_menu {
            self.context_menu=false;
            if Self::rect_contains(x,y,535,339,818,374){
                self.create_file(Kind::Folder);self.open(App::Files);
            }else if Self::rect_contains(x,y,535,378,818,411){
                self.sort_desc=!self.sort_desc;self.refresh_files();
                self.inform(if self.sort_desc{"Sorted Z to A."}else{"Sorted A to Z."});
            }else if Self::rect_contains(x,y,535,412,818,449){
                self.refresh_files();self.inform("Desktop refreshed.");
            }else if Self::rect_contains(x,y,535,474,818,511){
                self.open(App::Settings);self.settings_tab=4;
            }else if Self::rect_contains(x,y,535,514,818,546){
                self.open(App::Settings);self.settings_tab=0;
            }else if Self::rect_contains(x,y,535,549,818,580){
                self.open(App::Settings);self.settings_tab=3;
            }
            return out;
        }
        if self.notifications {
            if Self::rect_contains(input.x,input.y,1250,600,1410,740) {
                self.notifications_cleared=true;
                self.notices.clear();self.inform("Notification history cleared.");
            }
            self.notifications=false;return out;
        }
        // The real Figma dock remains visible above open windows. Let it
        // switch applications before window hit-testing catches the click.
        if (804..870).contains(&y) {
            if let Some(app)=Self::icon_at(x,y){self.open(app);return out;}
        }
        if let Some(w)=self.window {
            let (ww,wh)=Self::window_size(w.app);
            let (lx,ly)=(x-w.x,y-w.y);
            if Self::rect_contains(lx,ly,0,0,ww,wh){
                // Native Figma traffic-light controls: close, minimize, focus.
                let cy=59;
                if (ly-cy).abs()<16 {
                    if (lx-81).abs()<12 {self.close_active();return out;}
                    if (lx-104).abs()<12 {self.minimize_active();return out;}
                    if (lx-127).abs()<12 {
                        self.inform("Window focus mode: Figma artwork stays at native size.");return out;
                    }
                }
                if w.app==App::Settings {
                    if (127..=181).contains(&ly) && (55..=285).contains(&lx) {
                        self.search_active=true;return out;
                    }
                    if (260..=550).contains(&ly) && (18..=294).contains(&lx) {
                        let centers=[293,335,377,419,461];
                        for (i,cy) in centers.iter().enumerate(){
                            if (ly-*cy).abs()<=22 {self.settings_tab=i;break;}
                        }
                        return out;
                    }
                }
                if w.app==App::Settings {
                    if self.handle_settings(lx,ly){return out;}
                }
                if !matches!(w.app,App::Settings|App::Mail|App::Web|App::Files) {
                    out.extend(self.handle_custom_app(w.app,lx,ly,_cash));
                    return out;
                }
                if w.app==App::Web {
                    if ((180..780).contains(&lx) && (90..168).contains(&ly)) ||
                       ((102..838).contains(&lx) && (207..260).contains(&ly)) {
                        self.search_active=true;
                        self.inform("Type a search question or secret URL, then press Enter.");
                    }
                    return out;
                }
                if w.app==App::Files {
                    if (351..1180).contains(&lx) && (247..595).contains(&ly) {
                        let idx=((ly-264).max(0)/65) as usize;
                        self.selected_file=idx.min(self.files.len().saturating_sub(1));
                    } else if (970..1150).contains(&lx) && (75..135).contains(&ly) {
                        self.create_file(Kind::Folder);
                    }
                    return out;
                }
                if w.app==App::Mail {
                    if Self::rect_contains(lx,ly,195,457,395,495) {
                        self.mail_compose=true;self.mail_to.clear();return out;
                    }
                    if Self::rect_contains(lx,ly,192,405,330,448) {
                        out.push(Action::DeleteMail);return out;
                    }
                    if Self::rect_contains(lx,ly,348,405,485,448) {
                        out.push(Action::InspectMail);return out;
                    }
                    if Self::rect_contains(lx,ly,505,405,651,448) {
                        out.push(Action::AcceptMail);return out;
                    }
                    if Self::rect_contains(lx,ly,251,445,408,488) ||
                       Self::rect_contains(lx,ly,708,47,730,82){
                        out.push(Action::DeleteMail);return out;
                    }
                    if Self::rect_contains(lx,ly,248,130,821,393){
                        out.push(Action::InspectMail);return out;
                    }
                }
                if ly<80 {self.drag=Some(Drag{mx:x,my:y,x:w.x,y:w.y});}
                return out;
            }
        }
        // Bring a window behind the active one to the foreground without losing state.
        if let Some(index)=self.parked.iter().rposition(|w|{
            let (ww,wh)=Self::window_size(w.app);
            Self::rect_contains(x,y,w.x,w.y,w.x+ww,w.y+wh)
        }){
            let chosen=self.parked.remove(index);
            if let Some(active)=self.window.replace(chosen){self.parked.push(active);}
            return out;
        }
        // The creator-focus switch and desktop folders are interactive.
        if self.window.is_none() && Self::rect_contains(x,y,485,342,568,407){
            self.system.focus_mode=!self.system.focus_mode;
            self.save_settings();self.inform(if self.system.focus_mode{"Creator focus enabled."}else{"Creator focus disabled."});
            return out;
        }
        if self.window.is_none() && Self::rect_contains(x,y,1260,75,1410,460){
            self.open(App::Files);return out;
        }
        // Exact bottom dock icon bounds, also present on every Figma window frame.
        if let Some(app)=Self::icon_at(x,y){self.open(app);return out;}
        // Clicking the top-left OS mark opens the exact Figma Launchpad overlay.
        if x<142 && y<35{self.launchpad=true;self.window=None;return out;}
        // The Figma notification button is the top-right action/slider icon.
        if x>=1210 && y<35 {self.notifications=true;return out;}
        // Clicking the wallpaper leaves other open apps available through the dock.
        if self.window.is_some(){self.minimize_active();}
        out
    }
    fn handle_settings(&mut self,x:i32,y:i32)->bool {
        let control=(790..980).contains(&x);
        let mut changed=false;
        match self.settings_tab {
            0 => {
                if control && (602..670).contains(&y) {
                    self.system.night_comfort=!self.system.night_comfort;changed=true;
                    self.inform(if self.system.night_comfort{"Night comfort enabled."}else{"Night comfort disabled."});
                } else if control && (420..505).contains(&y){
                    self.system.display_preset^=1;changed=true;
                    self.inform("Window size updated. CarterOS keeps its original Figma artwork.");
                } else if control && (510..595).contains(&y) {
                    self.system.interface_scale=if self.system.interface_scale==100{125}else{100};changed=true;
                    self.inform("CarterOS interface zoom updated.");
                }
            }
            1 => {
                if (605..920).contains(&x) && (305..362).contains(&y){
                    self.system.master_volume=(((x-625) as f32/230.0)*100.0).clamp(0.0,100.0) as u8;
                    changed=true;
                } else if control && (191..275).contains(&y) {
                    self.system.speakers=!self.system.speakers;changed=true;
                } else if control && (370..467).contains(&y) {
                    self.system.microphone=!self.system.microphone;changed=true;
                } else if control && (475..548).contains(&y) {
                    self.system.system_sounds=!self.system.system_sounds;changed=true;
                }
            }
            2 => {
                if control && (322..395).contains(&y) {
                    self.system.wifi=!self.system.wifi;changed=true;
                    self.inform(if self.system.wifi{"CarterPrivate connected (simulated)."}else{"CarterOS is offline."});
                } else if control && (405..490).contains(&y) {
                    self.system.public_shield=!self.system.public_shield;changed=true;
                } else if control && (503..597).contains(&y) {
                    self.inform("FREE_PC_VAN is unsecured. Connection refused.");
                }
            }
            3 => {
                if control && (290..380).contains(&y){self.system.location_permission=!self.system.location_permission;changed=true;}
                else if control && (390..480).contains(&y){self.system.camera_permission=!self.system.camera_permission;changed=true;}
                else if control && (483..576).contains(&y){self.system.doxxblock=!self.system.doxxblock;changed=true;}
                else if control && (588..660).contains(&y){self.system.diagnostics=!self.system.diagnostics;changed=true;}
            }
            4 => {
                if (328..968).contains(&x) && (215..369).contains(&y) {
                    let n=((x-328)/214).min(2) as u8;
                    if n==2 && !self.compromised_unlocked {self.inform("Compromised theme unlocks after a location leak.");}
                    else {self.system.wallpaper=n;changed=true;}
                } else if control && (409..507).contains(&y){
                    self.system.dark_appearance=!self.system.dark_appearance;changed=true;
                } else if (731..946).contains(&x) && (520..576).contains(&y){
                    self.system.accent=(((x-743)/33).clamp(0,5)) as u8;changed=true;
                }
            }
            _=>{},
        }
        if changed {self.save_settings();}
        changed
    }
    fn handle_custom_app(&mut self,app:App,x:i32,y:i32,cash:i32)->Vec<Action> {
        let mut out=Vec::new();
        if !(310..995).contains(&x){return out;}
        let selected=((y-215)/78).clamp(0,5);
        match app {
            App::AmazonCrime => {
                if !self.system.wifi {self.inform("Connect to Wi-Fi before ordering.");}
                else if (148..202).contains(&y) {
                    // Three categories. Existing Figma window chrome is preserved.
                    self.shop_page=if x<545{0}else if x<760{1}else{2};
                } else if (200..616).contains(&y){
                    let row=(y-200)/52;
                    let item=match self.shop_page{0=>row+1,1=>row+9,_=>row+17};
                    if (1..=18).contains(&item) && (self.shop_page!=2 || row<2){out.push(Action::BuyItem(item));}
                }
            }
            App::AppleStore => {
                if !self.system.wifi {self.inform("Apple Store is offline.");}
                else if (195..695).contains(&y) {
                    let idx=(selected as usize+6).min(11);
                    if self.installed[idx]{self.inform("App already installed.");}
                    else if cash>=80 {
                        self.installed[idx]=true;self.save_installed();out.push(Action::Spend(80));
                        self.inform("App installed. Open it from the dock.");
                    }else{self.inform("Not enough money to install this app.");}
                }
            }
            App::KidRadar => self.inform("KidRadar has refreshed its detector."),
            App::DoxxBlock => {
                if (205..435).contains(&y){out.push(Action::InspectMail);self.inform("Scanning current email.");}
            }
            App::Studio => {
                if self.video_cooldown==0.0 {
                    out.push(Action::VideoUpload);self.video_cooldown=50.0;
                    self.inform("Video uploaded. Popularity and exposure increased.");
                }else{self.inform("Rendering video; try again later.");}
            }
            App::CamWatch => self.inform(if self.system.camera_permission{"Security feed refreshed."}else{"Camera access is disabled in Privacy."}),
            App::FanCord => {
                if self.system.wifi {out.push(Action::FanPost);self.inform("You posted to FanCord.");}
                else {self.inform("FanCord is offline.");}
            }
            App::FakeLocation => {
                if !self.system.location_permission {self.inform("Enable location permission in Privacy.");}
                else if !self.system.wifi {self.inform("Location server is offline.");}
                else if self.fake_cooldown==0.0 {
                    out.push(Action::FakeSpoof);self.fake_cooldown=55.0;
                    self.inform("Decoy location activated for nearby fans.");
                }else {self.inform("Spoof engine recharging.");}
            }
            _=>{},
        }
        out
    }
    fn paint_toggle<D: RaylibDraw>(&self,d:&mut D,a:&Assets,x:i32,y:i32,on:bool,default_on:bool){
        if on!=default_on {a.paste(d,if on{"toggle_on"}else{"toggle_off"},x,y);}
    }
    fn dynamic_settings<D: RaylibDraw>(&self,d:&mut D,a:&Assets,w:Window){
        let x=w.x;let y=w.y;
        if self.search_active || !self.search.is_empty(){
            d.draw_rectangle(x+68,y+130,213,34,Color::new(249,248,252,255));
            d.draw_text(&self.search,x+78,y+138,15,Color::new(48,34,63,255));
        }
        match self.settings_tab {
            0 => {
                self.paint_toggle(d,a,x+903,y+621,self.system.night_comfort,false);
                if self.system.display_preset!=0 {
                    d.draw_rectangle(x+800,y+440,145,34,Color::WHITE);
                    d.draw_text("1280 x 720",x+805,y+450,17,Color::new(96,83,111,255));
                }
                if self.system.interface_scale!=100 {
                    d.draw_rectangle(x+807,y+532,136,30,Color::WHITE);
                    d.draw_text("125%",x+832,y+538,17,Color::new(96,83,111,255));
                }
            }
            1 => {
                self.paint_toggle(d,a,x+903,y+493,self.system.system_sounds,true);
                if self.system.master_volume!=64 {
                    d.draw_rectangle(x+625,y+300,325,36,Color::WHITE);
                    d.draw_rectangle(x+632,y+323,234,4,Color::new(219,216,228,255));
                    let fill=(self.system.master_volume as f32*2.34) as i32;
                    d.draw_rectangle(x+632,y+323,fill,4,Color::new(40,126,240,255));
                    d.draw_circle(x+632+fill,y+325,9.0,Color::new(241,239,246,255));
                    d.draw_circle_lines(x+632+fill,y+325,9.0,Color::new(180,174,198,255));
                    d.draw_text(&format!("{}%",self.system.master_volume),x+900,y+315,15,Color::new(98,85,113,255));
                }
                if !self.system.speakers {d.draw_rectangle(x+802,y+224,145,32,Color::WHITE);d.draw_text("Muted",x+823,y+230,16,Color::DARKGRAY);}
                if !self.system.microphone {d.draw_rectangle(x+800,y+401,148,32,Color::WHITE);d.draw_text("Disabled",x+807,y+407,16,Color::DARKGRAY);}
            }
            2 => {
                self.paint_toggle(d,a,x+903,y+350,self.system.wifi,true);
                self.paint_toggle(d,a,x+903,y+440,self.system.public_shield,true);
                if !self.system.wifi {
                    d.draw_rectangle(x+834,y+231,118,28,Color::WHITE);
                    d.draw_text("Disconnected",x+836,y+238,14,Color::RED);
                }
            }
            3 => {
                self.paint_toggle(d,a,x+903,y+510,self.system.doxxblock,true);
                self.paint_toggle(d,a,x+903,y+601,self.system.diagnostics,false);
                if !self.system.location_permission {d.draw_rectangle(x+814,y+323,134,32,Color::WHITE);d.draw_text("Blocked",x+818,y+331,16,Color::RED);}
                if !self.system.camera_permission {d.draw_rectangle(x+814,y+413,134,32,Color::WHITE);d.draw_text("Blocked",x+818,y+421,16,Color::RED);}
                if !self.system.doxxblock {d.draw_rectangle(x+720,y+526,180,28,Color::WHITE);d.draw_text("Protection off",x+725,y+531,16,Color::RED);}
            }
            4 => {
                if self.system.wallpaper!=0 {
                    // Keep original thumbnail colors, change only selection outline.
                    d.draw_rectangle_lines(x+328,y+232,204,127,Color::new(216,207,225,255));
                    let offset=if self.system.wallpaper==1{542}else{760};
                    d.draw_rectangle_lines(x+offset,y+232,202,126,Color::new(40,126,240,255));
                    d.draw_rectangle_lines(x+offset+1,y+233,200,124,Color::new(40,126,240,255));
                }
                if self.system.dark_appearance {
                    d.draw_rectangle(x+790,y+432,164,31,Color::WHITE);
                    d.draw_text("Dark",x+840,y+440,16,Color::new(96,83,111,255));
                }
                if self.system.accent>0 {
                    let colors=[Color::new(40,126,240,255),Color::new(157,110,222,255),
                        Color::new(225,109,153,255),Color::new(242,164,81,255),
                        Color::new(96,162,132,255),Color::new(123,127,145,255)];
                    d.draw_circle_lines(x+753+self.system.accent as i32*33,y+546,13.0,colors[self.system.accent as usize]);
                }
            }
            _=>{}
        }
    }
    fn draw_compose_overlay<D: RaylibDraw>(&self,d:&mut D,x:i32,y:i32){
        let navy=Color::new(48,34,63,255);let muted=Color::new(105,89,121,255);
        d.draw_rectangle(x+175,y+85,563,421,Color::new(251,249,253,255));
        d.draw_rectangle_lines(x+175,y+85,563,421,Color::new(140,124,164,255));
        d.draw_text("New CarterMail message",x+200,y+120,26,navy);
        d.draw_text("To: (fictional address)",x+200,y+186,17,muted);
        d.draw_rectangle(x+199,y+218,505,44,Color::WHITE);
        d.draw_rectangle_lines(x+199,y+218,505,44,Color::new(180,173,194,255));
        let visible:String=self.mail_to.chars().take(52).collect();
        d.draw_text(&visible,x+208,y+230,17,navy);
        d.draw_text("Enter: Send   |   Esc: Cancel",x+200,y+307,18,muted);
        d.draw_text("Your email stays inside RUN CARTER.",x+200,y+369,16,muted);
    }
    fn dynamic_mail<D: RaylibDraw>(&self,d:&mut D,w:Window,ctx:&Context<'_>){
        // The Figma mail window itself is untouched. Only the static demonstration
        // message is replaced by actual in-game text inside the original content pane.
        let x=w.x;let y=w.y;
        d.draw_rectangle(x+190,y+79,535,327,Color::new(249,248,251,255));
        if !ctx.mail.pending {
            d.draw_text("Inbox cleared for today",x+204,y+114,24,Color::new(48,34,63,255));
            d.draw_text("You can end your shift to receive new mail.",x+204,y+167,17,Color::DARKGRAY);
            d.draw_rectangle(x+508,y+409,205,31,Color::new(39,131,101,255));
            d.draw_text("END SHIFT (ENTER)",x+519,y+417,14,Color::WHITE);
            d.draw_text("Press Ctrl+N to compose a fictional email",x+204,y+218,16,Color::DARKGRAY);
            if self.mail_compose {self.draw_compose_overlay(d,x,y);}
            return;
        }
        let navy=Color::new(48,34,63,255);let muted=Color::new(105,89,121,255);
        d.draw_text(ctx.mail.subject,x+199,y+96,22,navy);
        d.draw_text(ctx.mail.sender,x+200,y+131,15,muted);
        d.draw_line(x+200,y+159,x+708,y+159,Color::new(220,217,228,255));
        d.draw_text(ctx.mail.body_a,x+200,y+189,17,muted);
        d.draw_text(ctx.mail.body_b,x+200,y+220,17,muted);
        if ctx.inspected {d.draw_text(ctx.mail.clue,x+200,y+278,15,Color::new(18,128,90,255));}
        if !self.system.wifi {d.draw_text("OFFLINE - connect in Settings",x+200,y+315,17,Color::RED);}
        d.draw_rectangle(x+192,y+409,135,27,Color::new(40,126,240,255));
        d.draw_text("DELETE",x+224,y+416,14,Color::WHITE);
        d.draw_rectangle(x+351,y+409,132,27,Color::new(240,239,247,255));
        d.draw_text("INSPECT",x+375,y+416,14,navy);
        d.draw_rectangle(x+508,y+409,135,27,Color::new(39,131,101,255));
        d.draw_text("ACCEPT (ENTER)",x+516,y+417,13,Color::WHITE);
        d.draw_text(&format!("Email #{} | Cash ${} | Exposure {}%",ctx.mail.index+1,ctx.cash,ctx.exposure as i32),x+199,y+360,15,muted);
        d.draw_rectangle(x+195,y+457,196,33,Color::new(40,126,240,255));
        d.draw_text("COMPOSE (Ctrl+N)",x+202,y+466,16,Color::WHITE);
        if self.mail_compose {self.draw_compose_overlay(d,x,y);}
    }
    fn draw_virtual_app<D: RaylibDraw>(&self,d:&mut D,app:App,w:Window,ctx:&Context<'_>){
        let x=w.x;let y=w.y;
        // Only the body of an otherwise existing Figma window is extended. Figma
        // does not contain designs for these apps, so this surface isn't exact.
        d.draw_rectangle(x+298,y+87,681,594,Color::new(249,248,251,255));
        d.draw_rectangle(x+55,y+87,235,593,Color::new(232,230,240,255));
        let navy=Color::new(48,34,63,255);let muted=Color::new(116,103,130,255);
        d.draw_text(app.name(),x+315,y+107,30,navy);
        d.draw_line(x+315,y+159,x+964,y+159,Color::new(219,215,227,255));
        d.draw_text("RUN CARTER  |  CarterOS",x+74,y+172,18,navy);
        d.draw_text("Functional in-game application",x+74,y+210,15,muted);
        let mut rows:Vec<String>=Vec::new();
        match app {
            App::AmazonCrime=>{
                if self.shop_page==0 {
                    for s in ["Raspberry Pi decoy     $85","AntiDoxx patch     $95",
                              "3 email scanners     $65","Reinforced door     $190",
                              "AIRHORN     $120   [Z]","CONFETTI CANNON     $220   [X]",
                              "DECOY DRONE     $300   [C]","FICTIONAL SUGAR PACK     $75   [R]"] {rows.push(s.into());}
                } else {
                    let first=if self.shop_page==1{0}else{8};
                    for w in crate::arsenal::CATALOG.iter().skip(first).take(8) {
                        rows.push(format!("{}   ${}    {} charges",w.name,w.cost,w.charges));
                    }
                }
                rows.push(format!("Orders: {} en route | {} ready at front door",
                    ctx.orders_pending,ctx.deliveries_ready));
                if ctx.orders_pending>0 {rows.push(format!("Next delivery in {} seconds",ctx.next_delivery_eta.max(0.0).ceil() as i32));}
            },
            App::AppleStore=>{for (i,a) in DOCK_APPS.iter().enumerate().skip(6){rows.push(format!("{}     {}",a.name(),if self.installed[i]{"INSTALLED"}else{"INSTALL $80"}));}},
            App::KidRadar=>{
                rows.push(format!("Nearby visitors: {}",ctx.kids));
                if ctx.nearest_kid>=0.0 {rows.push(format!("Closest fan: {:.1} m away",ctx.nearest_kid));}
                else {rows.push("No kids detected within the playable area".into());}
                if ctx.alarm>=0.0 {rows.push(format!("Incoming wave in {:.0} seconds",ctx.alarm.ceil()));}
                else {rows.push("No incoming wave currently detected".into());}
                rows.push("Detection uses the in-game office coordinates".into());
            },
            App::DoxxBlock=>{rows.push(format!("Protection: {}",if self.system.doxxblock{"ON"}else{"OFF"}));rows.push(format!("Exposure: {}%",ctx.exposure as i32));rows.push(format!("Email scanner charges: {}",ctx.scans));rows.push("Click to inspect current email".into());},
            App::Studio=>{rows.push("CreatorStudio / projects".into());rows.push("Build_047.cstudio".into());rows.push(format!("Upload readiness: {}",if self.video_cooldown==0.0{"READY"}else{"RENDERING"}));rows.push("CLICK TO UPLOAD (+$140, more attention)".into());},
            App::CamWatch=>{
                if self.system.camera_permission {rows.push(format!("Front entrance  |  {} visitors",ctx.front_visitors));}
                else {rows.push("FEED BLOCKED — enable camera access in Privacy".into());}
                rows.push(format!("Camera permission: {}",if self.system.camera_permission{"GRANTED"}else{"DENIED"}));
                rows.push("Live feed is a simulation, not a real camera.".into());
            },
            App::FanCord=>{rows.push("# general   |   # free-pc-requests".into());rows.push("Little Timmy: FREE PC PLS".into());rows.push("Anonymous: Found your house LOL".into());rows.push("Click to post (+attention)".into());},
            App::FakeLocation=>{rows.push("Decoy: Fake PC giveaway warehouse".into());rows.push(format!("Location access: {}",if self.system.location_permission{"ALLOWED"}else{"BLOCKED"}));rows.push(format!("Spoof: {}",if self.fake_cooldown==0.0{"READY"}else{"RECHARGING"}));rows.push("Click to activate location decoy".into());},
            _=>{},
        }
        if app==App::AmazonCrime {
            d.draw_rectangle(x+308,y+149,655,47,Color::new(238,235,246,255));
            d.draw_text("GADGETS",x+332,y+161,16,navy);
            d.draw_text("ARMORY 1",x+542,y+161,16,navy);
            d.draw_text("ARMORY 2",x+759,y+161,16,navy);
            let tabx=x+318+self.shop_page as i32*219;
            d.draw_rectangle(tabx,y+188,164,4,Color::new(46,126,240,255));
        }
        for (i,line) in rows.iter().enumerate(){
            let compact=app==App::AmazonCrime;
            let yy=y+if compact{207+i as i32*52}else{216+i as i32*78};
            let rh=if compact{46}else{57};
            if yy<y+625 {
                d.draw_rectangle(x+310,yy-6,650,rh,Color::WHITE);
                d.draw_rectangle_lines(x+310,yy-6,650,rh,Color::new(226,223,232,255));
                d.draw_text(&line.chars().take(65).collect::<String>(),x+327,yy+7,if compact{16}else{18},navy);
            }
        }
        d.draw_text(&format!("Day {}   Cash ${}   Exposure {:.0}%",ctx.day,ctx.cash,ctx.exposure),x+320,y+627,16,muted);
    }
    fn draw_window<D: RaylibDraw>(&self,d:&mut D,assets:&Assets,w:Window,ctx:&Context<'_>){
        if w.app==App::Mail{assets.draw(d,"carterweb_inactive",0,0);}
        let (px,py)=w.app.default_origin();
        assets.draw(d,self.window_sprite(w.app),w.x-px,w.y-py);
        match w.app {
            App::Settings=>self.dynamic_settings(d,assets,w),
            App::Mail=>self.dynamic_mail(d,w,ctx),
            App::Files=>self.dynamic_files(d,w),
            App::Web=>self.dynamic_web(d,w),
            _=>self.draw_virtual_app(d,w.app,w,ctx),
        }
    }
    fn draw_toast<D: RaylibDraw>(&self,d:&mut D){
        if self.toast_left>0.0 {
            let navy=Color::new(43,34,65,239);
            let tw=(self.toast.len() as i32*8+32).clamp(230,650);
            d.draw_rectangle(1440-tw-29,54,tw,42,navy);
            d.draw_text(&self.toast,1440-tw-14,67,15,Color::WHITE);
        }
    }
    pub fn draw<D: RaylibDraw>(&self,d:&mut D,assets:&Assets,ctx:&Context<'_>){
        if self.page==Page::Boot {assets.draw(d,"full_frame_12",0,0);return;}
        if self.page==Page::Lock {assets.draw(d,"full_frame_11",0,0);return;}
        let leak=ctx.alarm>=0.0||ctx.active_wave>=0.0;
        if leak {assets.draw(d,"full_frame_14",0,0);return;}
        if self.launchpad {
            assets.draw(d,"full_frame_02",0,0);
            if !self.launchpad_search.is_empty(){
                d.draw_rectangle(562,72,350,40,Color::new(250,249,253,250));
                d.draw_text(&self.launchpad_search,581,85,20,Color::new(48,34,63,255));
                let q=self.launchpad_search.to_lowercase();
                if let Some(app)=DOCK_APPS.iter().find(|a|a.name().to_lowercase().contains(&q)){
                    d.draw_rectangle(558,135,395,49,Color::new(248,247,251,250));
                    d.draw_text(&format!("Enter to open {}",app.name()),570,150,19,Color::new(48,34,63,255));
                }
            }
            self.draw_toast(d);return;
        }
        // Unmodified native source artboard for the original idle state.
        let original_bg=self.system.wallpaper==0;
        if !self.notifications && !self.context_menu && original_bg && self.parked.is_empty() {
            if let Some(w)=self.window {
                if (w.x,w.y)==w.app.default_origin() {
                    let full=match w.app {
                        App::Mail=>Some("full_frame_10"),App::Files=>Some("full_frame_04"),
                        App::Settings=>Some(match self.settings_tab {
                            1=>"full_frame_06",2=>"full_frame_07",3=>"full_frame_08",
                            4=>"full_frame_09",_=>"full_frame_05",
                        }),_=>None,
                    };
                    if let Some(src)=full {
                        assets.draw(d,src,0,0);
                        if w.app==App::Settings{self.dynamic_settings(d,assets,w);}
                        if w.app==App::Mail{self.dynamic_mail(d,w,ctx);}
                        if w.app==App::Files{self.dynamic_files(d,w);}
                        self.draw_toast(d);return;
                    }
                }
            } else {
                assets.draw(d,"full_frame_01",0,0);
                if !self.system.focus_mode{assets.paste(d,"toggle_off",507,351);}
                self.draw_toast(d);return;
            }
        }
        // Independently extracted original layers retain pixel positions.
        match self.system.wallpaper {
            2=>assets.draw(d,"leak_bg",0,0),
            1=>assets.draw(d,"wallpaper_after_hours",0,0),
            _=>assets.draw(d,"desktop_bg",0,0),
        }
        assets.draw(d,"menu_bar",0,0);assets.draw(d,"dock",0,0);
        if self.window.is_none(){assets.draw(d,"desktop_widgets",0,0);}
        for w in &self.parked {self.draw_window(d,assets,*w,ctx);}
        if let Some(w)=self.window {self.draw_window(d,assets,w,ctx);}
        if self.context_menu{assets.draw(d,"context_menu",0,0);}
        if self.notifications{
            assets.draw(d,"notification_center",0,0);
            if self.notifications_cleared && self.notices.is_empty() {
                d.draw_rectangle(1020,200,380,420,Color::new(249,248,251,255));
                d.draw_text("No new notifications",1070,305,19,Color::DARKGRAY);
            } else if !self.notices.is_empty() {
                d.draw_rectangle(1020,215,377,360,Color::new(249,248,251,255));
                for (i,item) in self.notices.iter().rev().take(6).enumerate() {
                    let clipped:String=item.chars().take(43).collect();
                    d.draw_text(&clipped,1037,232+i as i32*48,15,Color::new(48,34,63,255));
                    d.draw_line(1030,265+i as i32*48,1380,265+i as i32*48,Color::new(232,229,238,255));
                }
            }
        }
        self.draw_toast(d);
    }
    fn dynamic_files<D: RaylibDraw>(&self,d:&mut D,w:Window){
        let x=w.x;let y=w.y;
        d.draw_rectangle(x+351,y+247,822,350,Color::new(249,248,251,255));
        let navy=Color::new(48,34,63,255);
        d.draw_text(if self.show_trash{"TRASH"}else{"CARTER DRIVE"},x+363,y+251,16,navy);
        if self.file_search_active || !self.file_search.is_empty(){
            d.draw_rectangle(x+820,y+174,338,36,Color::new(244,243,250,255));
            d.draw_text(&format!("Search: {}",self.file_search),x+832,y+184,16,navy);
        }
        for (i,name) in self.files.iter().take(5).enumerate(){
            let yy=y+280+i as i32*59;
            if self.selected_file==i{d.draw_rectangle(x+351,yy-3,818,44,Color::new(215,228,249,255));}
            let prefix=if self.drive.get(name).map(|e|e.kind==Kind::Folder).unwrap_or(false){"[Folder] "}else{""};
            d.draw_text(&format!("{prefix}{name}"),x+375,yy+8,17,navy);
        }
        d.draw_text("Enter: open | Backspace: Trash | F2: rename",x+363,y+610,15,Color::DARKGRAY);
        d.draw_text("Ctrl+N: new text | Ctrl+Shift+N: folder | Ctrl+T: Trash",x+363,y+635,15,Color::DARKGRAY);
        if let Some(ref original)=self.rename_target{
            d.draw_rectangle(x+365,y+360,720,160,Color::new(241,239,248,250));
            d.draw_text(&format!("Rename: {original}"),x+385,y+380,17,navy);
            d.draw_text(&self.rename_text,x+385,y+415,19,navy);
            d.draw_text("Enter to confirm / Esc to cancel",x+385,y+460,16,Color::DARKGRAY);
        }
        if let Some(ref file)=self.editor_target {
            d.draw_rectangle(x+300,y+118,880,560,Color::new(251,251,253,255));
            d.draw_rectangle_lines(x+300,y+118,880,560,Color::new(210,205,224,255));
            d.draw_text(&format!("{}{}",file,if self.editor_dirty{" *"}else{""}),x+328,y+142,24,navy);
            d.draw_text("In-game plain-text editor (no host files)",x+328,y+183,16,Color::DARKGRAY);
            for (i,line) in self.editor_text.lines().take(14).enumerate(){
                let safe:String=line.chars().take(80).collect();
                d.draw_text(&safe,x+330,y+218+i as i32*27,18,navy);
            }
            d.draw_rectangle(x+610,y+593,180,45,Color::new(40,126,240,255));
            d.draw_text("SAVE (Ctrl+S)",x+625,y+606,16,Color::WHITE);
            d.draw_rectangle(x+820,y+593,180,45,Color::new(232,228,240,255));
            d.draw_text("CLOSE (Esc)",x+839,y+606,16,navy);
        }
    }
    fn dynamic_web<D: RaylibDraw>(&self,d:&mut D,w:Window){
        let (x,y)=(w.x,w.y);
        let navy=Color::new(48,34,63,255);let muted=Color::new(107,94,123,255);
        d.draw_rectangle(x+76,y+120,784,393,Color::new(249,248,251,255));
        // Existing Figma browser chrome stays untouched; only the content is dynamic.
        d.draw_text("CarterSearch",x+110,y+141,30,navy);
        d.draw_text("Search the internet. Trust it at your own risk.",x+110,y+178,17,muted);
        d.draw_rectangle(x+102,y+209,736,46,Color::WHITE);
        d.draw_rectangle_lines(x+102,y+209,736,46,Color::new(202,191,221,255));
        let prompt=if self.search_active {format!("{}|",self.search)}else if !self.search.is_empty(){self.search.clone()}else{"Search anything or enter a URL (click address bar)".into()};
        d.draw_text(&prompt.chars().take(70).collect::<String>(),x+117,y+224,17,muted);
        if !self.system.wifi {
            d.draw_text("OFFLINE - enable Wi-Fi in Settings",x+110,y+287,19,Color::RED);
        } else if let Some(ref result)=self.web.current {
            if result.kind==AnswerKind::NoResults {
                // Intentional 9% dead air: no status, fake result or answer is displayed.
                return;
            }
            let heading=match result.kind {
                AnswerKind::Ai=>"LOCAL AI RESPONSE", AnswerKind::Comedic=>"UNRELIABLE / SATIRE",
                AnswerKind::NoResults=>"NOTHING FOUND",AnswerKind::OfflineFallback=>"OFFLINE FALLBACK (NOT AI)",
                AnswerKind::Secret=>"HIDDEN PAGE FOUND",AnswerKind::Loading=>"GENERATING...",
            };
            d.draw_text(heading,x+111,y+278,14,Color::new(40,126,240,255));
            d.draw_text(&result.title.chars().take(73).collect::<String>(),x+111,y+302,22,navy);
            let mut at=341;
            for line in Self::wrap_text(&result.answer,83).iter().take(5){
                d.draw_text(line,x+113,y+at,16,muted);at+=24;
            }
            if let Some((ref name,ref desc))=result.snippets.first(){
                d.draw_line(x+111,y+468,x+820,y+468,Color::new(223,218,232,255));
                d.draw_text(&name.chars().take(60).collect::<String>(),x+111,y+475,15,Color::new(40,126,240,255));
                d.draw_text(&desc.chars().take(80).collect::<String>(),x+111,y+495,13,muted);
            }
        } else {
            d.draw_text("CarterWeb isn't connected to the actual public web.",x+110,y+293,19,muted);
            d.draw_text("Some results are AI-generated. Others are spectacularly wrong.",x+110,y+326,16,muted);
        }
    }
    fn wrap_text(text:&str,width:usize)->Vec<String>{
        let mut lines=Vec::new();let mut line=String::new();
        for paragraph in text.lines(){
            if paragraph.is_empty(){if !line.is_empty(){lines.push(std::mem::take(&mut line));}continue;}
            for word in paragraph.split_whitespace(){
                if line.len()+word.len()+1>width && !line.is_empty(){lines.push(std::mem::take(&mut line));}
                if !line.is_empty(){line.push(' ');}line.push_str(word);
            }
            if !line.is_empty(){lines.push(std::mem::take(&mut line));}
        }
        lines
    }
    fn draw_unavailable<D: RaylibDraw>(&self,_d:&mut D){}

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn exact_artboard_and_layer_count(){assert_eq!((W,H),(1440,900));assert_eq!(LAYERS.len(),36);}
    #[test] fn core_icon_hitboxes(){assert_eq!(CarterOs::icon_at(324,840),Some(App::Mail));assert_eq!(CarterOs::icon_at(643,840),Some(App::Settings));}
    #[test] fn login_and_settings_navigation(){
        let mut os=CarterOs::new();os.tick(3.0);assert_eq!(os.page,Page::Lock);
        let mut input=Input::default();input.enter=true;os.handle(&input,0);assert_eq!(os.page,Page::Desktop);
        os.open(App::Settings);let mut click=Input::default();click.click=true;
        click.x=285;click.y=73+335;os.handle(&click,0);assert_eq!(os.settings_tab,1);
    }
    #[test] fn icon_grid_matches_source_artboard(){
        assert_eq!(CarterOs::launchpad_app(309,341),Some(App::Mail));
        assert_eq!(CarterOs::launchpad_app(1154,522),Some(App::FakeLocation));
    }
    #[test] fn no_sorryinc_apps_appear(){assert_eq!(DOCK_APPS.len(),12);}
    #[test] fn gameplay_notice_enters_history_even_with_creator_focus(){
        let mut os=CarterOs::new();os.persist_changes=false;
        os.system.focus_mode=true;os.notify("Delivery waiting");
        assert!(os.notices.iter().any(|n|n=="Delivery waiting"));
        assert_eq!(os.toast_left,0.0);
    }

    #[test] fn interactive_night_comfort_and_network(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;os.open(App::Settings);
        os.settings_tab=0;
        let old=os.system.night_comfort;
        assert!(os.handle_settings(925,638));assert_ne!(os.system.night_comfort,old);
        os.settings_tab=2;let wifi=os.system.wifi;
        assert!(os.handle_settings(925,363));assert_ne!(os.system.wifi,wifi);
    }
    #[test] fn dock_switches_application_over_an_open_window(){
        let mut os=CarterOs::new();os.page=Page::Desktop;os.open(App::Settings);
        let mut click=Input::default();click.click=true;click.x=324;click.y=812;
        os.handle(&click,0);
        assert_eq!(os.window.map(|w|w.app),Some(App::Mail));
    }
    #[test] fn lock_screen_shortcut_preserves_settings(){
        let mut os=CarterOs::new();os.page=Page::Desktop;os.open(App::Settings);
        os.system.night_comfort=true;
        let mut input=Input::default();input.lock=true;os.handle(&input,0);
        assert_eq!(os.page,Page::Lock);assert!(os.window.is_none());
        input.lock=false;input.enter=true;os.handle(&input,0);
        assert_eq!(os.page,Page::Desktop);assert!(os.system.night_comfort);
    }
    #[test] fn volume_slider_changes_persistent_state(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;os.open(App::Settings);
        os.settings_tab=1;
        let mut click=Input::default();click.click=true;click.x=192+830;click.y=73+335;
        os.handle(&click,0);
        assert!(os.system.master_volume>64);
    }
    #[test] fn all_apps_can_be_opened_after_install(){
        let mut os=CarterOs::new();os.page=Page::Desktop;
        os.installed=[true;12];
        for app in DOCK_APPS {os.open(app);assert_eq!(os.window.map(|w|w.app),Some(app));}
    }
    #[test] fn multiple_windows_preserve_position_and_order(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;
        os.open(App::Settings);os.window.as_mut().unwrap().x=265;
        os.open(App::Files);
        assert_eq!(os.parked.len(),1);
        assert_eq!(os.parked[0].app,App::Settings);
        os.open(App::Settings);
        assert_eq!(os.window.unwrap().x,265);
        assert_eq!(os.parked[0].app,App::Files);
        os.minimize_active();
        assert_eq!(os.window.map(|w|w.app),Some(App::Files));
        os.open(App::Settings);
        assert_eq!(os.window.map(|w|w.app),Some(App::Settings));
        os.close_active();
        assert_eq!(os.window.map(|w|w.app),Some(App::Files));
    }
    #[test] fn virtual_drive_file_lifecycle_via_os(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;
        os.open(App::Files);
        os.create_file(Kind::Text);
        let name=os.current_file().unwrap();
        os.open_current_file();
        assert_eq!(os.editor_target.as_deref(),Some(name.as_str()));
        os.editor_text="Hello from CarterOS".into();
        os.save_editor();
        assert_eq!(os.drive.get(&name).unwrap().content,"Hello from CarterOS");
        os.editor_target=None;
        os.drive.trash(&name,true);os.refresh_files();
        assert!(!os.files.contains(&name));
        os.show_trash=true;os.refresh_files();
        assert!(os.files.contains(&name));
    }
    #[test] fn desktop_context_menu_reaches_settings(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;
        os.context_menu=true;
        let mut i=Input::default();i.click=true;i.x=600;i.y=530;
        os.handle(&i,0);
        assert_eq!(os.window.map(|w|w.app),Some(App::Settings));
        assert_eq!(os.settings_tab,0);
    }
    #[test] fn launchpad_search_opens_matching_app(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;
        os.launchpad=true;
        let mut input=Input::default();input.chars="cartermail".into();
        os.handle(&input,0);
        input.chars.clear();input.enter=true;
        os.handle(&input,0);
        assert_eq!(os.window.map(|w|w.app),Some(App::Mail));
        assert!(!os.launchpad);
    }
    #[test] fn carterweb_secret_url_from_address_bar(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;
        os.system.wifi=true;os.open(App::Web);
        let mut input=Input::default();input.click=true;input.x=89+300;input.y=84+140;
        os.handle(&input,0);assert!(os.search_active);
        input.click=false;input.chars="https://pleasedonthurtmeimfragile.com".into();
        os.handle(&input,0);input.chars.clear();input.enter=true;
        let acts=os.handle(&input,0);
        assert!(acts.contains(&Action::Cheat(CheatKind::GodMode)));
        assert_eq!(os.web.current.as_ref().unwrap().kind,AnswerKind::Secret);
    }
    #[test] fn compose_secret_mail_and_normal_mail(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;
        os.open(App::Mail);
        let mut input=Input::default();input.new_file=true;
        os.handle(&input,0);assert!(os.mail_compose);
        input.new_file=false;input.chars="guardianangel@outofoffice.example".into();
        os.handle(&input,0);input.chars.clear();input.enter=true;
        let acts=os.handle(&input,0);
        assert!(acts.contains(&Action::Cheat(CheatKind::GodMode)));
        assert!(!os.mail_compose);
    }
    #[test] fn creator_focus_silences_noncritical_background_popups(){
        let mut os=CarterOs::new();os.persist_changes=false;os.page=Page::Desktop;
        os.system.focus_mode=true;
        os.toast.clear();os.toast_left=0.0;
        os.inform_background("Noncritical inbox reminder");
        assert_eq!(os.toast_left,0.0);
        assert_eq!(os.notices.len(),1);
        os.system.focus_mode=false;
        os.inform_background("Second reminder");
        assert!(os.toast_left>0.0);
    }

}
