use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};

use crate::LOGGER;
use quick_search_lib::Log;

use serde::{Deserialize, Serialize, Serializer};

pub struct ConfigLoader {
    pub lock: Mutex<()>,
} // this struct will pass out a loaded and serialized reference to the config that, when dropped, will save the config file and release the lock

impl ConfigLoader {
    pub fn new() -> Self {
        // let file = super::DIRECTORY.config_dir().join("config.toml");
        // LOGGER.trace("config file: {:?}", file);
        // let config_file = match std::fs::OpenOptions::new().read(true).write(true).create(true).open(&file) {
        //     Ok(config_file) => {
        //         LOGGER.info("Opened config file");
        //         config_file
        //     }
        //     Err(e) => {
        //         LOGGER.error("Failed to open config file: {}", e);
        //         panic("Failed to open config file: {}", e);
        //     }
        // };
        ConfigLoader { lock: Mutex::new(()) }
    }

    pub fn lock(&self) -> ConfigLock<'_> {
        let lock = match self.lock.lock() {
            Ok(lock) => {
                LOGGER.info("Locked config file");
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
            LOGGER.trace("config file modified");
            self.config.save();
            LOGGER.info("config file saved");
        } else {
            LOGGER.info("config file unmodified");
        }
        drop(self.lock.take())
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
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
    // pub show_countdown: bool,
    // pub flash_taskbar: bool,
    pub gap_between_search_bar_and_results: f32,
    pub timezone: chrono_tz::Tz,
    pub chrono_format_string: String,
    pub time_font_size: f32,
    pub clock_enabled: bool,
    pub log_level: quick_search_lib::LogLevelOrCustom,
    pub max_log_size: usize,
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
        LOGGER.trace(&format!("config file: {:?}", file));
        let config = match std::fs::read_to_string(&file) {
            Ok(config) => {
                LOGGER.info("Loaded config file");
                config
            }
            Err(e) => {
                LOGGER.error(&format!("Failed to load config file: {}", e));
                String::new()
            }
        };

        let config: PossibleConfig = match toml::from_str(&config) {
            Ok(config) => {
                LOGGER.info("Parsed config file");
                config
            }
            Err(e) => {
                LOGGER.error(&format!("Failed to parse config file: {} renaming and generating new one.", e));
                match std::fs::rename(&file, file.with_extension("toml.bak")) {
                    Ok(_) => {
                        LOGGER.info("Renamed config file");
                    }
                    Err(e) => {
                        LOGGER.error(&format!("Failed to rename config file: {}", e));
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
    pub fn save(&mut self) {
        // verify the format string is valid
        for item in chrono::format::StrftimeItems::new(&self.chrono_format_string) {
            if let chrono::format::Item::Error = item {
                LOGGER.error(&format!("Invalid chrono format string: {}", self.chrono_format_string));
                self.chrono_format_string = "%Y-%m-%d %H:%M:%S".to_string();
                break;
            }
        }
        let file = super::DIRECTORY.config_dir().join("config.toml");
        LOGGER.trace(&format!("config file: {:?}", file));
        let config = match toml::to_string(&self) {
            Ok(config) => {
                LOGGER.info("Serialized config");
                config
            }
            Err(e) => {
                LOGGER.error(&format!("Failed to serialize config: {}", e));
                return;
            }
        };

        match std::fs::write(&file, config) {
            Ok(_) => {
                LOGGER.info("Wrote config file");
            }
            Err(e) => {
                LOGGER.error(&format!("Failed to write config file: {}", e));
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
    // #[serde(default)]
    // show_countdown: Option<bool>,
    // #[serde(default)]
    // flash_taskbar: Option<bool>,
    #[serde(default)]
    gap_between_search_bar_and_results: Option<f32>,
    #[serde(default)]
    timezone: Option<chrono_tz::Tz>,
    #[serde(default)]
    chrono_format_string: Option<String>,
    #[serde(default)]
    time_font_size: Option<f32>,
    #[serde(default)]
    clock_enabled: Option<bool>,
    #[serde(default)]
    log_level: Option<quick_search_lib::LogLevelOrCustom>,
    #[serde(default)]
    max_log_size: Option<usize>,
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
            // show_countdown: config.show_countdown.unwrap_or(false),
            // flash_taskbar: config.flash_taskbar.unwrap_or(true),
            gap_between_search_bar_and_results: config.gap_between_search_bar_and_results.unwrap_or(10.0),
            timezone: config.timezone.unwrap_or(chrono_tz::Tz::UTC),
            chrono_format_string: config.chrono_format_string.unwrap_or_else(|| "%Y-%m-%d %H:%M:%S".to_string()),
            time_font_size: config.time_font_size.unwrap_or(20.0),
            clock_enabled: config.clock_enabled.unwrap_or(true),
            log_level: config.log_level.unwrap_or(quick_search_lib::LogLevelOrCustom::from_min_level(quick_search_lib::LogLevel::Error)),
            max_log_size: config.max_log_size.unwrap_or(1024),
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
