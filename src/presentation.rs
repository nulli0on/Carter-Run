use raylib::prelude::*;
use crate::{business::Hearing, system_state::SystemState, Game};

pub fn fit_canvas(width: i32, height: i32) -> Rectangle {
    let (w, h) = (crate::carteros::W as f32, crate::carteros::H as f32);
    let scale = (width as f32 / w).min(height as f32 / h);
    let (w, h) = (w * scale, h * scale);
    Rectangle::new((width as f32 - w) / 2.0, (height as f32 - h) / 2.0, w, h)
}

pub fn canvas_point(mouse: Vector2, viewport: Rectangle) -> Vector2 {
    Vector2::new((mouse.x - viewport.x) * crate::carteros::W as f32 / viewport.width.max(1.0),
        (mouse.y - viewport.y) * crate::carteros::H as f32 / viewport.height.max(1.0))
}

pub fn present_canvas(d: &mut impl RaylibDraw, texture: impl AsRef<raylib::ffi::Texture2D>, viewport: Rectangle) {
    d.clear_background(BACK);
    d.draw_texture_pro(texture, Rectangle::new(0.0, 0.0, crate::carteros::W as f32, -crate::carteros::H as f32),
        viewport, Vector2::zero(), 0.0, Color::WHITE);
}

const BACK: Color = Color::new(12, 14, 25, 255);
const PANEL: Color = Color::new(25, 28, 43, 255);
const PAPER: Color = Color::new(242, 236, 218, 255);
const TEXT: Color = Color::new(247, 244, 237, 255);
const QUIET: Color = Color::new(170, 180, 193, 255);
const TEAL: Color = Color::new(118, 228, 195, 255);
const PINK: Color = Color::new(251, 152, 155, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Menu {
    Title,
    ConfirmNew,
    Difficulty,
    Pause,
    Settings,
    QuitUnsaved,
}

impl Menu {
    pub fn rows(self, saved: bool) -> &'static [&'static str] {
        match self {
            Self::Title if saved => &["Continue", "New run", "Settings", "Quit"],
            Self::Title => &["New run", "Settings", "Quit"],
            Self::ConfirmNew => &["Keep current run", "Replace with new run"],
            Self::Difficulty => &["Standard", "Overtime", "Back to title"],
            Self::Pause => &["Resume", "Settings", "Save and quit"],
            Self::Settings => &["Mouse sensitivity", "Invert vertical mouse", "Master volume", "Back"],
            Self::QuitUnsaved => &["Keep playing", "Quit to last checkpoint"],
        }
    }
}

pub fn menu_row(y: usize) -> i32 { 342 + y as i32 * 62 }

pub fn draw_menu(d: &mut impl RaylibDraw, menu: Menu, selected: usize, saved: bool, game: &Game,
    settings: &SystemState, settings_error: Option<&str>) {
    let (width, height) = (crate::carteros::W, crate::carteros::H);
    let x = (width - 660) / 2;
    let label = match menu {
        Menu::Title => "RUN CARTER",
        Menu::Pause => "SHIFT PAUSED",
        Menu::ConfirmNew => "START OVER?",
        Menu::Difficulty => "CHOOSE YOUR SHIFT",
        Menu::Settings => "SETTINGS",
        Menu::QuitUnsaved => "CAN'T SAVE NOW",
    };
    d.clear_background(BACK);
    for offset in (0..height).step_by(6) {
        d.draw_line(0, offset, width, offset, Color::new(21, 24, 39, 255));
    }
    d.draw_rectangle(x - 28, 100, 716, 505, BACK);
    d.draw_rectangle(x, 118, 660, 5, TEAL);
    d.draw_text(label, x, 155, 60, TEXT);
    let sub = match menu {
        Menu::Title if saved => format!("ENDLESS OFFICE   /   LAST CHECKPOINT AVAILABLE"),
        Menu::Title => "ENDLESS OFFICE".to_owned(),
        Menu::Pause => format!("DAY {}   /   ${}   /   {:.0}% EXPOSURE", game.day, game.money, game.exposure),
        Menu::ConfirmNew => "Starting a new run replaces the current checkpoint.".to_owned(),
        Menu::Difficulty => "Overtime brings faster fans and earlier reinforcements.".to_owned(),
        Menu::Settings => "Use left and right to change a setting.".to_owned(),
        Menu::QuitUnsaved => game.message.clone(),
    };
    for (i, line) in wrap(&sub, 62).iter().take(3).enumerate() {
        d.draw_text(line, x, 242 + i as i32 * 23, 19, QUIET);
    }
    for (i, row) in menu.rows(saved).iter().enumerate() {
        let y = menu_row(i);
        let on = i == selected;
        d.draw_rectangle(x, y, 660, 48, if on { TEAL } else { PANEL });
        let label = match (menu, i) {
            (Menu::Settings, 0) => format!("{row}   {}%", settings.mouse_sensitivity),
            (Menu::Settings, 1) => format!("{row}   {}", if settings.invert_mouse { "On" } else { "Off" }),
            (Menu::Settings, 2) => format!("{row}   {}%", settings.master_volume),
            _ => (*row).to_owned(),
        };
        d.draw_text(&label, x + 20, y + 12, 24, if on { BACK } else { TEXT });
    }
    if menu == Menu::Title && game.achievements.best_shift > 0 {
        d.draw_text(&format!("Best shift  {}", game.achievements.best_shift), x + 439, 260, 19, TEAL);
    }
    d.draw_text("Arrow keys / Enter     F11  Fullscreen", x, 625, 17, QUIET);
    if menu == Menu::Settings {
        if let Some(error) = settings_error {
            for (i, line) in wrap(error, 65).iter().take(3).enumerate() {
                d.draw_text(line, x, 660 + i as i32 * 21, 17, PINK);
            }
        }
    }
    let footer = format!("RUN CARTER   /   v{}", env!("CARGO_PKG_VERSION"));
    d.draw_text(&footer, x, height - 36, 15, QUIET);
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut output = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        let mut used = 0;
        for word in paragraph.split_whitespace() {
            let length = word.chars().count();
            if used > 0 && used + 1 + length > width {
                output.push(std::mem::take(&mut line));
                used = 0;
            }
            if used > 0 {
                line.push(' ');
                used += 1;
            }
            for character in word.chars() {
                if used == width {
                    output.push(std::mem::take(&mut line));
                    used = 0;
                }
                line.push(character);
                used += 1;
            }
        }
        output.push(line);
    }
    output
}

