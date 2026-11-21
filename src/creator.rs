//! Decisions for the two fictional social apps. Daily results and one-time payouts survive reloads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Campaign {
    pub studio_project: u8, // 0 legacy / no upload, 1-3 project number
    pub studio: u8, // 0 idle, 1 flop comments, 2 success comments, 3 moderated
    pub fan_day: u32,
    pub fan_offer: u8, // 0 none, 1 dubious, 2 legitimate, 3 resolved
    pub pc_debt: i32,
    pub relocations: u32,
    pub incident: u8,
    pub lead: u8,
    pub morning: u8,
    pub spare_pc: bool,
    pub consult_done: bool,
    pub remote_next: bool,
    pub remote_today: bool,
    pub overhead_due: i32,
    pub archive_tip: bool,
}

pub const PROJECTS: [(&str, i32, f32); 3] = [
    ("Quiet cable guide", 1_100, 6.0),
    ("Budget PC build", 2_200, 13.0),
    ("Live setup reveal", 3_600, 23.0),
];

pub fn upload(seed: u64, day: u32, project: usize) -> Option<(bool, i32, f32)> {
    let &(_, pay, exposure) = PROJECTS.get(project)?;
    let roll = crate::inbox::mix(seed ^ u64::from(day) ^ 0x9d82_f7a3) % 100;
    let succeeds = roll < [80, 65, 52][project];
    Some((succeeds, if succeeds { pay } else { pay / 5 }, if succeeds { exposure } else { exposure / 3.0 }))
}

pub const CONSULT_PAY: i32 = 1_100;
pub const SPARE_COST: i32 = 1_450;

pub struct Comment {
    pub line: &'static str,
    pub labels: [&'static str;3],
    pub choices: [(i32,f32,u8);3],
}

pub fn comment(seed: u64, day: u32, hit: bool, project: u8) -> Comment {
    let roll = crate::inbox::mix(seed ^ u64::from(day) ^ 0x62fa_a53d);
    if project != 0 && roll % 2 == 0 {
        return match (project, hit) {
            (1, true) => Comment {
                line: "A repair club wants to teach your cable guide.",
                labels: ["Send the worksheets | Workshop deal tomorrow", "Sell a download | +$850 and +8 exposure", "Book an office tour | +$1,800 and +28 exposure"],
                choices: [(0,-5.0,6),(850,8.0,0),(1800,28.0,3)],
            },
            (1, false) => Comment {
                line: "A viewer spotted the wrong connector in your guide.",
                labels: ["Publish a correction | -12 exposure", "Make a lesson with the repair club", "Defend the clip live | +$700 and +24 exposure"],
                choices: [(0,-12.0,0),(0,-3.0,6),(700,24.0,3)],
            },
            (2, true) => Comment {
                line: "Your parts list sold out. A shop wants an exclusive.",
                labels: ["Publish substitute parts | Remote deal tomorrow", "Take the exclusive | +$1,600 and +16 exposure", "Invite cameras | +$2,400 and +34 exposure"],
                choices: [(0,-8.0,2),(1600,16.0,0),(2400,34.0,3)],
            },
            (2, false) => Comment {
                line: "Viewers say the budget build went over budget.",
                labels: ["Publish the full receipt | -14 exposure", "Offer a cheaper build | Workshop deal tomorrow", "Double down live | +$900 and +31 exposure"],
                choices: [(0,-14.0,0),(0,-4.0,6),(900,31.0,3)],
            },
            (_, true) => Comment {
                line: "The sponsor wants the uncut reveal. Your door is visible.",
                labels: ["Pull the clip | -27 exposure", "License a clean cut | Sponsor deal tomorrow", "Sell the raw footage | +$2,500 and +38 exposure"],
                choices: [(0,-27.0,0),(0,-13.0,1),(2500,38.0,3)],
            },
            (_, false) => Comment {
                line: "The stream went dark. Someone clipped the last frame.",
                labels: ["File a takedown | -18 exposure", "Explain the outage | Workshop deal tomorrow", "Restart the stream | +$950 and +35 exposure"],
                choices: [(0,-18.0,0),(0,3.0,6),(950,35.0,3)],
            },
        };
    }
    let pick = roll % 3;
    match (pick,hit) {
        (0,true) => Comment {
            line:"Top comment: Is that your office in the reflection?",
            labels:["Crop the shot  |  -20 exposure","Pin a correction  |  -6 exposure","Post the full clip  |  +$1,350 and +24 exposure"],
            choices:[(0,-20.0,1),(0,-6.0,2),(1350,24.0,3)],
        },
        (1,true) => Comment {
            line:"Top comment: We can hear a street name in the audio!",
            labels:["Replace the audio  |  -24 exposure","Pin an audio note  |  +4 exposure","Sell a remix to the sponsor  |  +$1,200 and +26 exposure"],
            choices:[(0,-24.0,1),(0,4.0,2),(1200,26.0,3)],
        },
        (_,true) => Comment {
            line:"Top comment: I mapped every storefront in that video.",
            labels:["Hide the storefronts  |  -19 exposure","Pin a debunk  |  -11 exposure","Boost the thread  |  +$1,550 and +32 exposure"],
            choices:[(0,-19.0,1),(0,-11.0,2),(1550,32.0,3)],
        },
        (0,false) => Comment {
            line:"Top comment: Bet you won't show us your full setup.",
            labels:["Limit replies  |  -9 exposure","Pin a teaser for a safer follow-up","Take the dare  |  +$600 and +34 exposure"],
            choices:[(0,-9.0,1),(0,0.0,2),(600,34.0,3)],
        },
        (1,false) => Comment {
            line:"Top comment: Make a shorter version and I'll share it.",
            labels:["Re-edit the video  |  -7 exposure","Pin the viewer's suggestion","Livestream the next take  |  +$500 and +25 exposure"],
            choices:[(0,-7.0,1),(0,0.0,2),(500,25.0,3)],
        },
        _ => Comment {
            line:"Top comment: Pin this and I'll share the uncut take.",
            labels:["Remove the uncut take  |  -12 exposure","Pin your own cut  |  +3 exposure","Let the clip spread  |  +$450 and +30 exposure"],
            choices:[(0,-12.0,1),(0,3.0,2),(450,30.0,3)],
        },
    }
}

pub fn fan_persona(seed:u64,day:u32)->usize {
    (crate::inbox::mix(seed ^ u64::from(day) ^ 0xfb84_f621)%3) as usize
}

pub fn fan_copy(offer:u8, persona:usize)->[&'static str;2] {
    if matches!(offer,1|2) {
        let ask = if offer==1 {
            ["Can your address confirm the booking?","Upload the unreleased live setup footage.","Open the partner-kit attachment to claim the deal."]
        } else {
            ["Can your office host one camera for the crossover?","Can we license your unreleased setup footage?","Can we courier demo hardware for a collab?"]
        }[persona];
        [["@FrameLab | Partnership desk","@LayerSix | Creator network","@PixelCrate | Collaboration team"][persona],ask]
    } else { ["",""] }
}

impl Campaign {
    pub fn fan_today(&self, seed: u64, day: u32) -> u8 {
        if self.fan_day == day { return self.fan_offer; }
        let roll = crate::inbox::mix(seed ^ u64::from(day) ^ 0x82ad_b369) % 100;
        if matches!(self.morning,3|5) && roll<65 { return if roll<32 {2} else {1}; }
        match roll { 0..=14 => 2, 15..=34 => 1, _ => 0 }
    }

