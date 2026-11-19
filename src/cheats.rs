//! Fictional in-game easter eggs. URLs are parsed as text and NEVER fetched.
//! Email recipients are also fictional, not sent to any mail server.
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheatKind { Money, GodMode, MaxStats, Items, ClearAggro, Noclip }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Secret { pub kind: CheatKind, pub message: &'static str }

pub const ACHIEVEMENT_ID: &str = "SHAME_SHAME_SHAME";
pub const ACHIEVEMENT_NAME: &str = "Shame, Shame, Shame!";

const MONEY_DOMAINS: &[&str] = &["iamapussypleasegivememoney.com","whoateallmyramen.com","thebankofbaddecisions.example"];
const GOD_DOMAINS: &[&str] = &["pleasedonthurtmeimfragile.com","mommytheyreshootingatme.org","icantdodgeanything.net","myhealthbarhasanxiety.example"];
const STATS_DOMAINS: &[&str] = &["idontwanttoplaythegame.com","skipstraighttothecredits.org","carrymepleaseimtrash.gg","certifiedmaincharacter.example"];
const ITEM_DOMAINS: &[&str] = &["ineedaimbotandwallhacks.net","giveittomebecauseicantaim.com","skillissuecompensation.org","thepiisalie.example"];
const AGGRO_DOMAINS: &[&str] = &["itwasjustaprankbro.gov","officerididntdoanything.com","witnessprotectionforidiots.org","imactuallyjusttheintern.example"];
const NOCLIP_DOMAINS: &[&str] = &["helpimstuckinawallagain.com","getmeoutofhereimscared.net","collisionisafriendlysuggestion.example"];

pub fn host_from_input(raw:&str)->Option<String>{
    let s=raw.trim().to_ascii_lowercase();
    if s.len()>240 || !s.is_ascii() || s.chars().any(|c|c.is_ascii_whitespace()) {return None;}
    let rest=if let Some(x)=s.strip_prefix("https://"){x}
        else if let Some(x)=s.strip_prefix("http://"){x}
        else {s.as_str()};
    let hostname=rest.split(&['/','?', '#', ':'][..]).next()?.trim_start_matches("www.");
    if hostname.is_empty() || hostname.starts_with('.') || hostname.ends_with('.') {return None;}
    if !hostname.chars().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||c=='-'||c=='.'){return None;}
    Some(hostname.to_owned())
}

pub fn website(raw:&str)->Option<Secret>{
    let host=host_from_input(raw)?;
    let mapping:[(&[&str],CheatKind,&str);6]=[
        (MONEY_DOMAINS,CheatKind::Money,"The imaginary central bank has made a terrible mistake. +$100,000!"),
        (GOD_DOMAINS,CheatKind::GodMode,"The universe reluctantly agrees to stop hurting you."),
        (STATS_DOMAINS,CheatKind::MaxStats,"Your character has been promoted directly to Level 99. Nobody is impressed."),
        (ITEM_DOMAINS,CheatKind::Items,"A suspicious warehouse has delivered a lifetime supply of distractions."),
        (AGGRO_DOMAINS,CheatKind::ClearAggro,"Every kid suddenly remembers they have homework. Wanted level erased."),
        (NOCLIP_DOMAINS,CheatKind::Noclip,"Collision is now optional. Pressing this cheat again restores physics."),
    ];
    mapping.iter().find(|(domains,_,_)|domains.contains(&host.as_str()))
        .map(|(_,kind,message)|Secret{kind:*kind,message:*message})
}

pub fn mail(raw:&str)->Option<Secret>{
    let recipient=raw.trim().to_ascii_lowercase();
    let mapping:[(&str,CheatKind,&str);9]=[
        ("payroll@theuniverse-owes-me.example",CheatKind::Money,"Payroll: We accidentally classified you as the CEO's CEO. Please don't tell accounting."),
        ("refunds@gravity-support.example",CheatKind::Noclip,"Gravity Support: Your physics subscription has been cancelled. You're welcome."),
        ("guardianangel@outofoffice.example",CheatKind::GodMode,"Your guardian angel replies: I have finally read the employee handbook. No damage for you."),
        ("fasttrack@definitely-not-cheating.example",CheatKind::MaxStats,"HR: After careful review, we have skipped the part where you earn any skills."),
        ("warehouse@unreasonablygenerous.example",CheatKind::Items,"Warehouse: Your order of 30 emergency Raspberry Pis has been mislabelled as office supplies."),
        ("principal@thekidsarelate.example",CheatKind::ClearAggro,"School attendance office: Every visitor has detention. They must leave immediately."),
        ("help@theplotarmorunion.example",CheatKind::GodMode,"The Plot Armor Union has approved your emergency membership."),
        ("accounting@infinite-budget.example",CheatKind::Money,"Accounting: We found your missing decimal point. It was very large."),
        ("oops@everyoneforgetstoday.example",CheatKind::ClearAggro,"Memory Department: All of today's free-PC requests have been misfiled.")
    ];
    mapping.iter().find(|(email,_,_)|recipient==*email)
       .map(|(_,kind,message)|Secret{kind:*kind,message:*message})
}

