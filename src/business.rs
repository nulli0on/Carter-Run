pub const REGISTRATION_REVENUE: i32 = 30_000;
pub const COUNSEL_FEE: i32 = 2_400;
const LEGACY_AUDIT_POINTS: u32 = 6;
const POWDER_RECEIPT: u8 = 1;
const RESTRICTED_RECEIPT: u8 = 2;
const MISSED_DEADLINE: u8 = 4;
const SUMMONS: u8 = 8;
pub const RULINGS: [&str; 2] = [
    "The record is upheld. The court orders the office sealed.",
    "The case is dismissed. Your tax balance remains due.",
];
pub const QUESTIONS: [&str; 3] = [
    "State the actual outstanding balance in Exhibit A and how you will settle it.",
    "How many receipts are actually verified in Exhibit B? Explain what they prove.",
    "The cover sheet makes three claims. Identify the one the exhibits contradict.",
];
const MONEY_CAP: i32 = 5_000_000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Business {
    pub gross: i32,
    pub due: i32,
    pub paid: i32,
    pub overdue: u32,
    pub sugar_orders: u32,
    pub restricted_orders: u32,
    pub evidence: u32,
    pub verdict: u8,
    pub storage_upgrade: bool,
    pub studio_upgrade: bool,
    pub overtime: bool,
    pub hearing_round: usize,
    pub hearing_score: u8,
    pub verified_cash: i32,
    pub counsel: bool,
    pub secret_open: bool,
    findings: u8,
    notice_day: u32,
    restricted_item: u8,
}

impl Business {
    pub fn registered(&self) -> bool { self.gross >= REGISTRATION_REVENUE }

    pub fn income(&mut self, amount: i32) {
        let before = self.gross;
        self.gross = self.gross.saturating_add(amount.max(0)).min(MONEY_CAP);
        let taxable = |gross: i32| (gross - REGISTRATION_REVENUE).max(0) * 15 / 100;
        let assessed = if before == MONEY_CAP { amount.max(0) * 15 / 100 }
            else { taxable(self.gross) - taxable(before) };
        self.due = self.due.saturating_add(assessed).min(MONEY_CAP);
    }

    pub fn pay(&mut self, cash: &mut i32) -> i32 {
        let amount = self.due.min(*cash);
        *cash -= amount;
        self.due -= amount;
        self.paid = self.paid.saturating_add(amount).min(MONEY_CAP);
        if self.due == 0 { self.overdue = 0; }
        amount
    }

    pub fn record_restricted_purchase(&mut self, catalog_index: usize) {
        self.restricted_orders = self.restricted_orders.saturating_add(1);
        if self.restricted_item == 0 {
            self.restricted_item = (catalog_index + 1) as u8;
        }
    }

    pub fn close_books(&mut self, day: u32) {
        if self.verdict != 0 { return; }
        if self.registered() && self.due >= 1_800 {
            self.overdue = self.overdue.saturating_add(1);
        }
        if day < 4 || self.findings & SUMMONS != 0 { return; }
        let purchase_flags = self.findings & (POWDER_RECEIPT | RESTRICTED_RECEIPT);
        let finding = if purchase_flags != 0 && self.findings.count_ones() >= 2 && day >= 6 {
            SUMMONS
        } else if self.sugar_orders > 0 && self.findings & POWDER_RECEIPT == 0 {
            POWDER_RECEIPT
        } else if self.restricted_orders > 0 && self.findings & RESTRICTED_RECEIPT == 0 {
            RESTRICTED_RECEIPT
        } else if self.overdue >= 3 && self.findings & MISSED_DEADLINE == 0 {
            MISSED_DEADLINE
        } else {
            return;
        };
        self.findings |= finding;
        self.notice_day = day.saturating_add(1);
    }

    pub fn may_open_secret(&self, day: u32) -> bool {
        self.verdict == 0 && (self.findings & SUMMONS != 0
            || (day >= 10 && self.evidence >= LEGACY_AUDIT_POINTS
                && self.sugar_orders > 0 && self.restricted_orders > 0))
    }

