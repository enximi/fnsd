use crate::core::{
    FileResource, FolderResource, NoteResource, RemoteMillis, SettingResource, VaultPath,
};
use crate::hash::{file_content_hash, path_hash, text_content_hash};

use crate::vault::fs::{Result, VaultFs, error::io};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultScanOptions {
    pub obsidian_config_dir: Option<VaultPath>,
    pub custom_config_dirs: Vec<VaultPath>,
    pub ignored_rules: Vec<ScanRule>,
    pub ignored_extensions: Vec<String>,
    pub whitelist_rules: Vec<ScanRule>,
    pub max_note_bytes: Option<u64>,
    pub max_file_bytes: Option<u64>,
    pub max_setting_bytes: Option<u64>,
}

impl Default for VaultScanOptions {
    fn default() -> Self {
        Self {
            obsidian_config_dir: Some(
                VaultPath::new(".obsidian").expect("default config dir is valid"),
            ),
            custom_config_dirs: Vec::new(),
            ignored_rules: vec![
                ScanRule::path(".git"),
                ScanRule::path(".obsidian/plugins/obsidian-fast-note-sync/data.json"),
                ScanRule::path(".obsidian/community-plugins.json"),
                ScanRule::path(".obsidian/workspace.json"),
                ScanRule::path(".obsidian/workspace-mobile.json"),
            ],
            ignored_extensions: Vec::new(),
            whitelist_rules: Vec::new(),
            max_note_bytes: Some(20 * 1024 * 1024),
            max_file_bytes: Some(50 * 1024 * 1024),
            max_setting_bytes: Some(50 * 1024 * 1024),
        }
    }
}

impl VaultScanOptions {
    pub fn should_ignore(&self, path: &VaultPath) -> bool {
        if self.is_whitelisted(path) {
            return false;
        }

        self.is_ignored_by_rule(path) || self.has_ignored_extension(path)
    }

    pub fn is_setting_path(&self, path: &VaultPath) -> bool {
        self.is_obsidian_setting_path(path)
            || self
                .custom_config_dirs
                .iter()
                .any(|dir| path == dir || is_child_of(path, dir))
    }

    fn is_obsidian_config_dir(&self, path: &VaultPath) -> bool {
        self.obsidian_config_dir.as_ref() == Some(path)
    }

    fn is_under_obsidian_config_dir(&self, path: &VaultPath) -> bool {
        self.obsidian_config_dir
            .as_ref()
            .is_some_and(|dir| path == dir || is_child_of(path, dir))
    }

    fn is_custom_config_dir(&self, path: &VaultPath) -> bool {
        self.custom_config_dirs
            .iter()
            .any(|dir| path == dir || is_child_of(path, dir))
    }

    fn is_obsidian_setting_path(&self, path: &VaultPath) -> bool {
        let Some(config_dir) = &self.obsidian_config_dir else {
            return false;
        };

        let Some(relative) = path.as_str().strip_prefix(config_dir.as_str()) else {
            return false;
        };
        let relative = relative.trim_start_matches('/');
        let segments: Vec<&str> = relative.split('/').collect();

        match segments.as_slice() {
            [file_name] => file_name.ends_with(".json"),
            ["plugins", _, file_name] => {
                file_name.ends_with(".json")
                    || file_name.ends_with(".js")
                    || file_name.ends_with(".css")
            }
            ["themes", _, file_name] => file_name.ends_with(".css") || file_name.ends_with(".json"),
            _ => false,
        }
    }

    fn is_whitelisted(&self, path: &VaultPath) -> bool {
        self.whitelist_rules.iter().any(|rule| rule.matches(path))
    }

    fn is_ignored_by_rule(&self, path: &VaultPath) -> bool {
        self.ignored_rules.iter().any(|rule| rule.matches(path))
    }

    fn has_ignored_extension(&self, path: &VaultPath) -> bool {
        let Some((_, extension)) = path.as_str().rsplit_once('.') else {
            return false;
        };

        self.ignored_extensions
            .iter()
            .any(|item| extension.eq_ignore_ascii_case(item.trim_start_matches('.')))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRule {
    pattern: String,
    case_sensitive: bool,
    regex: bool,
}

impl ScanRule {
    pub fn path(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern
                .into()
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_string(),
            case_sensitive: false,
            regex: false,
        }
    }

    pub fn case_sensitive(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern
                .into()
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_string(),
            case_sensitive: true,
            regex: false,
        }
    }