pub fn browser_easter_egg(raw:&str)->Option<&'static str>{
    match host_from_input(raw)?.as_str(){
        "404brainnotfound.net"=>Some("ERROR 404: A functioning brain could not be located. Have you checked the recycling bin?"),
        "freepcforliterallyeveryone.example"=>Some("Congratulations! You have won ONE (1) photo of a computer. Shipping: $4,999."),
        "whyistheracoonintheoffice.example"=>Some("The raccoon is now the head of IT. It has already improved productivity."),
        "thecakeisacpu.example"=>Some("Tech support confirms that frosting is NOT an approved thermal interface material."),
        "meetingthatcouldhavebeenanemail.example"=>Some("Calendar: A six-hour meeting has been cancelled. Everyone lives."),
        "thekidsareunionizing.example"=>Some("The Free PC Union has negotiated three PCs and unlimited juice boxes."),
        "whydidyouclickthis.example"=>Some("You clicked a link titled Why Did You Click This. We have no further questions."),
        "canyourunawayfromyourfans.example"=>Some("Carterpedia: Running from fans is technically cardio, not a marketing strategy."),
        "calltheitraccoon.example"=>Some("The raccoon has fixed your router and is requesting dental insurance."),
        "mygpuismadeofcheese.example"=>Some("Benchmark: 280 FPS. Also 17 grams of cheddar per frame."),
        "thefloorisjustawebsite.example"=>Some("Floor.exe has stopped responding. Please remain above ground."),
        "pleaseletmeinimthepizza.example"=>Some("The pizza was refused entry. The pizza was also nine children in a trench coat."),
        "howtouninstallchildren.example"=>Some("Control Panel > Uninstall Programs > Unfortunately They Are Real."),
        "wheredidmydoorgo.example"=>Some("Your door has left the chat. Please contact the hallway administrator."),
        _=>None
    }
}

/// Easter egg recipients give jokes, never real email or cheat effects.
pub fn funny_mail(raw:&str)->Option<&'static str>{
    match raw.trim().to_ascii_lowercase().as_str(){
        "jobapplication@raccoon-hr.example"=>Some("Thank you. We hired the raccoon instead. It asked for fewer breaks."),
        "cancellation@freepc-subscription.example"=>Some("Your subscription to free-PC requests cannot be cancelled. It has been renewed for eternity."),
        "support@chairs-with-feelings.example"=>Some("Your chair would like you to stop blaming it for your gaming skills."),
        "hello@theinfinitehallway.example"=>Some("Delivery failed: the recipient is still walking toward the end of the hallway."),
        "dearcarter@fanswithoutlimits.example"=>Some("AUTOREPLY: Thank you for your thoughtful email. The fans have taken over this mailbox."),
        _=>None,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CheatState {
    pub god_mode:bool,pub noclip:bool,pub ever_used:bool,
    pub pending_steam:bool,pub steam_recorded:bool,pub uses:u32,
}
impl CheatState {
    pub fn path()->PathBuf{PathBuf::from(".run_carter/cheat_state_v1.cfg")}
    pub fn read()->Self {
        let Ok(text)=fs::read_to_string(Self::path()) else {return Self::default()};
        let mut state=Self::default();
        for line in text.lines(){let Some((k,v))=line.split_once('=')else{continue};
            match k {"god_mode"=>state.god_mode=v=="1", "noclip"=>state.noclip=v=="1",
               "ever_used"=>state.ever_used=v=="1", "pending_steam"=>state.pending_steam=v=="1",
               "steam_recorded"=>state.steam_recorded=v=="1",
               "uses"=>{state.uses=v.parse().unwrap_or(0).min(99999)},_=>{}}
        }
        state
    }
    pub fn write(&self)->std::io::Result<()> {
        fs::create_dir_all(".run_carter")?;
        let contents=format!("god_mode={}\nnoclip={}\never_used={}\npending_steam={}\nsteam_recorded={}\nuses={}\n",
            self.god_mode as u8,self.noclip as u8,self.ever_used as u8,
            self.pending_steam as u8,self.steam_recorded as u8,self.uses);
        let path=Self::path();let temp=path.with_extension("tmp");
        fs::write(&temp,contents)?;fs::rename(temp,path)
    }
    pub fn applied(&mut self) -> bool {
        let first=!self.ever_used;
        self.ever_used=true;
        if !self.steam_recorded{self.pending_steam=true;}
        self.uses=self.uses.saturating_add(1);
        first
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test] fn provided_websites_work_without_network(){
    for url in ["https://pleasedonthurtmeimfragile.com", "https://mommytheyreshootingatme.org", "https://icantdodgeanything.net"] { assert_eq!(website(url).unwrap().kind,CheatKind::GodMode);}
    for u in ["https://idontwanttoplaythegame.com","https://skipstraighttothecredits.org","https://carrymepleaseimtrash.gg"]{assert_eq!(website(u).unwrap().kind,CheatKind::MaxStats);}
    for u in ["https://ineedaimbotandwallhacks.net","https://giveittomebecauseicantaim.com","https://skillissuecompensation.org"]{assert_eq!(website(u).unwrap().kind,CheatKind::Items);}
    for u in ["https://itwasjustaprankbro.gov","https://officerididntdoanything.com","https://witnessprotectionforidiots.org"]{assert_eq!(website(u).unwrap().kind,CheatKind::ClearAggro);}
    for u in ["https://helpimstuckinawallagain.com","https://getmeoutofhereimscared.net"]{assert_eq!(website(u).unwrap().kind,CheatKind::Noclip);}
    assert_eq!(website("https://iamapussypleasegivememoney.com").unwrap().kind,CheatKind::Money);
  }
  #[test] fn no_host_suffix_trick_or_actual_network(){
    assert!(website("https://pleasedonthurtmeimfragile.com.evil.test").is_none());
    assert!(website("my favorite dog").is_none());
    assert!(website("https://pleasedonthurtmeimfragile.com/secret").is_some());
  }
  #[test] fn secret_mail_and_first_use(){
    assert_eq!(mail("GUARDIANANGEL@OUTOFOFFICE.EXAMPLE").unwrap().kind,CheatKind::GodMode);
    let mut state=CheatState::default();assert!(state.applied());assert!(!state.applied());
    assert_eq!(state.uses,2);assert!(state.pending_steam);
  }
}
