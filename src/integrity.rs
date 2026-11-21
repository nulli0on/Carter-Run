//! Best-effort, non-cryptographic detection of accidental or obvious save modification.
//! This is NOT anti-cheat security: files and this code are controlled by the player.
use std::{fs,path::Path};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Result {Untracked,Unchanged,Modified,Unreadable}
fn tag(data:&[u8])->u64{
    let mut sum:u64=0xcbf29ce484222325;
    for b in b"RUN_CARTER_LOCAL_V10".iter().chain(data.iter()){
        sum^=u64::from(*b);sum=sum.wrapping_mul(0x100000001b3);
    }
    sum
}
pub fn protect(body: &str) -> String {
    format!("{body}checksum={:016x}\n", tag(body.as_bytes()))
}

pub fn verify(path: &Path) -> Result {
    let Ok(data) = fs::read_to_string(path) else { return Result::Unreadable; };
    verify_text(&data, path)
}

pub fn verify_text(data: &str, path: &Path) -> Result {
    if let Some((body, checksum)) = data.rsplit_once("\nchecksum=") {
        let hash_at = body.len() + 1;
        let expected = format!("{:016x}", tag(data[..hash_at].as_bytes()));
        return if checksum.trim_end_matches(['\r', '\n']) == expected { Result::Unchanged } else { Result::Modified };
    }
    let signature = match fs::read_to_string(path.with_file_name("game_v10.check")) {
        Ok(signature) => signature,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Result::Untracked,
        Err(_) => return Result::Unreadable,
    };
    let expected = format!("RUN_CARTER_CHECK_V1\n{:016x}\n", tag(data.as_bytes()));
    if signature == expected { Result::Unchanged } else { Result::Modified }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn checksum_changes_when_payload_changes(){assert_ne!(tag(b"cash=100"),tag(b"cash=999"));}
}
