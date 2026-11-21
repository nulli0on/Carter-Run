use std::{io::Read, process::{Command, Stdio}, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

static BUSY: AtomicBool = AtomicBool::new(false);

pub fn configured_model() -> Option<String> {
    let value = std::env::var("CARTEROS_AI_MODEL").unwrap_or_else(|_| "qwen3.5:0.8b".into());
    if value.eq_ignore_ascii_case("off") { return None; }
    (!value.is_empty() && value.len() <= 100 && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-:/".contains(&b))).then_some(value)
}

pub fn ask(model: &str, prompt: &str) -> Option<String> {
    if BUSY.swap(true, Ordering::AcqRel) { return None; }
    struct Release;
    impl Drop for Release {
        fn drop(&mut self) { BUSY.store(false, Ordering::Release); }
    }
    let _release = Release;
    let mut show = ollama();
    show.args(["show", model]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let mut check = show.spawn().ok()?;
    if !wait(&mut check, 4)?.success() { return None; }

    let mut command = ollama();
    command.args(["run", "--think=false", model, prompt])
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = command.spawn().ok()?;
    let stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut stdout = stdout;
        let mut buffer = [0; 4096];
        while let Ok(n) = stdout.read(&mut buffer) {
            if n == 0 { break; }
            kept.extend_from_slice(&buffer[..n.min(8192 - kept.len())]);
        }
        kept
    });
    let status = wait(&mut child, 55);
    let bytes = reader.join().ok()?;
    status.filter(|s| s.success())?;
    let output = String::from_utf8_lossy(&bytes).chars()
        .filter(|c| !c.is_control() || *c == '\n').take(2500).collect::<String>();
    (!output.trim().is_empty()).then_some(output)
}

fn ollama() -> Command {
    let mut command = Command::new("ollama");
    command.env("OLLAMA_HOST", "127.0.0.1:11434")
        .env("OLLAMA_NO_CLOUD", "1")
        .env("NO_PROXY", "127.0.0.1,localhost");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}

fn wait(child: &mut std::process::Child, seconds: u64) -> Option<std::process::ExitStatus> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if start.elapsed() < Duration::from_secs(seconds) => {
                std::thread::sleep(Duration::from_millis(50));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}
