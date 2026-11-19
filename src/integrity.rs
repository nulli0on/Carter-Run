//! Best-effort, non-cryptographic detection of accidental or obvious save modification.
//! This is NOT anti-cheat security: files and this code are controlled by the player.
use std::{fs,io,path::Path};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Result {Untracked,Unchanged,Modified,Unreadable}
fn tag(data:&[u8])->u64{
    let mut sum:u64=0xcbf29ce484222325;
    for b in b"RUN_CARTER_LOCAL_V10".iter().chain(data.iter()){
        sum^=u64::from(*b);sum=sum.wrapping_mul(0x100000001b3);
    }
    sum
}
fn stamp_path()->&'static str{".run_carter/game_v10.check"}
pub fn seal(path:&Path)->io::Result<()> {
    let data=fs::read(path)?;
    fs::create_dir_all(".run_carter")?;
    let temp=".run_carter/game_v10.check.tmp";
    fs::write(temp,format!("RUN_CARTER_CHECK_V1\n{:016x}\n",tag(&data)))?;
    fs::rename(temp,stamp_path())
}
pub fn verify(path:&Path)->Result {
    if !Path::new(stamp_path()).exists(){return Result::Untracked;}
    let Ok(signature)=fs::read_to_string(stamp_path())else{return Result::Unreadable};
    let Ok(data)=fs::read(path)else{return Result::Unreadable};
    let expected=format!("RUN_CARTER_CHECK_V1\n{:016x}\n",tag(&data));
    if signature==expected {Result::Unchanged}else{Result::Modified}
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn checksum_changes_when_payload_changes(){assert_ne!(tag(b"cash=100"),tag(b"cash=999"));}
}
