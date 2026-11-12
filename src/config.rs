use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::theme::Theme;
use crate::triggers::{BindingTarget, Trigger};

const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConfig {
    pub version: u32,
    pub bindings: Vec<(BindingTarget, Trigger)>,
    pub cycle_order: Vec<String>,
    pub lang: Lang,
    pub theme: Theme,
    /// Title-matching regex used by the watcher to identify Dofus windows.
    /// `None` (or missing in older JSONs) → fall back to
    /// `crate::win::DEFAULT_TITLE_REGEX`. Lets users adapt to future Dofus
    /// title format changes without rebuilding.
    #[serde(default)]
    pub title_regex: Option<String>,
    /// "–" and "Activer" minimize to the Windows taskbar instead of hiding
    /// in the system tray. Missing in older JSONs → false (tray, as before).
    #[serde(default)]
    pub minimize_to_taskbar: bool,
    /// Shortcuts only fire while a Dofus window is in front. Missing in older
    /// JSONs → true: the safer behaviour applies to existing users too.
    #[serde(default = "default_true")]
    pub shortcuts_only_in_dofus: bool,
    /// Keep the window above every other one, Dofus included. Missing in
    /// older JSONs → false: a normal window, also for existing users.
    #[serde(default)]
    pub always_on_top: bool,
    /// NPC travel: everyone teleports and follows the first account (Ctrl+W),
    /// the only one to walk. Off → each account walks by itself.
    #[serde(default)]
    pub npc_follow_leader: bool,
    /// Pause before the leader's follow shortcut, in ms (0.5–3 s).
    #[serde(default = "default_follow_delay")]
    pub npc_follow_delay_ms: u32,
    /// Keys of the game ROrganizer presses itself (open chat, follow the
    /// leader). Missing → the game's defaults.
    #[serde(default)]
    pub game_keys: crate::game_keys::GameKeys,
    /// Account that invites, walks and is followed; missing = first of the list.
    #[serde(default)]
    pub group_leader: Option<String>,
    /// Starred zaaps, shown first in the picker (`favorites`).
    #[serde(default)]
    pub favorite_zaaps: Vec<String>,
    /// Starred NPCs, shown first in the picker (`favorites`).
    #[serde(default)]
    pub favorite_npcs: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_follow_delay() -> u32 {
    crate::npcs::FOLLOW_DELAY_DEFAULT_MS
}

/// `%APPDATA%\rorganizer\config.json` on Windows. Returns None on platforms
/// where no usable config dir is reported (extremely rare on Windows).
pub fn default_path() -> Option<PathBuf> {
    use directories::BaseDirs;
    let base = BaseDirs::new()?;
    Some(base.config_dir().join("rorganizer").join("config.json"))
}

/// Reads + parses the config file. Any failure (missing, corrupted, version
/// mismatch) returns None — the caller falls back to defaults silently.
pub fn load(path: &Path) -> Option<StoredConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    let cfg: StoredConfig = serde_json::from_str(&text).ok()?;
    if cfg.version != CURRENT_VERSION {
        return None;
    }
    Some(cfg)
}

/// Sibling path: `<file>.tmp` next to `path`, in the same directory so the
/// final `rename` stays on a single filesystem (atomic on Windows + Linux).
fn tmp_path(path: &Path) -> std::path::PathBuf {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config".into());
    path.with_file_name(format!("{name}.tmp"))
}

/// Writes the config to a sibling `.tmp` then renames to the final path so a
/// crash mid-write never corrupts the existing file.
///
/// Calls `sync_all()` on the tmp before the rename to flush the file's
/// data + metadata to the disk controller; without it, a power loss
/// between write and rename can leave a zero-byte tmp on disk and an
/// older valid `config.json` — annoying but recoverable. With it, the
/// final state on disk is either "old config" or "new config", never
/// half-written.
pub fn save_atomic(path: &Path, cfg: &StoredConfig) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = tmp_path(path);
    let text = serde_json::to_string_pretty(cfg).map_err(std::io::Error::other)?;
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(tmp, path)?;
    Ok(())
}

