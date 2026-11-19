//! In-game debug console parser. Does not execute shell commands or inspect host memory.
#[derive(Debug,Clone,PartialEq)]
pub enum Command {
    Help, Status, DebugOn, Cash(i32), Countdown(f32), SugarSpawn,
    AiBullshit, AiNormal, Kids(u8), Teleport, ClearAggro, Unknown,
}

pub fn parse(s:&str)->Command {
    let s=s.trim().to_ascii_lowercase();
    let mut words=s.split_whitespace();
    match (words.next(),words.next(),words.next()) {
        (Some("help"),None,None)=>Command::Help,
        (Some("status"),None,None)=>Command::Status,
        (Some("debug"),Some("on"),None)=>Command::DebugOn,
        (Some("cash"),Some(n),None)=>n.parse::<i32>().ok().filter(|n|*n>=0 && *n<=1_000_000).map(Command::Cash).unwrap_or(Command::Unknown),
        (Some("timer"),Some(n),None)=>n.parse::<f32>().ok().filter(|n|n.is_finite()&&*n>=0.1&&*n<=75.0).map(Command::Countdown).unwrap_or(Command::Unknown),
        (Some("sugar"),Some("spawn"),None)=>Command::SugarSpawn,
        (Some("ai"),Some("bullshit"),None)=>Command::AiBullshit,
        (Some("ai"),Some("normal"),None)=>Command::AiNormal,
        (Some("kids"),Some(n),None)=>n.parse::<u8>().ok().filter(|n|*n>0&&*n<=12).map(Command::Kids).unwrap_or(Command::Unknown),
        (Some("tp"),Some("desk"),None)=>Command::Teleport,
        (Some("clear"),Some("aggro"),None)=>Command::ClearAggro,
        _=>Command::Unknown,
    }
}

#[derive(Default)]
pub struct Console {pub open:bool,pub buffer:String,pub history:Vec<String>}
impl Console {
    pub fn log(&mut self,s:&str){self.history.push(s.to_owned());if self.history.len()>9{self.history.remove(0);}}
    pub fn write(&mut self,chars:&str){for c in chars.chars(){if !c.is_control() && self.buffer.len()<90{self.buffer.push(c);}}}
    pub fn submit(&mut self)->Command {
        let input=std::mem::take(&mut self.buffer);
        self.log(&format!("> {}",input));parse(&input)
    }
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn no_shell_and_bounded_values(){assert_eq!(parse("debug on"),Command::DebugOn);assert_eq!(parse("cash 1000"),Command::Cash(1000));assert_eq!(parse("cash -1"),Command::Unknown);assert_eq!(parse("rm -rf /"),Command::Unknown);assert_eq!(parse("timer NaN"),Command::Unknown);}
    #[test]fn history_is_limited(){let mut c=Console::default();for _ in 0..30{c.log("x");}assert_eq!(c.history.len(),9);}
}