    pub fn regex(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            case_sensitive: false,
            regex: true,
        }
    }

    pub fn case_sensitive_regex(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            case_sensitive: true,
            regex: true,
        }
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    pub fn is_regex(&self) -> bool {
        self.regex
    }

    fn matches(&self, path: &VaultPath) -> bool {
        let path = path.as_str();

        if self.regex {
            regex_prefix_matches(path, &self.pattern, self.case_sensitive)
        } else {
            pattern_prefix_matches(path, &self.pattern, self.case_sensitive)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VaultSnapshot {
    pub notes: Vec<NoteResource>,
    pub files: Vec<FileResource>,
    pub folders: Vec<FolderResource>,
    pub settings: Vec<SettingResource>,
}

impl VaultFs {
    pub fn scan(&self, options: &VaultScanOptions) -> Result<VaultSnapshot> {
        let mut snapshot = VaultSnapshot::default();
        self.scan_root(options, &mut snapshot)?;
        Ok(snapshot)
    }

    pub(crate) fn scan_path(
        &self,
        path: VaultPath,
        options: &VaultScanOptions,
        snapshot: &mut VaultSnapshot,
    ) -> Result<()> {
        if options.should_ignore(&path) {
            return Ok(());
        }

        let absolute = self.resolve_for_write(&path)?;
        let metadata = std::fs::symlink_metadata(&absolute).map_err(|err| io(&absolute, err))?;
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            return Ok(());
        }

        let absolute = self.resolve_existing(&path)?;

        if file_type.is_dir() {
            if options.is_obsidian_config_dir(&path) {
                self.scan_obsidian_config_dir(path, options, snapshot)?;
            } else if options.is_under_obsidian_config_dir(&path) {
                return Ok(());
            } else if options.is_custom_config_dir(&path) {
                self.scan_dir(path, options, snapshot)?;
            } else {
                snapshot.folders.push(FolderResource {
                    path: path.clone(),
                    path_hash: path_hash(path.as_str())?,
                    mtime: modified_millis(&metadata),
                });

                self.scan_dir(path, options, snapshot)?;
            }

            return Ok(());
        }

        if !file_type.is_file() {
            return Ok(());
        }

        let size = metadata.len();

        if options.is_setting_path(&path) {
            if exceeds_limit(size, options.max_setting_bytes) {
                return Ok(());
            }

            let content = std::fs::read_to_string(&absolute).map_err(|err| io(&absolute, err))?;
            snapshot.settings.push(SettingResource {
                path: path.clone(),
                path_hash: path_hash(path.as_str())?,
                content_hash: text_content_hash(&content),
                ctime: created_millis(&metadata),
                mtime: modified_millis(&metadata),
            });
            return Ok(());
        }

        if is_note_path(&path) {
            if exceeds_limit(size, options.max_note_bytes) {
                return Ok(());
            }

            let content = std::fs::read_to_string(&absolute).map_err(|err| io(&absolute, err))?;
            snapshot.notes.push(NoteResource {
                path: path.clone(),
                path_hash: path_hash(path.as_str())?,
                content_hash: text_content_hash(&content),
                ctime: created_millis(&metadata),
                mtime: modified_millis(&metadata),
            });
        } else {
            if exceeds_limit(size, options.max_file_bytes) {
                return Ok(());
            }

            let bytes = std::fs::read(&absolute).map_err(|err| io(&absolute, err))?;
            snapshot.files.push(FileResource {
                path: path.clone(),
                path_hash: path_hash(path.as_str())?,
                content_hash: file_content_hash(&bytes),
                size,
                ctime: created_millis(&metadata),
                mtime: modified_millis(&metadata),
            });
        }

        Ok(())
    }

    fn scan_root(&self, options: &VaultScanOptions, snapshot: &mut VaultSnapshot) -> Result<()> {
        let entries = std::fs::read_dir(self.root()).map_err(|err| io(self.root(), err))?;

        for entry in entries {
            let entry = entry.map_err(|err| io(self.root(), err))?;
            let child = self.path_from_absolute(entry.path())?;
            self.scan_path(child, options, snapshot)?;
        }

        Ok(())
    }

    fn scan_obsidian_config_dir(
        &self,
        path: VaultPath,
        options: &VaultScanOptions,
        snapshot: &mut VaultSnapshot,
    ) -> Result<()> {
        let absolute = self.resolve_existing(&path)?;
        let entries = std::fs::read_dir(&absolute).map_err(|err| io(&absolute, err))?;

        for entry in entries {
            let entry = entry.map_err(|err| io(&absolute, err))?;
            let child = self.path_from_absolute(entry.path())?;

            if is_root_json_setting(&path, &child) {
                self.scan_path(child, options, snapshot)?;
            }
        }

        self.scan_named_children(&path, "plugins", &["json", "js", "css"], options, snapshot)?;
        self.scan_named_children(&path, "themes", &["css", "json"], options, snapshot)?;

        Ok(())
    }

    fn scan_named_children(
        &self,
        root: &VaultPath,
        child_name: &str,
        extensions: &[&str],
        options: &VaultScanOptions,
        snapshot: &mut VaultSnapshot,
    ) -> Result<()> {
        let child_root = VaultPath::new(format!("{root}/{child_name}"))?;
        let Ok(absolute) = self.resolve_existing(&child_root) else {
            return Ok(());
        };
        let metadata = std::fs::symlink_metadata(&absolute).map_err(|err| io(&absolute, err))?;

        if !metadata.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&absolute).map_err(|err| io(&absolute, err))?;

        for entry in entries {
            let entry = entry.map_err(|err| io(&absolute, err))?;
            let plugin_or_theme_path = self.path_from_absolute(entry.path())?;
            let plugin_or_theme_absolute = self.resolve_for_write(&plugin_or_theme_path)?;
            let metadata = std::fs::symlink_metadata(&plugin_or_theme_absolute)
                .map_err(|err| io(&plugin_or_theme_absolute, err))?;

            if !metadata.is_dir() {
                continue;
            }

            let plugin_or_theme_absolute = self.resolve_existing(&plugin_or_theme_path)?;

            let files = std::fs::read_dir(&plugin_or_theme_absolute)
                .map_err(|err| io(&plugin_or_theme_absolute, err))?;

            for file in files {
                let file = file.map_err(|err| io(&plugin_or_theme_absolute, err))?;
                let file_path = self.path_from_absolute(file.path())?;

                if has_extension(&file_path, extensions) {
                    self.scan_path(file_path, options, snapshot)?;
                }
            }
        }

        Ok(())
    }

    fn scan_dir(
        &self,
        path: VaultPath,
        options: &VaultScanOptions,
        snapshot: &mut VaultSnapshot,
    ) -> Result<()> {
        let absolute = self.resolve_existing(&path)?;

        let entries = std::fs::read_dir(&absolute).map_err(|err| io(&absolute, err))?;

        for entry in entries {
            let entry = entry.map_err(|err| io(&absolute, err))?;
            let child = self.path_from_absolute(entry.path())?;
            self.scan_path(child, options, snapshot)?;
        }

        Ok(())
    }
}

