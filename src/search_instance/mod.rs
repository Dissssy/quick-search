mod app;
mod config;
use std::{collections::HashSet, thread::JoinHandle};

use quick_search_lib::{ColoredChar, Searchable_TO};

pub fn instance(search_bar: bool) {
    let plugins = load_plugins();

    if search_bar {
        let cl = super::CONFIG_FILE.lock();
        let config = cl.get();
        let mut app = app::App::new(plugins);
        if config.audio_enabled {
            let mut audio = rusty_audio::Audio::new();
            audio.add("notif", crate::AUDIO_FILE_PATH.clone());
            app.set_audio(audio);
        }
        egui_overlay::start(app);
    } else {
        let app = config::App::new();
        egui_overlay::start(app);
    }
}

pub fn preload() {
    let _ = load_plugins();
}

fn load_plugins() -> Vec<Plugin> {
    let dir = super::DIRECTORY.data_dir().join("plugins");
    log::trace!("plugins directory: {:?}", dir);
    let mut plugins = Vec::new();
    log::trace!("loading plugins");

    let files = match std::fs::read_dir(&dir) {
        Ok(files) => {
            let files = files.collect::<Vec<_>>();
            log::info!("Loaded plugins directory, found {} files", files.len());
            files
        }
        Err(e) => {
            log::error!("Failed to read plugins directory: {}", e);
            return plugins;
        }
    };

    let mut cl = super::CONFIG_FILE.lock();

    let mut to_remove = Vec::new();
    let mut taken_names = HashSet::new();
    let mut found_names = HashSet::new();

    {
        let config = cl.get();
        for entry in files {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    log::trace!("entry: {:?}", path);
                    // check if file name ends with .dll, .so, or .dylib
                    if let Some(file_name) = path.file_name() {
                        let file_name = file_name.to_string_lossy();
                        log::trace!("file name: {:?}", file_name);
                        if file_name.ends_with(".dll") || file_name.ends_with(".so") || file_name.ends_with(".dylib") {
                            log::trace!("plugin is a library");

                            match quick_search_lib::load_library(path.as_path()) {
                                Ok(library) => {
                                    log::trace!("library loaded");
                                    let plogon = library.get_searchable()(quick_search_lib::PluginId {
                                        filename: file_name.into_owned().into(),
                                    });
                                    log::trace!("searchable loaded");
                                    let name: &'static str = Searchable_TO::name(&plogon).into();
                                    found_names.insert(name);
                                    log::trace!("name: {}", name);
                                    if taken_names.contains(name) {
                                        log::error!("plugin name {} is already taken", name);
                                        continue;
                                    }
                                    if !config.get_plugin(name).enabled {
                                        log::info!("plugin {} is disabled", name);
                                        continue;
                                    }
                                    taken_names.insert(name);
                                    let colored_name = Searchable_TO::colored_name(&plogon);
                                    log::trace!("colored_name: {:?}", colored_name);
                                    let priority = config.get_plugin(name).priority;
                                    let id = Searchable_TO::plugin_id(&plogon);
                                    plugins.push(Plugin {
                                        name,
                                        colored_name: colored_char_to_layout_job(colored_name.into()),
                                        priority,
                                        id: id.clone(),
                                        // path,
                                        _p: plogon,
                                        _l: library,
                                    });
                                    log::trace!("plugin added to list");
                                }
                                Err(e) => {
                                    log::error!("Failed to load library: {}", e);
                                }
                            }
                        } else {
                            eprintln!("not a library: {:?}", file_name);
                        }
                    } else {
                        log::error!("Entry has no file name");
                    }
                }
                Err(e) => {
                    log::error!("Failed to read entry: {}", e);
                }
            }
        }
        for (name, _) in config.plugin_states.iter() {
            if !found_names.contains(name.as_str()) {
                to_remove.push(name.clone());
            }
        }
    }

    for name in to_remove {
        cl.get_mut().plugin_states.remove(&name);
    }

    log::info!("found and loaded {} plugins", plugins.len());
    plugins
}

struct Plugin {
    name: &'static str,
    colored_name: egui::text::LayoutJob,
    priority: u32,
    id: quick_search_lib::PluginId,
    // path: std::path::PathBuf,
    _p: Searchable_TO<'static, quick_search_lib::abi_stable::std_types::RBox<()>>,
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
        let p = self._p.clone();
        let query = query.to_string();

        let metadata = SearchMetadata {
            pretty_name: self.colored_name.clone(),
            priority: self.priority,
            raw_name: self.name.to_string(),
            id: self.id.clone(),
        };

        std::thread::spawn(move || (p.search(query.into()).into(), metadata))
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
}
