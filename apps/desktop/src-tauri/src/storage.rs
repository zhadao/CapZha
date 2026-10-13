use crate::general_settings::GeneralSettingsStore;
use chrono::{DateTime, Local};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

const MAX_DATE_DEPTH: usize = 3;
const PROJECT_EXTENSION: &str = "cap";

pub fn default_recordings_dir(app: &AppHandle) -> PathBuf {
    if let Ok(videos) = app.path().video_dir() {
        return videos.join("Cap");
    }

    if let Ok(data) = app.path().app_data_dir() {
        return data.join("recordings");
    }

    PathBuf::from("Cap")
}

pub fn recordings_root(app: &AppHandle) -> PathBuf {
    let configured = GeneralSettingsStore::get(app)
        .ok()
        .flatten()
        .and_then(|settings| settings.recordings_dir)
        .filter(|dir| !dir.trim().is_empty());

    let root = match configured {
        Some(dir) => PathBuf::from(dir),
        None => default_recordings_dir(app),
    };

    std::fs::create_dir_all(&root).ok();
    register_asset_scope(app, &root);
    root
}

fn registered_scopes() -> &'static Mutex<HashSet<PathBuf>> {
    static REGISTERED: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    REGISTERED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn register_asset_scope(app: &AppHandle, root: &Path) {
    let already_registered = {
        let mut registered = match registered_scopes().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        !registered.insert(root.to_path_buf())
    };

    if already_registered {
        return;
    }

    let _ = app.asset_protocol_scope().allow_directory(root, true);
}

pub fn legacy_recordings_root(app: &AppHandle) -> Option<PathBuf> {
    let path = app.path().app_data_dir().ok()?.join("recordings");
    path.exists().then_some(path)
}

pub fn day_dir(root: &Path, now: DateTime<Local>) -> PathBuf {
    root.join(now.format("%Y").to_string())
        .join(now.format("%m").to_string())
        .join(now.format("%d").to_string())
}

pub fn timestamp_name(now: DateTime<Local>, extension: &str) -> String {
    format!(
        "cap_{}.{}",
        now.format("%Y-%m-%d_%H.%M.%S"),
        extension.trim_start_matches('.')
    )
}

pub fn new_project_dir(app: &AppHandle, now: DateTime<Local>) -> PathBuf {
    let dir = day_dir(&recordings_root(app), now);
    std::fs::create_dir_all(&dir).ok();
    dir.join(unique_name(&dir, &timestamp_name(now, PROJECT_EXTENSION)))
}

pub fn delivery_path(app: &AppHandle, now: DateTime<Local>, extension: &str) -> PathBuf {
    let dir = day_dir(&recordings_root(app), now);
    std::fs::create_dir_all(&dir).ok();
    dir.join(unique_name(&dir, &timestamp_name(now, extension)))
}

fn unique_name(dir: &Path, name: &str) -> String {
    if !dir.join(name).exists() {
        return name.to_string();
    }

    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("cap");
    let extension = path.extension().and_then(|value| value.to_str());

    let mut counter = 1;
    loop {
        let candidate = match extension {
            Some(ext) => format!("{stem} ({counter}).{ext}"),
            None => format!("{stem} ({counter})"),
        };

        if !dir.join(&candidate).exists() {
            return candidate;
        }

        counter += 1;
    }
}

pub fn iter_project_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut roots = vec![recordings_root(app)];

    if let Some(legacy) = legacy_recordings_root(app) {
        if !roots.contains(&legacy) {
            roots.push(legacy);
        }
    }

    let mut projects = Vec::new();
    for root in roots {
        collect_projects(&root, 0, &mut projects);
    }

    projects
}

fn collect_projects(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_DATE_DEPTH {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        if path.extension().and_then(|value| value.to_str()) == Some(PROJECT_EXTENSION) {
            out.push(path);
        } else {
            collect_projects(&path, depth + 1, out);
        }
    }
}

pub fn project_scan_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    for project in iter_project_dirs(app) {
        if let Some(parent) = project.parent() {
            if !dirs.iter().any(|dir| dir == parent) {
                dirs.push(parent.to_path_buf());
            }
        }
    }

    dirs
}

pub fn cleanup_old_recordings(app: &AppHandle) {
    let max = GeneralSettingsStore::get(app)
        .ok()
        .flatten()
        .map(|settings| settings.max_recordings)
        .unwrap_or(50) as usize;

    if max == 0 {
        return;
    }

    let mut projects = iter_project_dirs(app);

    if projects.len() <= max {
        return;
    }

    projects.sort_by(|a, b| project_time(b).cmp(&project_time(a)));

    for path in projects.into_iter().skip(max) {
        if !path.join("recording-meta.json").exists() {
            continue;
        }

        if let Err(error) = std::fs::remove_dir_all(&path) {
            tracing::error!(
                "Failed to remove old recording '{}': {error}",
                path.display()
            );
        }
    }
}

pub fn link_or_copy(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        let _ = std::fs::remove_file(destination);
    }

    if std::fs::hard_link(source, destination).is_ok() {
        return Ok(());
    }

    std::fs::copy(source, destination).map(|_| ()).map_err(|error| {
        format!(
            "Failed to copy '{}' to '{}': {error}",
            source.display(),
            destination.display()
        )
    })
}

fn project_time(path: &Path) -> std::time::SystemTime {
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(created) = metadata.created() {
            return created;
        }

        if let Ok(modified) = metadata.modified() {
            return modified;
        }
    }

    std::time::SystemTime::UNIX_EPOCH
}