    pub fn has_notice(&self, day: u32) -> bool {
        self.verdict == 0 && !self.secret_open
            && (self.findings & SUMMONS != 0 || self.notice_day == day)
    }

    pub fn audit_notice(&self, day: u32) -> Option<crate::inbox::Mail> {
        if !self.has_notice(day) { return None; }
        let (subject, opening, detail, action) = if self.may_open_secret(day) {
            ("Summons to hearing 16-C", "The examiners finished assembling your case.",
                "Accept to attend. Dismiss this message to defer.",
                "To reopen it later, enter finanzamt://archiv in CarterWeb.")
        } else {
            ("Finanzamt opened an investigation", "Investigators matched these records to your office.",
                self.findings_text(), "They will review further evidence when you close a shift.")
        };
        let detail = if !self.may_open_secret(day) && self.findings & RESTRICTED_RECEIPT != 0 {
            self.restricted_name().map_or_else(|| detail.to_string(), |name| format!("Matched an invoice for {name}."))
        } else { detail.to_string() };
        Some(crate::inbox::Mail {
            sender: "clerk@finanzamt.example".into(), subject: subject.into(), first_line: opening.into(),
            second_line: action.into(), extra_line: Some(detail),
            clue: "Official in-game case notice. No payout or exposure.", payout: 0, leak: 0.0, scam: false,
        })
    }

    fn restricted_name(&self) -> Option<&'static str> {
        let index = usize::from(self.restricted_item.checked_sub(1)?);
        crate::arsenal::CATALOG.get(index).map(|entry| entry.name)
    }

    fn findings_text(&self) -> &'static str {
        match self.findings & 7 {
            1 => "Evidence: flagged powder receipt.",
            2 => "Evidence: restricted armory invoice.",
            3 => "Evidence: powder receipt and restricted armory invoice.",
            4 => "Evidence: missed filing deadline.",
            5 => "Evidence: powder receipt and missed filing deadline.",
            6 => "Evidence: armory invoice and missed filing deadline.",
            7 => "Evidence: powder, restricted armory invoice, and arrears.",
            _ => "Evidence: purchase records archived from an earlier case.",
        }
    }

    pub fn summoned(&self) -> bool { self.secret_open && self.verdict == 0 }

    pub fn debit_counsel(&mut self, cash: &mut i32) -> Result<(), &'static str> {
        if self.counsel { return Err("Counsel is already retained."); }
        if self.hearing_round > 0 { return Err("Counsel must be retained before your first statement."); }
        if self.verified_cash.min(*cash) < COUNSEL_FEE {
            return Err("Not enough documented earnings. You will represent yourself.");
        }
        *cash -= COUNSEL_FEE;
        self.verified_cash -= COUNSEL_FEE;
        self.counsel = true;
        Ok(())
    }

    pub fn encode(&self) -> String {
        format!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}", self.gross, self.due, self.paid, self.overdue,
            self.sugar_orders, self.restricted_orders, self.evidence, self.verdict,
            u8::from(self.storage_upgrade), u8::from(self.studio_upgrade), u8::from(self.overtime),
            self.hearing_round, self.hearing_score, self.verified_cash, u8::from(self.counsel), u8::from(self.secret_open), self.findings, self.notice_day, self.restricted_item)
    }

    pub fn decode(text: &str) -> Option<Self> {
        let fields = text.split(',').map(str::parse::<u32>).collect::<Result<Vec<_>, _>>().ok()?;
        let &[gross, due, paid, overdue, sugar_orders, restricted_orders, evidence, verdict, storage_upgrade, studio_upgrade, overtime, hearing_round, hearing_score, ref extra @ ..] = fields.as_slice() else { return None; };
        let (verified_cash, counsel, secret_open, findings, notice_day, restricted_item) = match extra {
            [] => (0, false, hearing_round > 0, 0, 0, 0), // v0.20 predates the documented funds ledger.
            [cash, flag] if *cash <= MONEY_CAP as u32 && *flag <= 1 => (*cash as i32, *flag == 1, hearing_round > 0, 0, 0, 0),
            [cash, flag, secret] if *cash <= MONEY_CAP as u32 && *flag <= 1 && *secret <= 1 => (*cash as i32, *flag == 1, *secret == 1, 0, 0, 0),
            [cash, flag, secret, findings, day, item] if *cash <= MONEY_CAP as u32 && *flag <= 1
                && *secret <= 1 && *findings <= 15 && *item <= crate::arsenal::COUNT as u32
                => (*cash as i32, *flag == 1, *secret == 1, *findings as u8, *day, *item as u8),
            _ => return None,
        };
        if [gross, due, paid].iter().any(|n| *n > MONEY_CAP as u32) || evidence > LEGACY_AUDIT_POINTS || verdict > 4 || storage_upgrade > 1 || studio_upgrade > 1 || overtime > 1 || hearing_round > 3 || hearing_score > hearing_round { return None; }
        Some(Self { gross: gross as i32, due: due as i32, paid: paid as i32, overdue, sugar_orders,
            restricted_orders, evidence, verdict: verdict as u8, storage_upgrade: storage_upgrade == 1, studio_upgrade: studio_upgrade == 1,
            overtime: overtime == 1, hearing_round: hearing_round as usize, hearing_score: hearing_score as u8, verified_cash, counsel, secret_open, findings, notice_day, restricted_item })
    }
}

