mod app;
mod config;
use std::sync::Arc;
use std::{collections::HashSet, thread::JoinHandle};

use crate::LOGGER;
use quick_search_lib::Log;

use quick_search_lib::{ColoredChar, Searchable_TO};

pub fn instance(search_bar: bool) {
    let plugins = load_plugins();

    if search_bar {
        egui_overlay::start(app::App::new(plugins));
    } else {
        egui_overlay::start(config::App::new(plugins));
    }
}

pub fn preload() {
    let _ = load_plugins();
}

pub struct PluginLoadResult {
    pub plugins: Vec<Plugin>,
    // first string is the plugin path, second string is the error message
    pub errors: Vec<(String, String)>,
    pub missing: Vec<String>,
}

fn load_plugins() -> PluginLoadResult {
    let dir = super::DIRECTORY.data_dir().join("plugins");
    LOGGER.trace(&format!("plugins directory: {:?}", dir));
    let mut plugins = Vec::new();
    let mut errors = Vec::new();
    let mut missing = Vec::new();
    LOGGER.trace("loading plugins");

    let files = match std::fs::read_dir(&dir) {
        Ok(files) => {
            let files = files.collect::<Vec<_>>();
            LOGGER.info(&format!("Loaded plugins directory, found {} files", files.len()));
            files
        }
        Err(e) => {
            LOGGER.error(&format!("Failed to read plugins directory: {}", e));
            errors.push((dir.to_string_lossy().into(), format!("Failed to read plugins directory: {}", e)));
            return PluginLoadResult { plugins, errors, missing };
        }
    };

    // let mut to_remove = Vec::new();
    let mut taken_names = HashSet::new();
    let mut found_names = HashSet::new();
    let mut cl = super::CONFIG_FILE.lock();

    {
        let config = cl.get_mut();
        for entry in files {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    LOGGER.trace(&format!("entry: {:?}", path));
                    // check if file name ends with .dll, .so, or .dylib
                    if let Some(file_name) = path.file_name() {
                        let file_name = file_name.to_string_lossy();
                        LOGGER.trace(&format!("file name: {:?}", file_name));
                        if file_name.ends_with(".dll") || file_name.ends_with(".so") || file_name.ends_with(".dylib") {
                            LOGGER.trace("plugin is a library");

                            match quick_search_lib::load_library(path.as_path()) {
                                Ok(library) => {
                                    LOGGER.trace("library loaded");
                                    let scoped_logger = LOGGER.new_scoped(&file_name);
                                    let mut plogon = library.get_searchable()(
                                        quick_search_lib::PluginId {
                                            filename: file_name.into_owned().into(),
                                        },
                                        scoped_logger,
                                    );
                                    LOGGER.trace("searchable loaded");
                                    let name: &'static str = Searchable_TO::name(&plogon).into();
                                    found_names.insert(name);
                                    LOGGER.trace(&format!("name: {}", name));
                                    if taken_names.contains(name) {
                                        LOGGER.error(&format!("plugin name {} is already taken", name));
                                        errors.push((path.to_string_lossy().into(), format!("plugin name `{}` is already taken", name)));
                                        continue;
                                    }
                                    let default_plugin_config: quick_search_lib::Config = Searchable_TO::get_config_entries(&plogon);
                                    let plugin_info = config.get_mut_or_default_plugin(name, default_plugin_config.clone());
                                    if !plugin_info.enabled {
                                        LOGGER.info(&format!("plugin {} is disabled", name));
                                        continue;
                                    }
                                    taken_names.insert(name);
                                    let colored_name = Searchable_TO::colored_name(&plogon);
                                    LOGGER.trace(&format!("colored_name: {}", colored_name.iter().map(|c| c.char()).collect::<String>()));
                                    let id = Searchable_TO::plugin_id(&plogon);
                                    LOGGER.trace(&format!("id: {:?}", id));

                                    // do plugin config checking here
                                    for (key, value) in default_plugin_config.iter() {
                                        // we want to ensure that the plugin config contains the correct keys and that the enum variant of the value is the same, but NOT the contained value
                                        // if plugin_info.plugin_config.get(key.as_str()).is_none() {
                                        //     LOGGER.warn(&format!("plugin {} is missing config key {}", name, key));
                                        //     plugin_info.plugin_config.insert(key.clone(), value.clone());
                                        // } else if plugin_info.plugin_config.get(key.as_str()).map(|v| v.variant()) != Some(value.variant()) {
                                        //     LOGGER.warn(&format!("plugin {} has incorrect config key {}", name, key));
                                        //     plugin_info.plugin_config.insert(key.clone(), value.clone());
                                        // }

                                        match plugin_info.plugin_config.get_mut(key.as_str()) {
                                            Some(v) => match (v, value) {
                                                (quick_search_lib::EntryType::String { .. }, quick_search_lib::EntryType::String { .. }) => {}
                                                (quick_search_lib::EntryType::Bool { .. }, quick_search_lib::EntryType::Bool { .. }) => {}
                                                (quick_search_lib::EntryType::Int { min, max, .. }, quick_search_lib::EntryType::Int { min: new_min, max: new_max, .. }) => {
                                                    LOGGER.trace(&format!("plugin {} has int config key {}", name, key));
                                                    LOGGER.trace(&format!("old min: {:?}, old max: {:?}", min, max));
                                                    *min = *new_min;
                                                    *max = *new_max;
                                                    LOGGER.trace(&format!("new min: {:?}, new max: {:?}", min, max));
                                                }
                                                (quick_search_lib::EntryType::Float { min, max, .. }, quick_search_lib::EntryType::Float { min: new_min, max: new_max, .. }) => {
                                                    LOGGER.trace(&format!("plugin {} has float config key {}", name, key));
                                                    LOGGER.trace(&format!("old min: {:?}, old max: {:?}", min, max));
                                                    *min = *new_min;
                                                    *max = *new_max;
                                                    LOGGER.trace(&format!("new min: {:?}, new max: {:?}", min, max));
                                                }
                                                (quick_search_lib::EntryType::Enum { options, .. }, quick_search_lib::EntryType::Enum { options: new_options, .. }) => {
                                                    LOGGER.trace(&format!("plugin {} has enum config key {}", name, key));
                                                    LOGGER.trace(&format!("old options: {:#?}", options));
                                                    *options = new_options.clone();
                                                    LOGGER.trace(&format!("new options: {:#?}", options));
                                                }
                                                _ => {
                                                    LOGGER.warn(&format!("plugin {} has incorrect config key {}", name, key));
                                                    plugin_info.plugin_config.insert(key.clone(), value.clone());
                                                }
                                            },
                                            None => {
                                                LOGGER.warn(&format!("plugin {} is missing config key {}", name, key));
                                                plugin_info.plugin_config.insert(key.clone(), value.clone());
                                            }
                                        }
                                    }

                                    // now that we've done all the validation, let's do some key trimming
                                    let mut to_remove = Vec::new();
                                    for (key, _) in plugin_info.plugin_config.iter() {
                                        if default_plugin_config.get(key.as_str()).is_none() {
                                            to_remove.push(key.clone());
                                        }
                                    }
                                    for key in to_remove {
                                        plugin_info.plugin_config.remove(&key);
                                    }

                                    // and finally, send a clone of the plugin config back to the plugin
                                    Searchable_TO::lazy_load_config(&mut plogon, plugin_info.plugin_config.clone());

                                    plugins.push(Plugin {
                                        name,
                                        // delay: plugin_info.delay,
                                        colored_name: colored_char_to_layout_job(colored_name.into()),
                                        priority: plugin_info.priority,
                                        id: id.clone(),
                                        // path,
                                        _p: Arc::new(plogon),
                                        _l: library,
                                    });
                                    LOGGER.trace("plugin added to list");
                                }
                                Err(e) => {
                                    LOGGER.error(&format!("Failed to load library: {}", e));
                                    errors.push((path.to_string_lossy().into(), "Library was compiled for a different version of the ABI".into()));
                                }
                            }
                        } else {
                            LOGGER.error(&format!("not a library: {:?}", file_name));
                            errors.push((path.to_string_lossy().into(), "not a library".into()));
                        }
                    } else {
                        LOGGER.error("Entry has no file name");
                        errors.push((path.to_string_lossy().into(), "Entry has no file name".into()));
                    }
                }
                Err(e) => {
                    LOGGER.error(&format!("Failed to read entry: {}", e));
                    errors.push((dir.to_string_lossy().into(), "Failed to read file".into()));
                }
            }
        }
        // for (name, _) in config.plugin_states.iter() {
        //     if !found_names.contains(name.as_str()) {
        //         to_remove.push(name.clone());
        //     }
        // }

        for name in config.plugin_states.keys() {
            if !found_names.contains(name.as_str()) {
                missing.push(name.clone());
            }
        }
    }

    // for name in to_remove {
    //     cl.get_mut().plugin_states.remove(&name);
    // }

    LOGGER.info(&format!("found and loaded {} plugins", plugins.len()));
    PluginLoadResult { plugins, errors, missing }
}

