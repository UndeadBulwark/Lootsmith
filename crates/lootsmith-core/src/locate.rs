//! Locate Borderlands 2 save files on this machine.
//!
//! Covers the places BL2 actually writes saves on Linux — the native Aspyr port,
//! Steam/Proton prefixes (appid 49520), Flatpak Steam, and generic Wine/Lutris
//! prefixes (including custom ones like a ZLuda prefix under `$HOME`) — plus the
//! usual Windows and macOS locations. Matching is case-insensitive because Wine
//! creates both `Documents` and `My Documents BACKUP` and the game writes
//! `My games`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A discovered save file.
#[derive(Clone, Debug)]
pub struct SaveCandidate {
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
}

impl SaveCandidate {
    /// The `<steamid64>` folder the save lives in, when it is numeric.
    pub fn steam_id(&self) -> Option<String> {
        let parent = self
            .path
            .parent()?
            .file_name()?
            .to_string_lossy()
            .to_string();
        if parent.chars().all(|c| c.is_ascii_digit()) && !parent.is_empty() {
            Some(parent)
        } else {
            None
        }
    }

    /// A `YYYY-MM-DD HH:MM` (UTC) string, or `?` when unknown.
    pub fn modified_text(&self) -> String {
        match self.modified {
            Some(t) => format_time(t),
            None => "?".to_string(),
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn eq_ci(name: &std::ffi::OsStr, target: &str) -> bool {
    name.to_string_lossy().eq_ignore_ascii_case(target)
}

/// Directories to begin searching from (each is either a `SaveData` dir or a
/// directory that contains one somewhere beneath it).
fn default_starts() -> Vec<PathBuf> {
    let mut starts = Vec::new();
    let Some(home) = home_dir() else {
        return starts;
    };

    // Native Aspyr Linux port.
    starts.push(home.join(".local/share/aspyr-media/borderlands 2/willowgame/savedata"));

    // Steam / Proton (appid 49520) across the common Steam roots.
    let steam_roots = [
        home.join(".steam/steam"),
        home.join(".steam/root"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ];
    for root in steam_roots {
        starts.push(root.join("steamapps/compatdata/49520/pfx/drive_c/users"));
    }

    // Windows.
    if let Some(profile) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        starts.push(profile.join("Documents/My Games/Borderlands 2/WillowGame/SaveData"));
    } else {
        starts.push(home.join("Documents/My Games/Borderlands 2/WillowGame/SaveData"));
    }

    // macOS (rare).
    starts.push(home.join("Library/Application Support/Borderlands 2/SaveData"));

    // Generic Wine / Lutris / custom prefixes under $HOME.
    starts.push(home.join(".wine/drive_c/users"));
    starts.push(home.join("Games"));
    starts.extend(extra_prefix_user_dirs(&home));

    // Explicit roots from the environment (colon-separated).
    if let Ok(extra) = std::env::var("LOOTSMITH_SAVE_ROOTS") {
        for part in extra.split(':') {
            let part = part.trim();
            if !part.is_empty() {
                starts.push(PathBuf::from(part));
            }
        }
    }
    if let Ok(explicit) = std::env::var("LOOTSMITH_SAVES") {
        let explicit = explicit.trim();
        if !explicit.is_empty() {
            starts.push(PathBuf::from(explicit));
        }
    }

    starts
}

/// Wine prefixes that are one or two levels under `$HOME` (covers
/// `~/bl2-zluda/prefix-baseline/drive_c/users`, `~/<game>/drive_c/users`, ...).
fn extra_prefix_user_dirs(home: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(level1) = std::fs::read_dir(home) else {
        return out;
    };
    for (i, entry) in level1.flatten().enumerate() {
        if i > 1000 {
            break;
        }
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        // Skip hidden dirs (handled explicitly) except known prefixes.
        if name.starts_with('.') {
            continue;
        }
        if path.join("drive_c/users").is_dir() {
            out.push(path.join("drive_c/users"));
        }
        let Ok(level2) = std::fs::read_dir(&path) else {
            continue;
        };
        for (j, sub) in level2.flatten().enumerate() {
            if j > 500 {
                break;
            }
            let users = sub.path().join("drive_c/users");
            if users.is_dir() {
                out.push(users);
            }
        }
    }
    out
}

/// Walk `start` (bounded) and collect every `*.sav` under a `savedata` directory
/// whose path mentions `borderlands`.
fn collect_from(start: &Path, out: &mut Vec<SaveCandidate>) {
    if !start.exists() {
        return;
    }
    // The start itself may be a SaveData directory.
    if start
        .file_name()
        .map(|n| eq_ci(n, "savedata"))
        .unwrap_or(false)
    {
        collect_saves(start, out);
        return;
    }
    let mut stack = vec![(start.to_path_buf(), 0usize)];
    let mut budget = 40_000usize;
    while let Some((dir, depth)) = stack.pop() {
        if depth > 10 || budget == 0 {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            budget = budget.saturating_sub(1);
            if budget == 0 {
                break;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name == "savedata" {
                if path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains("borderlands")
                {
                    collect_saves(&path, out);
                }
                continue;
            }
            if matches!(name.as_str(), "node_modules" | "target" | ".git" | ".cache") {
                continue;
            }
            stack.push((path, depth + 1));
        }
    }
}

/// Collect `*.sav` files directly in `savedata` or one level below it.
fn collect_saves(savedata: &Path, out: &mut Vec<SaveCandidate>) {
    let mut stack = vec![(savedata.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > 2 {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                stack.push((path, depth + 1));
            } else if path
                .extension()
                .map(|e| e.eq_ignore_ascii_case("sav"))
                .unwrap_or(false)
            {
                let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
                out.push(SaveCandidate { path, modified });
            }
        }
    }
}

fn candidate_key(cand: &SaveCandidate) -> (Option<String>, String) {
    let filename = cand
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    (cand.steam_id(), filename)
}

fn path_is_backup(path: &Path) -> bool {
    path.to_string_lossy().to_lowercase().contains("backup")
}

/// Drop a save only when an identical non-backup twin exists (Wine writes both
/// `Documents` and `My Documents BACKUP`); then sort newest-first.
fn dedupe_and_sort(found: &mut Vec<SaveCandidate>) {
    // Collapse the same physical file reached through symlinked Steam roots.
    let mut seen = std::collections::HashSet::new();
    found.retain(|c| {
        let key = std::fs::canonicalize(&c.path).unwrap_or_else(|_| c.path.clone());
        seen.insert(key)
    });

    let keys: Vec<(Option<String>, String)> = found.iter().map(candidate_key).collect();
    let mut drop = vec![false; found.len()];
    for i in 0..found.len() {
        if !path_is_backup(&found[i].path) {
            continue;
        }
        for (j, key) in keys.iter().enumerate() {
            if i != j && keys[i] == *key && !path_is_backup(&found[j].path) {
                drop[i] = true;
                break;
            }
        }
    }
    let mut out = Vec::new();
    for (i, cand) in found.drain(..).enumerate() {
        if !drop[i] {
            out.push(cand);
        }
    }
    out.sort_by_key(|c| std::cmp::Reverse(c.modified));
    *found = out;
}

/// Every save found in the usual locations, newest first.
pub fn find_saves() -> Vec<SaveCandidate> {
    let starts = default_starts();
    let mut found = Vec::new();
    for start in &starts {
        collect_from(start, &mut found);
    }
    dedupe_and_sort(&mut found);
    found
}

/// Every save found beneath a single starting directory (for tests/tools).
pub fn find_saves_in(start: &Path) -> Vec<SaveCandidate> {
    let mut found = Vec::new();
    collect_from(start, &mut found);
    dedupe_and_sort(&mut found);
    found
}

/// `YYYY-MM-DD HH:MM` (UTC) from a `SystemTime`.
fn format_time(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        rem / 3600,
        (rem % 3600) / 60
    )
}

/// Days since the Unix epoch -> (year, month, day). Howard Hinnant's algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
    }

    #[test]
    fn finds_saves_case_insensitively_and_dedupes_backup() {
        let root = std::env::temp_dir().join(format!("lootsmith-locate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        // A ZLuda-style prefix with lowercase "documents"/"my games".
        let prefix = root.join("mygame/prefix/drive_c/users/steamuser");
        let docs = prefix.join("documents/my games/Borderlands 2/WillowGame/SaveData/123");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("Save0001.sav"), b"sav").unwrap();
        // Wine's backup copy — must be dropped in favour of the real one.
        let backup =
            prefix.join("My Documents BACKUP/My games/Borderlands 2/WillowGame/SaveData/123");
        std::fs::create_dir_all(&backup).unwrap();
        std::fs::write(backup.join("Save0001.sav"), b"sav").unwrap();
        // A second, older save.
        std::fs::write(docs.join("Save0002.sav"), b"sav").unwrap();
        // A non-Borderlands savedata directory, which must be ignored.
        let other = root.join("other/drive_c/users/x/SavedData");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("Save0001.sav"), b"sav").unwrap();

        let found = find_saves_in(&root);
        assert_eq!(found.len(), 2, "one per file, backup dropped: {found:?}");
        assert!(found
            .iter()
            .all(|c| !c.path.to_string_lossy().contains("BACKUP")));
        assert_eq!(found[0].steam_id().as_deref(), Some("123"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
