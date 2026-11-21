//! Persistent, entirely fictional Carter Drive. Never reads or modifies host documents.
//! Text records are hex-encoded for safe delimiters.
use std::fs;
use std::path::{Path, PathBuf};

const MAX_ENTRIES: usize = 128;
const MAX_TEXT_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { Folder, Text, Project }
impl Kind {
    fn code(self) -> &'static str {match self {Self::Folder=>"D",Self::Text=>"T",Self::Project=>"P"}}
    fn from_code(s: &str) -> Option<Self> {match s {"D"=>Some(Self::Folder),"T"=>Some(Self::Text),"P"=>Some(Self::Project),_=>None}}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {pub name:String, pub kind:Kind, pub content:String, pub trashed:bool}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drive {pub items:Vec<Entry>}

fn encode(s: &str) -> String {
    const HEX: &[u8;16]=b"0123456789abcdef";
    let mut out=String::with_capacity(s.len()*2);
    for b in s.bytes(){out.push(HEX[(b >> 4) as usize] as char);out.push(HEX[(b & 15) as usize] as char);}
    out
}
fn decode(s: &str) -> Option<String> {
    if s.len()%2!=0 || s.len()>MAX_TEXT_BYTES*2 {return None;}
    let mut bytes=Vec::with_capacity(s.len()/2);
    for ch in s.as_bytes().chunks_exact(2) {
        let hi=(ch[0] as char).to_digit(16)?;
        let lo=(ch[1] as char).to_digit(16)?;
        bytes.push(((hi << 4)|lo) as u8);
    }
    String::from_utf8(bytes).ok()
}

impl Default for Drive {
    fn default() -> Self {Self {items:vec![
        Entry{name:"Escape plan.txt".into(),kind:Kind::Text,
            content:"IF THE KIDS GET INSIDE:\n1. Grab a Raspberry Pi.\n2. Do not promise a free PC.\n3. RUN, CARTER.".into(),trashed:false},
        Entry{name:"Build_047.cstudio".into(),kind:Kind::Project,
            content:"CarterStudio project #47\nStatus: incomplete".into(),trashed:false},
        Entry{name:"No_free_PCs.pdf".into(),kind:Kind::Text,
            content:"Official policy: no free PCs available at this time.".into(),trashed:false},
        Entry{name:"Projects".into(),kind:Kind::Folder,content:String::new(),trashed:false},
    ]}}
}
impl Drive {
    pub fn path() -> PathBuf {crate::storage::path("drive_v2.cfg")}
    pub fn load(path:&Path) -> Self {
        let Ok(contents)=fs::read_to_string(path) else {return Self::default()};
        let mut items=Vec::new();
        if !contents.starts_with("CARTER_DRIVE_V2\n"){return Self::default();}
        for line in contents.lines().skip(1).take(MAX_ENTRIES) {
            let fields:Vec<_>=line.split('|').collect();
            if fields.len()!=4 {continue;}
            let (Some(kind),Some(name),Some(content))=(Kind::from_code(fields[0]),decode(fields[2]),decode(fields[3])) else {continue};
            if !valid_name(&name) || content.len()>MAX_TEXT_BYTES || items.iter().any(|e:&Entry|e.name==name){continue;}
            items.push(Entry{kind,trashed:fields[1]=="1",name,content});
        }
        Self{items}
    }
    pub fn save(&self,path:&Path) -> std::io::Result<()> {
        let mut result=String::from("CARTER_DRIVE_V2\n");
        for e in self.items.iter().take(MAX_ENTRIES) {
            result.push_str(&format!("{}|{}|{}|{}\n",e.kind.code(),u8::from(e.trashed),encode(&e.name),encode(&e.content)));
        }
        crate::storage::replace(path, &result)
    }
    pub fn names(&self,show_trash:bool,query:&str)->Vec<String> {
        let query=query.to_lowercase();
        self.items.iter().filter(|e| e.trashed==show_trash && e.name.to_lowercase().contains(&query))
            .map(|e|e.name.clone()).collect()
    }
    pub fn get(&self,name:&str)->Option<&Entry>{self.items.iter().find(|e|e.name==name)}
    pub fn create(&mut self,name:&str,kind:Kind)->Result<(),&'static str>{
        if self.items.len()>=MAX_ENTRIES {return Err("Drive is full");}
        if !valid_name(name){return Err("Invalid name");}
        if self.items.iter().any(|e|e.name.eq_ignore_ascii_case(name)){return Err("Name already exists");}
        self.items.push(Entry{name:name.into(),kind,content:String::new(),trashed:false});Ok(())
    }
    pub fn rename(&mut self,old:&str,new:&str)->Result<(),&'static str>{
        if !valid_name(new){return Err("Invalid name");}
        if self.items.iter().any(|e|e.name.eq_ignore_ascii_case(new) && e.name!=old){return Err("Name already exists");}
        let item=self.items.iter_mut().find(|e|e.name==old).ok_or("File not found")?;
        item.name=new.to_string();Ok(())
    }
    pub fn set_content(&mut self,name:&str,content:&str)->Result<(),&'static str>{
        if content.len()>MAX_TEXT_BYTES {return Err("Text limit: 8192 bytes");}
        let item=self.items.iter_mut().find(|e|e.name==name).ok_or("File not found")?;
        if item.kind==Kind::Folder {return Err("Folders cannot contain document text");}
        if item.trashed{return Err("Restore item before editing");}
        item.content=content.into();Ok(())
    }
    pub fn trash(&mut self,name:&str,trashed:bool)->bool{
        if let Some(item)=self.items.iter_mut().find(|e|e.name==name){item.trashed=trashed;true}else{false}
    }
}
fn valid_name(name:&str)->bool{
    !name.is_empty() && name.len()<=72 && !name.chars().any(|c| matches!(c, '/' | '\\' | '\n' | '\r' | '|'))
        && !name.starts_with('.') && name.trim()==name
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn name_rejects_paths_duplicates_and_bad_names(){
        let mut d=Drive::default();
        assert!(d.create("../host",Kind::Text).is_err());
        assert!(d.create("Projects",Kind::Folder).is_err());
        assert!(d.create("new.txt",Kind::Text).is_ok());
        assert!(d.rename("new.txt","final.txt").is_ok());
    }
    #[test] fn text_round_trip_and_trash_restoration(){
        let mut d=Drive::default();d.create("test.txt",Kind::Text).unwrap();
        d.set_content("test.txt","A | B\nUnicode: こんにちは").unwrap();
        d.trash("test.txt",true);
        let p=std::env::temp_dir().join(format!("carteros_drive_{}_{}.cfg",std::process::id(),"roundtrip"));
        d.save(&p).unwrap();
        let mut after=Drive::load(&p);let _=fs::remove_file(&p);
        assert_eq!(d,after);assert!(after.names(false,"").iter().all(|n|n!="test.txt"));
        after.trash("test.txt",false);
        assert!(after.names(false,"").iter().any(|n|n=="test.txt"));
    }
    #[test] fn text_limit_and_folder_restriction(){
        let mut d=Drive::default();assert!(d.set_content("Projects","hello").is_err());
        assert!(d.set_content("Escape plan.txt",&"x".repeat(9000)).is_err());
    }
}