    pub fn encode(&self) -> String {
        format!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",self.studio,self.fan_day,self.fan_offer,self.pc_debt,
            self.relocations,self.incident,self.lead,self.morning,u8::from(self.spare_pc),u8::from(self.consult_done),
            u8::from(self.remote_next),u8::from(self.remote_today),self.overhead_due,u8::from(self.archive_tip),self.studio_project)
    }
    pub fn decode(input:&str)->Option<Self> {
        let nums=input.split(',').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().ok()?;
        let mut state=Self::default();
        let &[studio,fan_day,fan_offer,debt,ref extra @ ..]=nums.as_slice() else {return None;};
        if studio>3 || fan_offer>3 || debt>i32::MAX as u32 {return None;}
        state.studio=studio as u8;
        state.fan_day=fan_day;
        state.fan_offer=fan_offer as u8;
        state.pc_debt=debt as i32;
        if let &[r,i,l,m,s,c,n,t,ref tail @ ..]=extra {
            if i>2 || !matches!(l,0..=3 | 6) || m>6 || [s,c,n,t].into_iter().any(|flag|flag>1) {return None;}
            state.relocations=r;
            state.incident=i as u8;
            state.lead=l as u8;
            state.morning=m as u8;
            state.spare_pc=s==1;
            state.consult_done=c==1;
            state.remote_next=n==1;
            state.remote_today=t==1;
            if let &[overhead,tip,ref project @ ..]=tail {
                if overhead>i32::MAX as u32 || tip>1 {return None;}
                state.overhead_due=overhead as i32;
                state.archive_tip=tip==1;
                match project {
                    [] => {},
                    [id] if *id <= 3 => state.studio_project = *id as u8,
                    _ => return None,
                }
            } else if !tail.is_empty() {return None;}
        } else if !extra.is_empty() {return None;}
        Some(state)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_project_keeps_its_comment_and_workshop_follow_up() {
        let day = 9;
        let seed = (0..100).find(|&seed| comment(seed, day, true, 1).choices[0].2 == 6).unwrap();
        let campaign = Campaign { studio: 2, studio_project: 1, lead: 6, ..Campaign::default() };
        let restored = Campaign::decode(&campaign.encode()).unwrap();
        assert_eq!(restored, campaign);
        let event = comment(seed, day, true, restored.studio_project);
        assert!(event.line.contains("repair club"));
        assert_ne!(event.line, comment(seed, day, true, 3).line);
        assert_eq!(crate::inbox::follow_up(restored.lead).unwrap().payout, 2600);
    }

    #[test]
    fn resolved_collab_and_debt_stay_resolved_after_reload() {
        let state = Campaign { studio: 3, fan_day: 14, fan_offer: 3, pc_debt: 5000, spare_pc: true, lead: 3, ..Campaign::default() };
        let restored = Campaign::decode(&state.encode()).unwrap();
        assert_eq!(restored, state);
        assert_eq!(restored.fan_today(6, 14), 3);
        assert!(restored.fan_today(6, 15) < 3);
    }
}
