use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};

use serde::{Serialize, Serializer};

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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    #[serde(serialize_with = "ordered_map")]
    pub plugin_states: HashMap<String, PluginConfig>,
    #[serde(default = "default_audio_enabled")]
    pub audio_enabled: bool,
}

fn ordered_map<S, K: Ord + Serialize, V: Serialize>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

fn default_audio_enabled() -> bool {
    true
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

        let config: Config = match toml::from_str(&config) {
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
                Config {
                    plugin_states: HashMap::new(),
                    audio_enabled: true,
                }
            }
        };

        config
    }
    // pub fn get_plugin_mut(&mut self, name: &str) -> &mut PluginConfig {
    //     self.plugin_states.entry(name.to_string()).or_insert(PluginConfig { enabled: true, priority: 0 })
    // }
    pub fn get_plugin(&self, name: &str) -> &PluginConfig {
        self.plugin_states.get(name).unwrap_or(&PluginConfig { enabled: true, priority: 0 })
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

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginConfig {
    pub enabled: bool,
    pub priority: u32,
}
