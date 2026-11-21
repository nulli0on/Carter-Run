use raylib::core::audio::{RaylibAudio, Sound};
use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

const BRIDGE: &str = "scripts/court_voice.py";

struct Job {
    child: Child,
    expires: Instant,
    lines: Receiver<String>,
}

impl Job {
    fn start(mut command: Command, timeout: u64) -> Result<Self, &'static str> {
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let mut child = command.spawn().map_err(|_| "Speech helper unavailable. Check the game setup guide.")?;
        let output = child.stdout.take().expect("speech stdout was piped");
        let (send, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output.take(1024)).lines().map_while(Result::ok) {
                if send.send(line).is_err() { break; }
            }
        });
        Ok(Self { child, expires: Instant::now() + Duration::from_secs(timeout), lines })
    }

    fn poll(&mut self) -> Option<Result<(), &'static str>> {
        match self.child.try_wait() {
            Ok(Some(status)) if status.success() => Some(Ok(())),
            Ok(Some(_)) => Some(Err("Speech unavailable. Check your microphone and local speech setup.")),
            Ok(None) if Instant::now() < self.expires => None,
            _ => Some(Err("Speech timed out. You can still type your statement.")),
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn python() -> Command {
    let exe = std::env::var_os("CARTEROS_SPEECH_PYTHON")
        .unwrap_or_else(|| if cfg!(windows) { "python".into() } else { "python3".into() });
    let mut command = Command::new(exe);
    command.arg(BRIDGE);
    command
}

#[derive(Default)]
pub struct CourtVoice<'audio> {
    capture: Option<Job>,
    clips: [Option<Sound<'audio>>; 5],
    played: Option<usize>,
    pub muted: bool,
    pub status: &'static str,
}

impl<'audio> CourtVoice<'audio> {
    pub fn new(audio: Option<&'audio RaylibAudio>) -> Self {
        let clips = ["clerk", "juror", "judge", "guilty", "dismissed"].map(|name| {
            audio.and_then(|device| device.new_sound(&format!("assets/sfx/court/{name}.wav")).ok())
        });
        Self { clips, ..Self::default() }
    }

    fn stop(&mut self) {
        if let Some(clip) = self.played.take().and_then(|id| self.clips[id].as_ref()) {
            clip.stop();
        }
    }

    pub fn reset(&mut self) {
        self.stop();
        self.capture = None;
        self.status = "";
    }

    pub fn busy(&self) -> bool { self.capture.is_some() }

    pub fn toggle_recording(&mut self) {
        if self.busy() {
            self.cancel_recording();
            return;
        }
        self.stop();
        let mut command = python();
        command.arg("listen");
        match Job::start(command, 30) {
            Ok(job) => {
                self.capture = Some(job);
                self.status = "Starting microphone. Wait for the recording indicator.";
            }
            Err(error) => self.status = error,
        }
    }

    pub fn cancel_recording(&mut self) {
        if self.capture.take().is_some() { self.status = "Recording cancelled."; }
    }

    pub fn transcript(&mut self) -> Option<String> {
        let job = self.capture.as_mut()?;
        loop {
            match job.lines.try_recv() {
                Ok(line) if line == "READY" => self.status = "Recording for seven seconds. F8 cancels.",
                Ok(line) if line.starts_with("DRAFT ") => {
                    self.capture = None;
                    let text: String = line[6..].chars()
                        .filter(|c| c.is_ascii() && !c.is_control()).take(145).collect();
                    if text.trim().is_empty() {
                        self.status = "No speech detected. Try again or type.";
                        return None;
                    }
                    self.status = "Check the transcript. Enter submits it.";
                    return Some(text);
                }
                Ok(_) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = match job.poll() {
                        Some(Err(error)) => error,
                        _ => "No transcript received. Try again or type.",
                    };
                    self.capture = None;
                    return None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    if let Some(Err(error)) = job.poll() {
                        self.status = error;
                        self.capture = None;
                    }
                    return None;
                }
            }
        }
    }

    pub fn toggle_narration(&mut self) {
        self.muted = !self.muted;
        self.stop();
    }

    pub fn narrate(&mut self, cue: usize, enabled: bool, volume: u8) {
        if !enabled || self.muted || self.busy() || volume == 0 {
            self.stop();
        } else if self.played != Some(cue) {
            self.stop();
            self.played = Some(cue);
            if let Some(clip) = &self.clips[cue] {
                clip.play();
            }
        }
    }
}