pub struct Hearing {
    pub round: usize,
    pub input: String,
    pub notes: String,
    pub response: String,
    pub result: Option<bool>,
    pub showing_docket: bool,
    due: i32,
    receipts: u32,
    mismatch: Discrepancy,
    pub score: u8,
}

enum Discrepancy { Duplicate, Payment, Service }

fn case_numbers(statement:&str,words:&[String])->Vec<u32> {
    let mut numbers=statement.split(|c:char|!c.is_ascii_digit()&&c!=',')
        .filter_map(|n|n.replace(',',"").parse::<u32>().ok()).collect::<Vec<_>>();
    let small=["zero","one","two","three","four","five","six","seven","eight","nine","ten",
        "eleven","twelve","thirteen","fourteen","fifteen","sixteen","seventeen","eighteen","nineteen"];
    let tens=["twenty","thirty","forty","fifty","sixty","seventy","eighty","ninety"];
    let mut total=0u32;
    let mut group=0u32;
    let mut pending=false;
    for (index,word) in words.iter().enumerate() {
        let word=word.as_str();
        if let Some(value)=small.iter().position(|&n|n==word) {
            group=group.saturating_add(value as u32);
            pending=true;
        } else if let Some(value)=tens.iter().position(|&n|n==word) {
            group=group.saturating_add((value as u32+2)*10);
            pending=true;
        } else if word=="hundred" && pending {
            group=group.saturating_mul(100);
        } else if matches!(word,"thousand"|"million") && pending {
            total=total.saturating_add(group.saturating_mul(if word=="thousand" {1_000} else {1_000_000}));
            group=0;
        } else if word=="and" && (total>0 || group>=100)
            && words.get(index+1).is_some_and(|next|small.contains(&next.as_str())||tens.contains(&next.as_str())) {
            continue;
        } else if pending {
            numbers.push(total.saturating_add(group));
            (total,group,pending)=(0,0,false);
        }
    }
    if pending {numbers.push(total.saturating_add(group));}
    numbers
}