pub struct Plugin {
    name: &'static str,
    colored_name: egui::text::LayoutJob,
    priority: u32,
    // delay: u32,
    id: quick_search_lib::PluginId,
    // path: std::path::PathBuf,
    _p: Arc<Searchable_TO<'static, quick_search_lib::abi_stable::std_types::RBox<()>>>,
    _l: quick_search_lib::SearchLib_Ref,
}

impl Plugin {
    // fn search(&self, query: &str) -> Vec<quick_search_lib::SearchResult> {
    //     self._p.search(query.into()).into()
    // }
    fn execute(&self, result: &quick_search_lib::SearchResult) {
        self._p.execute(result);
    }
    fn search_delayed(&self, query: &str) -> JoinHandle<(Vec<quick_search_lib::SearchResult>, SearchMetadata)> {
        let p = Arc::clone(&self._p);
        let query = query.to_string();

        let mut metadata = SearchMetadata {
            pretty_name: self.colored_name.clone(),
            priority: self.priority,
            raw_name: self.name.to_string(),
            id: self.id.clone(),
            num_results: 0,
        };

        std::thread::spawn(move || {
            let res: Vec<quick_search_lib::SearchResult> = p.search(query.into()).into();
            metadata.num_results = res.len();
            (res, metadata)
        })
    }
}

pub fn colored_char_to_layout_job(colored_chars: Vec<ColoredChar>) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for char in colored_chars {
        job.append(
            char.char().to_string().as_str(),
            0.0,
            egui::TextFormat {
                color: {
                    let (r, g, b, a) = into_rgb(char.color());
                    egui::Color32::from_rgba_premultiplied(r, g, b, a)
                },
                ..Default::default()
            },
        )
    }
    job
}

fn into_rgb(color: u32) -> (u8, u8, u8, u8) {
    // 0xRRGGBBAA
    let r = (color >> 24) as u8;
    let g = (color >> 16) as u8;
    let b = (color >> 8) as u8;
    let a = color as u8;
    (r, g, b, a)
}

pub struct SearchMetadata {
    pub pretty_name: egui::text::LayoutJob,
    pub priority: u32,
    pub raw_name: String,
    pub id: quick_search_lib::PluginId,
    pub num_results: usize,
}