fn is_note_path(path: &VaultPath) -> bool {
    has_extension(path, &["md"])
}

fn is_root_json_setting(config_dir: &VaultPath, path: &VaultPath) -> bool {
    let Some(relative) = path.as_str().strip_prefix(config_dir.as_str()) else {
        return false;
    };
    let relative = relative.trim_start_matches('/');

    !relative.contains('/') && relative.ends_with(".json")
}

fn has_extension(path: &VaultPath, extensions: &[&str]) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| {
            extensions
                .iter()
                .any(|item| extension.eq_ignore_ascii_case(item.trim_start_matches('.')))
        })
}

fn is_child_of(path: &VaultPath, parent: &VaultPath) -> bool {
    path.as_str()
        .strip_prefix(parent.as_str())
        .is_some_and(|rest| rest.starts_with('/'))
}

fn pattern_prefix_matches(path: &str, pattern: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        return path == pattern
            || path
                .strip_prefix(pattern)
                .is_some_and(|rest| rest.starts_with('/'));
    }

    let path = path.to_lowercase();
    let pattern = pattern.to_lowercase();

    path == pattern
        || path
            .strip_prefix(&pattern)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn regex_prefix_matches(path: &str, pattern: &str, case_sensitive: bool) -> bool {
    let pattern = format!("^{pattern}");
    let Ok(regex) = regex::RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .build()
    else {
        return false;
    };

    regex.is_match(path)
}

fn exceeds_limit(size: u64, limit: Option<u64>) -> bool {
    limit.is_some_and(|limit| size > limit)
}

fn created_millis(metadata: &std::fs::Metadata) -> RemoteMillis {
    metadata
        .created()
        .ok()
        .and_then(system_time_millis)
        .unwrap_or_else(|| modified_millis(metadata))
}

fn modified_millis(metadata: &std::fs::Metadata) -> RemoteMillis {
    metadata
        .modified()
        .ok()
        .and_then(system_time_millis)
        .unwrap_or_else(|| RemoteMillis::new(0).expect("zero timestamp is valid"))
}

fn system_time_millis(time: std::time::SystemTime) -> Option<RemoteMillis> {
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
    let millis = i64::try_from(duration.as_millis()).ok()?;
    RemoteMillis::new(millis).ok()
}