impl Hearing {
    pub fn new(seed: u64, business: &Business) -> Self {
        let mismatch = match crate::inbox::mix(seed ^ 0x5a14_9c03)%3 {
            0=>Discrepancy::Duplicate,1=>Discrepancy::Payment,_=>Discrepancy::Service,
        };
        let due=business.due;
        let receipts=business.sugar_orders.saturating_add(business.restricted_orders);
        let assessed=due+if matches!(mismatch,Discrepancy::Payment) {200} else {0};
        let listed=receipts.saturating_add(u32::from(matches!(mismatch,Discrepancy::Duplicate)));
        let served=if matches!(mismatch,Discrepancy::Service) {"16-B"} else {"16-C"};
        let item = business.restricted_name().map_or_else(String::new, |name| format!(" / {name}"));
        let notes=format!(
            "A / CASH BOOK
Open balance: ${due}. Overdue shifts: {}.
Assessment cover sheet claims ${assessed} outstanding.

B / PURCHASE RECORD
{} powder + {} restricted receipts = {receipts} verified receipts.
The case cover sheet lists {listed} purchases.

C / SERVICE LOG
Case on trial: 16-C. Case delivered to Carter: {served}.

{}",
            business.overdue,business.sugar_orders,business.restricted_orders,format_args!("{}{item}", business.findings_text()));
        Self { round:business.hearing_round.min(2),input:String::new(),notes,
            response:if business.hearing_round>0 {"Previous remarks are on record.".into()} else {String::new()},
            result:None,showing_docket:false,due,receipts,mismatch,score:business.hearing_score }
    }

