use std::sync::mpsc::{self, Receiver, TryRecvError};

#[derive(Clone, Debug)]
pub struct Mail {
    pub sender: String,
    pub subject: String,
    pub first_line: String,
    pub second_line: String,
    pub extra_line: Option<String>,
    pub clue: &'static str,
    pub payout: i32,
    pub leak: f32,
    pub scam: bool,
}

pub fn mix(mut n: u64) -> u64 {
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    n ^ (n >> 31)
}

fn pick(seed: u64, values: &[&'static str]) -> &'static str {
    values[(seed % values.len() as u64) as usize]
}

pub fn count(day: u32) -> usize {
    (3 + day / 12).min(10) as usize
}

fn kind_for(seed:u64,day:u32,index:usize)->usize {
    if day==1 { [0,1,2][index%3] }
    else { (index+(mix(seed^u64::from(day))%5) as usize)%5 }
}

pub fn valid_entry(entry:&str)->bool {
    !entry.contains('|') && entry.bytes().all(|b|b.is_ascii_graphic()||b==b' ')
        && ((1..=65).contains(&entry.len()) || valid_draft(entry))
}

fn valid_draft(entry:&str)->bool {
    let fields=entry.split('~').collect::<Vec<_>>();
    if let [subject,opening,detail]=fields.as_slice() {
        [(*subject,44),(*opening,65),(*detail,65)].into_iter().all(|(field,max)| {
            (5..=max).contains(&field.len()) && !field.contains(['@','$','/','\x5c',':','<','>','|'])
                && field.bytes().all(|b|b.is_ascii_graphic()||b==b' ')
                && field.chars().next().is_some_and(|c|c.is_ascii_alphabetic())
        })
    } else {false}
}

pub fn generate(seed: u64, day: u32, index: usize, live: &[String]) -> Mail {
    let n = mix(seed ^ (u64::from(day) << 32) ^ index as u64);
    let product = pick(n >> 18, &["desk fan", "GPU stand", "keyboard", "mini PC", "case", "capture card", "mousepad", "cable kit"]);
    let name = pick(n >> 6, &["ByteBubble", "DeskGoblin", "PixelCrate", "FrameForge", "CableClub", "FanStack", "RGB Bureau", "Quiet Supply"]);
    let sender = name.to_lowercase().replace(' ', "-");
    let kind=kind_for(seed,day,index);
    let (sender, subject, first_line, second_line, clue, payout, leak) = match kind {
        0 => {
            let pay = 2_250 + (n % 7) as i32 * 175;
            (format!("partners@{sender}.example"), format!("{name} | {product} review (${pay})"),
            format!("Would you feature our new {product} in a build?"), "Reply to accept. The review unit ships prepaid.",
            "Verified sponsor. No download or private details needed.", pay, (6 + day / 9).min(27) as f32)
        }
        1 => (format!("timmy{}@inbox.example", 100 + n % 900),
            pick(n, &["Quick question about my free PC", "Re: my 5090", "Can I pick it up today?", "One tiny upgrade request"]).into(),
            format!("Can you build me a PC with a free {product}?"), "My budget is exposure. Dad says that's a real currency.",
            "A real fan. Replying raises attention and earns nothing.", 0, (5 + day / 5).min(28) as f32),
        2 => (format!("verify@{sender}-urgent.fake"),
            pick(n, &["Shipping verification required", "Your channel is on hold", "Private stream leaked", "Final warning from support"]).into(),
            format!("We put your {product} shipment on hold."), "Install the attached address verifier to release it.",
            "Unverified sender. The installer leaks your office location.", 0, if day == 1 { 28.0 } else { (63 + day / 2).min(100) as f32 }),
        3 => {
            let pay = 3_600 + (n % 7) as i32 * 375;
            (format!("live@{sender}.example"), format!("Live desk reveal (${pay})"),
            format!("Can your {product} appear in a live desk tour?"), "Higher pay. The chat will see more of your office.",
            "Legitimate high-exposure contract. Prepare before accepting.", pay, (24 + day / 6).min(46) as f32)
        }
        _ => (format!("claims@{sender}-gift.fake"),
            format!("Claim your free {product} before midnight"), "Our giveaway bot selected the office account.".into(),
            "Open the attached installer and confirm your street address.",
            "Fake prize. The attachment sends location data.", 0, (74 + day / 2).min(100) as f32),
    };
    let mut mail=Mail {sender,subject,first_line,second_line:second_line.into(),extra_line:None,clue,payout,leak,scam:matches!(kind,2|4)};
    if let Some(entry)=live.get(index).filter(|entry|valid_entry(entry)) {
        let fields=entry.split('~').collect::<Vec<_>>();
        if let [subject,opening,detail]=fields.as_slice() {
            if !valid_draft(entry) {return mail;}
            mail.subject=if payout>0 {format!("{subject} (${payout})")} else {subject.to_string()};
            mail.first_line=opening.to_string();
            mail.extra_line=Some(detail.to_string());
        } else {mail.first_line=entry.clone();}
    }
    mail
}

pub fn follow_up(lead:u8)->Option<Mail> {
    let (subject,opening,terms,pay,leak) = match lead {
        1 => ("Sponsor follow-up: the clean edit ($1,700)","We saw the revised upload and want to use that cut.",
            "Approve the license from your desk. No location required.",1700,3.0),
        2 => ("Viewer follow-up: remote build ($2,250)","Your reply got our producer interested in your work.",
            "Record a follow-up without showing the office exterior.",2250,12.0),
        6 => ("Repair club workshop ($2,600)", "Your viewers asked for a private workshop.",
            "Send a recording through CarterMail. No office visit.", 2600, 4.0),
        5 => ("Campaign on hold","We saw the location rumors tied to yesterday's clip.",
            "We pulled the paid campaign. No action is required.",0,0.0),
        3 => ("Sponsor follow-up: the viral clip ($3,800)","We can pay for a second take while the clip is trending.",
            "Accepting books a live segment and increases exposure.",3800,36.0),
        4 => ("Verified FanCord remote deal ($4,800)","We confirmed the original account through CarterMail.",
            "Approve remote use of the footage with no visit.",4800,9.0),
        _ => return None,
    };
    Some(Mail {sender:"partnerships@creator-desk.example".into(),subject:subject.into(),first_line:opening.into(),
        second_line:terms.into(),extra_line:None,clue:"This follows a choice you made last shift.",payout:pay,leak,scam:false})
}

fn parse_drafts(answer:&str, day:u32, seed:u64)->Option<Vec<String>> {
    let mut batch=compose(day,seed);
    let mut received=false;
    for line in answer.lines() {
        let Some((slot,draft))=line.trim().split_once('~') else {continue;};
        let Some(index)=slot.trim().parse::<usize>().ok().and_then(|i|i.checked_sub(1)) else {continue;};
        let draft=draft.split('~').map(str::trim).collect::<Vec<_>>().join("~");
        if index<batch.len() && valid_draft(&draft) {
            batch[index]=draft;
            received=true;
        }
    }
    received.then_some(batch)
}

pub fn request_next(day:u32,seed:u64)->Option<Receiver<Option<Vec<String>>>> {
    let model=crate::local_ai::configured_model()?;
    let (send,receive)=mpsc::channel();
    std::thread::spawn(move || {
        let briefs=["A genuine sponsor wants a quiet PC hardware review",
            "A fan asks for a free PC in a funny way",
            "A suspicious sender claims a hardware shipment is on hold",
            "A real sponsor wants a live office tour",
            "A suspicious giveaway organizer says a hardware prize is waiting"];
        let briefs=(0..count(day)).map(|index|format!("{} {}",index+1,briefs[kind_for(seed,day,index)]))
            .collect::<Vec<_>>().join("
");
        let prompt=format!("Write fictional email pitches to a PC creator on day {day}. For each brief output exactly one row in the format index~subject~opening~detail. Use ASCII. Subject 5 to 44 characters, opening and detail 5 to 65 each. Short natural sentences. No amounts, addresses, URLs, or instructions. Only rows.
{briefs}");
        let batch=crate::local_ai::ask(&model,&prompt).and_then(|answer|parse_drafts(&answer,day,seed));
        let _=send.send(batch);
    });
    Some(receive)
}

pub fn poll(receiver: &Receiver<Option<Vec<String>>>) -> Result<Option<Vec<String>>, TryRecvError> {
    receiver.try_recv()
}

// Runs locally before the title screen and on each shift. Copy varies independently
// of the envelope, contract terms, and scanner verdict.
pub fn compose(day: u32, seed: u64) -> Vec<String> {
    const SUBJECTS: [[&str; 4]; 5] = [
        ["A quieter build", "Bench review invitation", "A loaner for your channel", "Can you try our prototype"],
        ["My budget is vibes", "A small follow-up", "Did I win a computer", "Free PC application"],
        ["Dispatch record incomplete", "Warehouse exception", "Your package is pending", "A note from shipping support"],
        ["Come live from your desk", "Studio tour invitation", "A set for our launch stream", "Behind your build"],
        ["The prize is still unclaimed", "One hardware box left", "Alternate winner selected", "A giveaway for your channel"],
    ];
    const OPENINGS: [[&str; 4]; 5] = [
        ["We liked the way you diagnosed that noisy fan.", "Our engineering team saw your teardown.", "We have a new batch on the test bench.", "The team wants an outside opinion before launch."],
        ["I cleared the biggest shelf in my room.", "My parents finally said yes to a computer.", "I already have a case and half a keyboard.", "Everyone at school thinks I won your raffle."],
        ["The courier sent this order back to the depot.", "The label is apparently missing one detail.", "We moved your parcel to the exception queue.", "The warehouse marked the shipment undeliverable."],
        ["Our viewers keep asking where you film.", "The launch team has an opening in the stream.", "We want to see your setup in working order.", "We are booking live segments for the launch."],
        ["The first winner never claimed the hardware.", "Our giveaway bot selected your channel.", "We found a sealed prize box in the stockroom.", "Your account is next on the prize list."],
    ];
    const DETAILS: [[&str; 4]; 5] = [
        ["We can provide a sample for the recording.", "An honest review would help the design team.", "A short bench demo would fit our launch.", "We would love a build with real measurements."],
        ["A fast graphics card is my only requirement.", "I promise to put your sticker on the front.", "Can it be here before my friends come over?", "My allowance is mostly in arcade tokens."],
        ["Your parcel is still on the pallet for now.", "Dispatch says it can go on the next van.", "The record needs one last approval.", "The depot cannot clear it automatically."],
        ["We would feature the desk in the main broadcast.", "The producer wants a short walkthrough.", "The host will cut to your office during the show.", "A camera will stay with the build for the segment."],
        ["The prize includes a box of accessories.", "The draw closes before the end of the shift.", "We have kept the box aside for the moment.", "The original winner has already been removed."],
    ];
    (0..count(day)).map(|index| {
        let kind = kind_for(seed, day, index);
        let roll = mix(seed ^ (u64::from(day) << 32) ^ index as u64 ^ 0x4f31_b816);
        format!("{}~{}~{}", pick(roll, &SUBJECTS[kind]),
            pick(roll >> 12, &OPENINGS[kind]), pick(roll >> 24, &DETAILS[kind]))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_mail_fills_slots_without_changing_terms() {
        let batch=parse_drafts("3~Fresh shipment status~Your package is gathering dust.~The scanner is having a rough day.
1~bad@domain~Bad instructions~Click this test",1,42).unwrap();
        assert_eq!(batch.len(),3);
        let mail=generate(42,1,2,&batch);
        assert_eq!(mail.subject,"Fresh shipment status");
        assert_eq!(mail.extra_line.as_deref(),Some("The scanner is having a rough day."));
        assert!(mail.scam);
        assert_eq!(mail.second_line,generate(42,1,2,&[]).second_line);
        assert!(!batch[0].contains("bad@domain"));
        for day in 1..100 {assert!(compose(day,42).iter().all(|entry|valid_entry(entry)));}
    }

    #[test]
    fn first_shift_is_safe_and_late_inbox_varies_without_losing_clues() {
        assert!((0..count(1)).map(|i| generate(42, 1, i, &[])).map(|mail| mail.leak).sum::<f32>() < 60.0);
        assert_ne!(generate(42, 99, 0, &[]).subject, generate(42, 100, 0, &[]).subject);
        let fallback = generate(42, 100, 0, &[]);
        let live = generate(42, 100, 0, &["Fresh generated line".into()]);
        assert_eq!(fallback.payout, live.payout);
        assert_eq!(fallback.clue, live.clue);
        assert_eq!(live.first_line, "Fresh generated line");
    }
}
