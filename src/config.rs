use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};

use serde::{Deserialize, Serialize, Serializer};

pub struct ConfigLoader {
    pub lock: Mutex<()>,
} // this struct will pass out a loaded and serialized reference to the config that, when dropped, will save the config file and release the lock

impl ConfigLoader {
    pub fn new() -> Self {
        // let file = super::DIRECTORY.config_dir().join("config.toml");
        // log::trace!("config file: {:?}", file);
        // let config_file = match std::fs::OpenOptions::new().read(true).write(true).create(true).open(&file) {
        //     Ok(config_file) => {
        //         log::info!("Opened config file");
        //         config_file
        //     }
        //     Err(e) => {
        //         log::error!("Failed to open config file: {}", e);
        //         panic!("Failed to open config file: {}", e);
        //     }
        // };
        ConfigLoader { lock: Mutex::new(()) }
    }

    pub fn lock(&self) -> ConfigLock<'_> {
        let lock = match self.lock.lock() {
            Ok(lock) => {
                log::info!("Locked config file");
                lock
            }
            Err(e) => e.into_inner(),
        };
        let config = Config::load();
        ConfigLock {
            config,
            lock: Some(lock),
            modified: false,
        }
    }
}

pub struct ConfigLock<'a> {
    config: Config,
    lock: Option<std::sync::MutexGuard<'a, ()>>,
    modified: bool,
}

impl ConfigLock<'_> {
    pub fn get(&self) -> &Config {
        &self.config
    }
    pub fn get_mut(&mut self) -> &mut Config {
        self.modified = true;
        &mut self.config
    }
}

impl Drop for ConfigLock<'_> {
    fn drop(&mut self) {
        if self.modified {
            log::trace!("config file modified");
            self.config.save();
            log::info!("config file saved");
        } else {
            log::info!("config file unmodified");
        }
        drop(self.lock.take())
    }
}

#[derive(Serialize)]
pub struct Config {
    #[serde(serialize_with = "ordered_map")]
    pub plugin_states: HashMap<String, PluginConfig>,
    pub audio_enabled: bool,
    pub truncate_context_length: usize,
    pub truncate_title_length: usize,
    pub appearance_delay: usize,
    pub entries_around_cursor: usize,
    pub group_entries_while_unselected: usize,
    pub total_search_delay: usize,
    pub show_countdown: bool,
}

fn ordered_map<S, K: Ord + Serialize, V: Serialize>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

impl Config {
    pub fn load() -> Self {
        let file = super::DIRECTORY.config_dir().join("config.toml");
        log::trace!("config file: {:?}", file);
        let config = match std::fs::read_to_string(&file) {
            Ok(config) => {
                log::info!("Loaded config file");
                config
            }
            Err(e) => {
                log::error!("Failed to load config file: {}", e);
                String::new()
            }
        };

        let config: PossibleConfig = match toml::from_str(&config) {
            Ok(config) => {
                log::info!("Parsed config file");
                config
            }
            Err(e) => {
                log::error!("Failed to parse config file: {} renaming and generating new one.", e);
                match std::fs::rename(&file, file.with_extension("toml.bak")) {
                    Ok(_) => {
                        log::info!("Renamed config file");
                    }
                    Err(e) => {
                        log::error!("Failed to rename config file: {}", e);
                    }
                };
                PossibleConfig::default()
            }
        };

        Config::from(config)
    }
    // pub fn get_plugin_mut(&mut self, name: &str) -> &mut PluginConfig {
    //     self.plugin_states.entry(name.to_string()).or_insert(PluginConfig { enabled: true, priority: 0 })
    // }
    // pub fn get_plugin(&mut self, name: &str) -> Option<&PluginConfig> {
    //     self.plugin_states.get(name)
    // }
    // pub fn get_or_default_plugin(&mut self, name: &str, default_config: quick_search_lib::Config) -> &PluginConfig {
    //     self.plugin_states.entry(name.to_string()).or_insert(PluginConfig {
    //         enabled: true,
    //         priority: 0,
    //         plugin_config: default_config,
    //     })
    // }
    pub fn get_mut_or_default_plugin(&mut self, name: &str, default_config: quick_search_lib::Config) -> &mut PluginConfig {
        self.plugin_states.entry(name.to_string()).or_insert(PluginConfig {
            enabled: true,
            priority: 0,
            delay: 100,
            plugin_config: default_config,
        })
    }
    pub fn get_plugin(&self, name: &str) -> Option<&PluginConfig> {
        self.plugin_states.get(name)
    }
    pub fn save(&self) {
        let file = super::DIRECTORY.config_dir().join("config.toml");
        log::trace!("config file: {:?}", file);
        let config = match toml::to_string(&self) {
            Ok(config) => {
                log::info!("Serialized config");
                config
            }
            Err(e) => {
                log::error!("Failed to serialize config: {}", e);
                return;
            }
        };

        match std::fs::write(&file, config) {
            Ok(_) => {
                log::info!("Wrote config file");
            }
            Err(e) => {
                log::error!("Failed to write config file: {}", e);
            }
        };
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub struct PluginConfig {
    pub enabled: bool,
    pub priority: u32,
    pub delay: u32,
    pub plugin_config: quick_search_lib::Config,
}

#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
struct PossibleConfig {
    #[serde(default)]
    plugin_states: Option<HashMap<String, PossiblePluginConfig>>,
    #[serde(default)]
    audio_enabled: Option<bool>,
    #[serde(default)]
    truncate_context_length: Option<usize>,
    #[serde(default)]
    truncate_title_length: Option<usize>,
    #[serde(default)]
    appearance_delay: Option<usize>,
    #[serde(default)]
    entries_around_cursor: Option<usize>,
    #[serde(default)]
    group_entries_while_unselected: Option<usize>,
    #[serde(default)]
    total_search_delay: Option<usize>,
    #[serde(default)]
    show_countdown: Option<bool>,
}

impl From<PossibleConfig> for Config {
    fn from(config: PossibleConfig) -> Self {
        Config {
            plugin_states: config.plugin_states.map(|h| h.into_iter().map(|(k, v)| (k, PluginConfig::from(v))).collect()).unwrap_or_default(),
            audio_enabled: config.audio_enabled.unwrap_or(true),
            truncate_context_length: config.truncate_context_length.unwrap_or(100),
            truncate_title_length: config.truncate_title_length.unwrap_or(100),
            appearance_delay: config.appearance_delay.unwrap_or(250),
            entries_around_cursor: config.entries_around_cursor.unwrap_or(2),
            group_entries_while_unselected: config.group_entries_while_unselected.unwrap_or(3),
            total_search_delay: config.total_search_delay.unwrap_or(500),
            show_countdown: config.show_countdown.unwrap_or(false),
        }
    }
}

#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
struct PossiblePluginConfig {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    priority: Option<u32>,
    #[serde(default)]
    delay: Option<u32>,
    #[serde(default)]
    plugin_config: Option<quick_search_lib::Config>,
}

impl From<PossiblePluginConfig> for PluginConfig {
    fn from(config: PossiblePluginConfig) -> Self {
        PluginConfig {
            enabled: config.enabled.unwrap_or(true),
            priority: config.priority.unwrap_or(0),
            delay: config.delay.unwrap_or(250),
            plugin_config: config.plugin_config.unwrap_or_default(),
        }
    }
}
