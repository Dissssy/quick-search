use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub plugin_states: HashMap<String, PluginConfig>,
    #[serde(default = "default_audio_enabled")]
    pub audio_enabled: bool,
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
    pub fn get_plugin(&mut self, name: &str) -> &mut PluginConfig {
        self.plugin_states.entry(name.to_string()).or_insert(PluginConfig { enabled: true, priority: 0 })
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub priority: u32,
}