    pub fn question(&self) -> &'static str {
        QUESTIONS[self.round.min(2)]
    }

    pub fn counsel_hint(&self) -> &'static str {
        match self.round {
            0 => "Counsel: State the exact amount in Exhibit A and say you will pay it.",
            1 => "Counsel: Add both categories in Exhibit B. Say how many receipts are verified.",
            _ => match self.mismatch {
                Discrepancy::Duplicate => "Counsel: Read both receipt counts in Exhibit B.",
                Discrepancy::Payment => "Counsel: Read both outstanding balances in Exhibit A.",
                Discrepancy::Service => "Counsel: Read both case numbers in Exhibit C.",
            },
        }
    }

    pub fn submit(&mut self) -> Option<(String, &'static str, &'static str)> {
        let statement = self.input.trim();
        if statement.is_empty() || self.result.is_some() { return None; }
        let question = self.question();
        let words = statement.split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|word| !word.is_empty()).map(str::to_ascii_lowercase).collect::<Vec<_>>();
        let includes = |terms: &[&str]| words.iter().any(|word| terms.contains(&word.as_str()));
        let numbers=case_numbers(statement,&words);
        let number=|n:u32|numbers.contains(&n);
        let contradictory_pair=match self.mismatch {
            Discrepancy::Duplicate=>includes(&["receipt","receipts","purchases","purchase"]) &&
                number(self.receipts) && number(self.receipts.saturating_add(1)),
            Discrepancy::Payment=>includes(&["balance","outstanding","assessment"]) &&
                number(self.due as u32) && number(self.due as u32+200),
            Discrepancy::Service=>{
                let file = statement.to_ascii_lowercase().replace(['-',' '],"");
                includes(&["service","served","delivered"]) &&
                    (file.contains("16b") || file.contains("sixteenb") || file.contains("sixteenbee")) &&
                    (file.contains("16c") || file.contains("sixteenc") || file.contains("sixteensee"))
            }
        };
        let (accepted,feedback)=match self.round {
            0=>(number(self.due as u32) && includes(&["pay","paid","payment","settle","cleared"]),
                "Your payment statement matches the cash book."),
            1=>(number(self.receipts) && includes(&["verified","valid","prove","documented","confirmed"]) &&
                includes(&["receipt","receipts","purchases","orders"]),
                "The panel records the verified receipt count."),
            _=>(contradictory_pair,"The objection identifies a contradiction in the record."),
        };
        if accepted { self.score += 1; }
        self.response = if accepted { feedback.to_owned() } else {
            match self.round {
                0 if includes(&["paid", "pay", "tax", "balance"]) => "The panel asks that future remarks address both the amount and payment.",
                0 => "The clerk could not reconcile your statement with Exhibit A.",
                1 if includes(&["receipt", "receipts", "invoice", "records"]) => "The examiner has marked this as a general comment on the receipts.",
                1 => "The examiner has entered a statement without a reference to the purchase record.",
                _ => "The certificate stands as filed.",
            }.to_owned()
        };
        self.round += 1;
        if self.round == 3 { self.result = Some(self.score == 3); }
        Some((std::mem::take(&mut self.input), question, feedback))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn court_scores_a_spoken_balance_and_receipt_count() {
        let book=Business {due:2150,sugar_orders:2,restricted_orders:4,..Business::default()};
        let seed=(0..30).find(|&seed|matches!(Hearing::new(seed,&book).mismatch,Discrepancy::Payment)).unwrap();
        let mut hearing=Hearing::new(seed,&book);
        for reply in [
            "I will pay two thousand one hundred and fifty dollars",
            "There are six verified receipts in the record",
            "The balance is two thousand one hundred fifty, but the assessment says two thousand three hundred fifty",
        ] {
            hearing.input=reply.into();
            hearing.submit().unwrap();
        }
        assert_eq!(hearing.result,Some(true));
    }

    #[test]
    fn paying_tax_prevents_arrears_but_does_not_destroy_receipts() {
        let mut book = Business::default();
        book.income(29_900);
        assert!(!book.registered());
        book.income(500);
        assert_eq!(book.due, 60);
        let mut cash = 200;
        assert_eq!(book.pay(&mut cash), 60);
        for day in 1..=8 { book.close_books(day); }
        assert!(!book.may_open_secret(9));
        book.restricted_orders = 1;
        book.sugar_orders = 1;
        book.close_books(9);
        assert!(book.audit_notice(10).unwrap().extra_line.unwrap().contains("powder"));
        assert!(!book.may_open_secret(10));
        book.close_books(10);
        assert!(!book.may_open_secret(11));
        book = Business::decode(&book.encode()).unwrap();
        book.close_books(11);
        assert!(!book.summoned());
        assert!(book.may_open_secret(12));
        book.secret_open = true;
        assert!(book.summoned());
        assert_eq!(Business::decode(&book.encode()), Some(book));
    }

    #[test]
    fn counsel_requires_available_documented_money_and_only_charges_once() {
        let mut book = Business { verified_cash: COUNSEL_FEE - 1, ..Business::default() };
        let mut cash = 100_000;
        assert!(book.debit_counsel(&mut cash).is_err());
        book.verified_cash = COUNSEL_FEE + 60;
        assert!(book.debit_counsel(&mut cash).is_ok());
        assert_eq!((cash, book.verified_cash), (100_000 - COUNSEL_FEE, 60));
        assert!(book.debit_counsel(&mut cash).is_err());
        assert_eq!(Business::decode(&book.encode()), Some(book));
        assert!(Business::decode("0,0,0,0,0,0,0,0,0,0,0,0,0").is_some());
    }

    #[test]
    fn exhibits_contain_a_case_specific_contradiction() {
        let book=Business {due:2150,sugar_orders:2,restricted_orders:4,..Business::default()};
        for seed in 0..12 {
            let mut hearing=Hearing::new(seed,&book);
            let answer=match hearing.mismatch {
                Discrepancy::Duplicate=>"The record verifies 6 receipts, but the cover sheet says 7.",
                Discrepancy::Payment=>"The balance is $2,150, but the assessment claims $2,350.",
                Discrepancy::Service=>"The service log says they delivered 16-B, but this is case 16-C.",
            };
            for reply in ["I will pay $2,150.","There are 6 verified receipts.",answer] {
                hearing.input=reply.into();
                hearing.submit().unwrap();
            }
            assert_eq!(hearing.result,Some(true));
        }
        let mut hearing=Hearing::new(4,&book);
        for reply in ["I will pay the balance.","Check the purchase receipts.","The certificate is a draft."] {
            hearing.input=reply.into();
            hearing.submit().unwrap();
        }
        assert_eq!(hearing.result,Some(false));
    }
}