pub const fn current_version() -> u32 {
    CURRENT_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::{BindingTarget, MouseBtn, Trigger, WheelDir};
    use tempfile::tempdir;

    fn sample_config() -> StoredConfig {
        StoredConfig {
            version: CURRENT_VERSION,
            bindings: vec![
                (BindingTarget::Account("Dowoop".into()), Trigger::Key(0x70)),
                (BindingTarget::CycleNext, Trigger::Mouse(MouseBtn::X1)),
                (BindingTarget::CyclePrev, Trigger::Wheel(WheelDir::Down)),
            ],
            cycle_order: vec!["Dowoop".into(), "Dowoip".into()],
            lang: Lang::Fr,
            theme: Theme::Dark,
            title_regex: None,
            minimize_to_taskbar: true,
            shortcuts_only_in_dofus: false,
            always_on_top: true,
            npc_follow_leader: true,
            npc_follow_delay_ms: 1200,
            game_keys: crate::game_keys::GameKeys::default(),
            group_leader: Some("Dowoip".into()),
            favorite_zaaps: vec!["Port de Madrestam [7,-4]".into()],
            favorite_npcs: vec!["Ganymède · Ganymède [1,2]".into()],
        }
    }

    #[test]
    fn config_without_always_on_top_field_is_a_normal_window() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut json = serde_json::to_value(sample_config()).unwrap();
        json.as_object_mut().unwrap().remove("always_on_top");
        std::fs::write(&path, json.to_string()).unwrap();
        assert!(!load(&path).expect("old config loads").always_on_top);
    }

    #[test]
    fn config_without_dofus_only_field_defaults_to_on() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut json = serde_json::to_value(sample_config()).unwrap();
        json.as_object_mut().unwrap().remove("shortcuts_only_in_dofus");
        std::fs::write(&path, json.to_string()).unwrap();
        assert!(load(&path).expect("old config loads").shortcuts_only_in_dofus);
    }

    #[test]
    fn config_without_minimize_field_defaults_to_tray() {
        // A config.json written before the option existed must still load.
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut json = serde_json::to_value(sample_config()).unwrap();
        json.as_object_mut().unwrap().remove("minimize_to_taskbar");
        std::fs::write(&path, json.to_string()).unwrap();
        assert!(!load(&path).expect("old config loads").minimize_to_taskbar);
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let cfg = sample_config();
        save_atomic(&path, &cfg).unwrap();
        let loaded = load(&path).expect("load should succeed");
        assert_eq!(loaded.version, cfg.version);
        assert_eq!(loaded.bindings, cfg.bindings);
        assert_eq!(loaded.cycle_order, cfg.cycle_order);
        assert_eq!(loaded.lang, cfg.lang);
        assert_eq!(loaded.theme, cfg.theme);
        assert_eq!(loaded.minimize_to_taskbar, cfg.minimize_to_taskbar);
        assert_eq!(loaded.shortcuts_only_in_dofus, cfg.shortcuts_only_in_dofus);
        assert_eq!(loaded.always_on_top, cfg.always_on_top);
        assert_eq!(loaded.npc_follow_leader, cfg.npc_follow_leader);
        assert_eq!(loaded.npc_follow_delay_ms, cfg.npc_follow_delay_ms);
        assert_eq!(loaded.group_leader, cfg.group_leader);
        assert_eq!(loaded.favorite_zaaps, cfg.favorite_zaaps);
        assert_eq!(loaded.favorite_npcs, cfg.favorite_npcs);
    }

    #[test]
    fn load_returns_none_on_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nope.json");
        assert!(load(&path).is_none());
    }

    #[test]
    fn load_returns_none_on_corrupt_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "not json {{{").unwrap();
        assert!(load(&path).is_none());
    }

    #[test]
    fn load_returns_none_on_version_mismatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut cfg = sample_config();
        cfg.version = CURRENT_VERSION + 1;
        save_atomic(&path, &cfg).unwrap();
        assert!(load(&path).is_none());
    }

    #[test]
    fn save_atomic_does_not_leave_tmp_on_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        save_atomic(&path, &sample_config()).unwrap();
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entries, vec!["config.json"], "no .tmp leftover, got {entries:?}");
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("deeper").join("config.json");
        save_atomic(&path, &sample_config()).unwrap();
        assert!(path.exists());
    }
}
