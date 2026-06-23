use std::path::{Path, PathBuf};

use fns_core::{VaultName, VaultPath};
use fns_vault_fs::{ScanRule, VaultScanOptions};
use serde::Deserialize;
use url::Url;

use crate::{ConfigError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct AppConfig {
    pub server: ServerConfig,
    pub vault: VaultConfig,
    pub store: StoreConfig,
    pub scan: ScanConfig,
    pub sync: SyncConfig,
    pub client: ClientConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            vault: VaultConfig::default(),
            store: StoreConfig::default(),
            scan: ScanConfig::default(),
            sync: SyncConfig::default(),
            client: ClientConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let builder = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(
                config::Environment::with_prefix("FNS_HEADLESS")
                    .separator("__")
                    .prefix_separator("_"),
            );
        let config: Self = builder.build()?.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        required("server.url", &self.server.url)?;
        required("server.api-token", &self.server.api_token)?;

        self.server.parsed_url()?;

        VaultName::new(&self.vault.name)?;
        ensure_path("vault.root", &self.vault.root)?;
        ensure_path("store.path", &self.store.path)?;

        if let Some(path) = &self.scan.obsidian_config_dir {
            VaultPath::new(path)?;
        }

        for path in &self.scan.custom_config_dirs {
            VaultPath::new(path)?;
        }

        Ok(())
    }

    pub fn vault_name(&self) -> Result<VaultName> {
        Ok(VaultName::new(&self.vault.name)?)
    }

    pub fn scan_options(&self) -> Result<VaultScanOptions> {
        let mut options = VaultScanOptions::default();
        options.obsidian_config_dir = self
            .scan
            .obsidian_config_dir
            .as_deref()
            .map(VaultPath::new)
            .transpose()?;
        options.custom_config_dirs = self
            .scan
            .custom_config_dirs
            .iter()
            .map(VaultPath::new)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        options.ignored_rules = self
            .scan
            .ignore_rules
            .iter()
            .map(RuleConfig::to_rule)
            .collect();
        options.ignored_extensions = self.scan.ignore_extensions.clone();
        options.whitelist_rules = self
            .scan
            .whitelist_rules
            .iter()
            .map(RuleConfig::to_rule)
            .collect();
        options.max_note_bytes = self.scan.max_note_mb.map(mb_to_bytes);
        options.max_file_bytes = self.scan.max_file_mb.map(mb_to_bytes);
        options.max_setting_bytes = self.scan.max_setting_mb.map(mb_to_bytes);
        Ok(options)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServerConfig {
    pub url: String,
    pub api_token: String,
}

impl ServerConfig {
    pub fn parsed_url(&self) -> Result<Url> {
        let url = Url::parse(&self.url)?;

        if !matches!(url.scheme(), "http" | "https" | "ws" | "wss") {
            return Err(ConfigError::InvalidServerUrl);
        }

        Ok(url)
    }

    pub fn ws_url(&self) -> Result<String> {
        let mut url = self.parsed_url()?;
        let scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            "ws" => "ws",
            "wss" => "wss",
            _ => return Err(ConfigError::InvalidServerUrl),
        };

        url.set_scheme(scheme)
            .map_err(|_| ConfigError::InvalidServerUrl)?;
        Ok(url.to_string())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct VaultConfig {
    pub name: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct StoreConfig {
    pub path: PathBuf,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from(".fns/state.json"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ScanConfig {
    pub obsidian_config_dir: Option<String>,
    pub custom_config_dirs: Vec<String>,
    pub ignore_rules: Vec<RuleConfig>,
    pub ignore_extensions: Vec<String>,
    pub whitelist_rules: Vec<RuleConfig>,
    pub max_note_mb: Option<u64>,
    pub max_file_mb: Option<u64>,
    pub max_setting_mb: Option<u64>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            obsidian_config_dir: Some(".obsidian".to_string()),
            custom_config_dirs: Vec::new(),
            ignore_rules: vec![
                RuleConfig::path(".git"),
                RuleConfig::path(".obsidian/plugins/obsidian-fast-note-sync/data.json"),
                RuleConfig::path(".obsidian/community-plugins.json"),
                RuleConfig::path(".obsidian/workspace.json"),
                RuleConfig::path(".obsidian/workspace-mobile.json"),
            ],
            ignore_extensions: Vec::new(),
            whitelist_rules: Vec::new(),
            max_note_mb: Some(20),
            max_file_mb: Some(50),
            max_setting_mb: Some(50),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ClientConfig {
    pub name: String,
    pub version: String,
    pub protobuf: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            name: "fns-headless".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            protobuf: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SyncConfig {
    pub offline_delete_sync_enabled: bool,
    pub transfer_concurrency_enabled: bool,
    pub max_concurrent_transfers: usize,
    pub transfer_timeout_seconds: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            offline_delete_sync_enabled: false,
            transfer_concurrency_enabled: true,
            max_concurrent_transfers: 4,
            transfer_timeout_seconds: 60 * 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    Pattern(String),
    Detailed {
        pattern: String,
        #[serde(default)]
        case_sensitive: bool,
        #[serde(default)]
        regex: bool,
    },
}

impl RuleConfig {
    pub fn path(pattern: impl Into<String>) -> Self {
        Self::Pattern(pattern.into())
    }

    pub fn to_rule(&self) -> ScanRule {
        match self {
            Self::Pattern(pattern) => ScanRule::path(pattern),
            Self::Detailed {
                pattern,
                case_sensitive,
                regex,
            } => match (*case_sensitive, *regex) {
                (true, true) => ScanRule::case_sensitive_regex(pattern),
                (true, false) => ScanRule::case_sensitive(pattern),
                (false, true) => ScanRule::regex(pattern),
                (false, false) => ScanRule::path(pattern),
            },
        }
    }
}

fn required(name: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ConfigError::MissingField(name));
    }

    Ok(())
}

fn ensure_path(name: &'static str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        return Err(ConfigError::EmptyPath(name));
    }

    Ok(())
}

fn mb_to_bytes(value: u64) -> u64 {
    value.saturating_mul(1024 * 1024)
}
