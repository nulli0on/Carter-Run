use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{atomic::{AtomicU64, Ordering}, OnceLock},
};

static SAVE_DIR: OnceLock<PathBuf> = OnceLock::new();
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const LEGACY_DIR: &str = ".run_carter";

const LEGACY_FILES: &[&str] = &[
    "game_v1.cfg", "game_v1.bak", "game_v10.check", "carteros_settings.cfg",
    "achievements_v1.cfg", "cheat_state_v1.cfg", "installed_apps.cfg",
    "drive_v2.cfg", "virtual_files.txt", "window_layout.cfg",
];

pub fn set_install_directory() -> io::Result<()> {
    let exe = env::current_exe()?;
    let root = exe.ancestors().skip(1).take(3)
        .find(|dir| dir.join("assets/office_original_textures_complete.glb").is_file())
        .map_or_else(env::current_dir, |dir| Ok(dir.to_path_buf()))?;
    env::set_current_dir(root)
}

pub fn init() {
    let legacy = PathBuf::from(LEGACY_DIR);
    let directory = if let Some(override_dir) = absolute_env("RUN_CARTER_SAVE_DIR") {
        override_dir
    } else if let Some(directory) = user_directory() {
        match migrate_legacy(&legacy, &directory) {
            Ok(()) => directory,
            Err(err) => {
                eprintln!("Could not import the previous save folder: {err}. Using the original folder.");
                legacy
            }
        }
    } else {
        legacy
    };
    let _ = SAVE_DIR.set(directory);
}

fn absolute_env(name: &str) -> Option<PathBuf> {
    env::var_os(name).map(PathBuf::from).filter(|path| path.is_absolute())
}

fn user_directory() -> Option<PathBuf> {
    if cfg!(windows) {
        absolute_env("LOCALAPPDATA").map(|dir| dir.join("RUN CARTER"))
    } else {
        absolute_env("XDG_DATA_HOME").or_else(|| absolute_env("HOME").map(|home| home.join(".local/share")))
            .map(|dir| dir.join("run-carter"))
    }
}

pub fn path(name: &str) -> PathBuf {
    SAVE_DIR.get().map_or_else(|| Path::new(LEGACY_DIR), PathBuf::as_path).join(name)
}

fn migrate_legacy(legacy: &Path, destination: &Path) -> io::Result<()> {
    if !legacy.is_dir() || destination.exists() {
        return Ok(());
    }
    let parent = destination.parent().expect("user data directory has a parent");
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".run-carter-import-{}", std::process::id()));
    fs::create_dir(&temporary)?;
    let result = (|| {
        for name in LEGACY_FILES {
            match fs::copy(legacy.join(name), temporary.join(name)) {
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        match fs::rename(&temporary, destination) {
            Err(_) if destination.is_dir() => Ok(()),
            other => other,
        }
    })();
    let _ = fs::remove_dir_all(&temporary);
    result
}

pub fn replace(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = path.with_extension(format!("{}.{}.tmp", std::process::id(), sequence));
    let mut file = fs::File::create_new(&temporary)?;
    let result = file.write_all(text.as_bytes()).and_then(|()| file.sync_all());
    drop(file);
    let result = result.and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_import_copies_backup_and_settings_and_preserves_newer_data() {
        let root = env::temp_dir().join(format!("run-carter-migration-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join(LEGACY_DIR);
        let live = root.join("user/run-carter");
        replace(&legacy.join("game_v1.bak"), "backup").unwrap();
        replace(&legacy.join("carteros_settings.cfg"), "legacy").unwrap();
        migrate_legacy(&legacy, &live).unwrap();
        assert_eq!(fs::read_to_string(live.join("game_v1.bak")).unwrap(), "backup");
        assert_eq!(fs::read_to_string(live.join("carteros_settings.cfg")).unwrap(), "legacy");
        replace(&live.join("carteros_settings.cfg"), "current").unwrap();
        migrate_legacy(&legacy, &live).unwrap();
        assert_eq!(fs::read_to_string(live.join("carteros_settings.cfg")).unwrap(), "current");
        assert_eq!(fs::read_to_string(legacy.join("carteros_settings.cfg")).unwrap(), "legacy");
        fs::remove_dir_all(root).unwrap();
    }
}