pub fn draw_hearing(d: &mut impl RaylibDraw, game: &Game, hearing: &Hearing, voice: &crate::court_voice::CourtVoice<'_>) {
    let (width, height) = (crate::carteros::W, crate::carteros::H);
    let x = (width - 1010) / 2;
    d.clear_background(BACK);
    d.draw_rectangle(x, 73, 1010, 4, PINK);
    d.draw_text("FINANZAMT", x, 98, 62, TEXT);
    d.draw_text("OFFICE OF UNREASONABLY SPECIFIC PAPERWORK", x, 177, 20, QUIET);
    if let Some(won) = hearing.result {
        d.draw_rectangle(x, 235, 1010, 280, PANEL);
        d.draw_text(if won { "CASE DISMISSED" } else { "GUILTY" }, x + 30, 267, 56, if won { TEAL } else { PINK });
        let detail = crate::business::RULINGS[usize::from(won)];
        d.draw_text(detail, x + 30, 351, 21, TEXT);
        d.draw_text(&format!("Day {}   /   ${} in business revenue", game.day, game.business.gross), x + 30, 406, 24, QUIET);
        if won && game.achievements.has("TAX_EVADER") {
            d.draw_text("ACHIEVEMENT   /   TAX EVADER", x + 30, 453, 23, TEAL);
        }
        d.draw_text(if won { "Enter  Return to the office" } else { "Enter  Face the Finanzamt" }, x, 555, 22, TEXT);
    } else if hearing.showing_docket {
        d.draw_rectangle(x, 235, 1010, 362, PAPER);
        for (i, line) in hearing.notes.lines().enumerate() {
            d.draw_text(line, x + 34, 256 + i as i32 * 28, 24, BACK);
        }
        d.draw_text("Tab  Back to the hearing", x, 616, 19, QUIET);
    } else {
        d.draw_rectangle(x, 235, 1010, 247, PANEL);
        let members = ["CLERK", "JUROR", "JUDGE"];
        d.draw_text(&format!("{}   /   STATEMENT {} OF 3", members[hearing.round], hearing.round + 1), x + 30, 257, 23, PINK);
        for (i, line) in wrap(hearing.question(), 70).iter().enumerate() {
            d.draw_text(line, x + 30, 306 + i as i32 * 27, 24, TEXT);
        }
        for (i, line) in wrap(&hearing.response, 79).iter().take(3).enumerate() {
            d.draw_text(line, x + 30, 377 + i as i32 * 24, 20, QUIET);
        }
        d.draw_rectangle(x, 503, 1010, 78, Color::new(33, 37, 54, 255));
        let input_lines = wrap(&format!("{}_", hearing.input), 78);
        for (i, line) in input_lines.iter().skip(input_lines.len().saturating_sub(2)).enumerate() {
            d.draw_text(line, x + 24, 519 + i as i32 * 24, 21, TEXT);
        }
        d.draw_text("Enter  Submit   /   Tab  Exhibits   /   F8  Microphone   /   F2  Voices   /   Esc  Title", x, 600, 18, QUIET);
        if !voice.status.is_empty() { d.draw_text(voice.status, x, 630, 17, if voice.busy() { PINK } else { TEAL }); }
        if game.business.counsel {
            d.draw_text(hearing.counsel_hint(), x, 662, 17, TEXT);
        } else if hearing.round == 0 {
            let balance = format!("F4  Retain counsel  ${}   /   ${} documented funds", crate::business::COUNSEL_FEE, game.business.verified_cash);
            d.draw_text(&balance, x, 662, 17, QUIET);
        }
    }
    d.draw_text("FICTIONAL ADMINISTRATIVE HEARING", x, height - 39, 15, QUIET);
}
