use std::{collections::HashMap, str::FromStr};

use egui::{Button, Color32, Label, RichText};
use egui_extras::{Column, TableBuilder};
use quick_search_lib::abi_stable::{std_types::RString, traits::IntoReprRust};

use crate::config::{ConfigLock, PluginConfig};

use super::PluginLoadResult;

pub struct App<'a> {
    config_lock: ConfigLock<'a>,
    loadresults: PluginLoadResult,
    no_plugins_including_missing: bool,
    states: Vec<(String, PluginConfig)>,
    size: Option<egui::Vec2>,
    positioned: bool,
    passthrough: bool,
    force_redraw_now: bool,
    close_at_end: CloseState,
    time: std::time::Instant,

    menu_open_for: Option<usize>,
    // autolaunch: auto_launch::AutoLaunch,
    // auto: bool,
    // auto_error: Option<String>,
    autolaunchinfo: Option<AutoLaunchInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseState {
    DoNothing,
    CloseNoSave,
    CloseSave,
}

impl App<'_> {
    pub fn new(loadresults: PluginLoadResult) -> Self {
        let config_lock = crate::CONFIG_FILE.lock();
        let mut states: Vec<(String, PluginConfig)> = config_lock.get().plugin_states.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        // sort:
        // put enabled plugins first, disabled plugins come after
        // within each group, sort by priority
        // if priority is the same, sort by name
        states.sort_by(|a, b| {
            if a.1.enabled == b.1.enabled {
                if a.1.priority == b.1.priority {
                    a.0.cmp(&b.0)
                } else {
                    b.1.priority.cmp(&a.1.priority)
                }
            } else if a.1.enabled {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        let autolaunchinfo = {
            if *crate::CURRENT_PATH == *crate::CORRECT_PATH {
                let autolaunch = auto_launch::AutoLaunchBuilder::new()
                    .set_app_name("QuickSearch")
                    .set_app_path(crate::get_correct_path().to_str().expect("path is not valid"))
                    .build()
                    .expect("failed to create autolaunch");

                Some(AutoLaunchInfo {
                    enabled: autolaunch.is_enabled().expect("failed to check autolaunch"),
                    error: None,
                    autolaunch,
                })
            } else {
                None
            }
        };

        Self {
            no_plugins_including_missing: states.iter().filter(|(name, _)| !loadresults.missing.contains(name)).count() == 0,
            loadresults,
            states,
            config_lock,
            size: None,
            positioned: false,
            passthrough: false,
            force_redraw_now: false,
            close_at_end: CloseState::DoNothing,
            time: std::time::Instant::now(),
            menu_open_for: None,
            // auto: autolaunch.is_enabled().expect("failed to check autolaunch"),
            // autolaunch,
            // auto_error: None,
            autolaunchinfo,
        }
    }
}

struct AutoLaunchInfo {
    enabled: bool,
    error: Option<String>,
    autolaunch: auto_launch::AutoLaunch,
}

impl<'a> egui_overlay::EguiOverlay for App<'a> {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut egui_overlay::egui_render_three_d::ThreeDBackend,
        glfw_backend: &mut egui_overlay::egui_window_glfw_passthrough::GlfwBackend,
    ) {
        if self.size.is_none() {
            glfw_backend.glfw.with_connected_monitors(|_glfw, monitors| {
                let monitor = monitors.first();
                match monitor {
                    None => {
                        log::error!("no monitor");
                    }
                    Some(monitor) => {
                        // this code will literally only run once so we're gonna also request focus
                        // unsafe {
                        //     let window_ptr = egui_overlay::egui_window_glfw_passthrough::glfw::Context::window_ptr(&glfw_backend.window);
                        //     log::!("window_ptr: {:p}", window_ptr);
                        //     log::!("null: {}", window_ptr.is_null());
                        //     if !window_ptr.is_null() {
                        //         let r = SetForegroundWindow(std::mem::transmute(window_ptr));
                        //         log::!("setforegroundwindow: {}", r);
                        //     }
                        // }
                        // std::thread::sleep(std::time::Duration::from_millis(100));
                        // glfw_backend.window.focus();
                        // glfw_backend.window.hide();
                        glfw_backend.window.show();
                        glfw_backend.window.set_mouse_passthrough(true);
                        glfw_backend.window.set_title("QuickSearch Config");
                        glfw_backend.window.set_icon_from_pixels(crate::icon_pixelimages());

                        // std::thread::sleep(std::time::Duration::from_millis(100));

                        let current_focus_name = unsafe {
                            let current = winapi::um::winuser::GetForegroundWindow();
                            let mut window_title = [0u16; 1024];
                            let len = winapi::um::winuser::GetWindowTextW(current, window_title.as_mut_ptr(), window_title.len() as i32);
                            let current_name = String::from_utf16_lossy(&window_title[..len as usize]);
                            log::info!("current window: {}", current_name);
                            current_name
                        };

                        if current_focus_name != "QuickSearch Config" {
                            // glfw_backend.window.hide();
                            // glfw_backend.window.show();
                            glfw_backend.window.set_should_close(true);
                        } //else if let Some(audio) = &mut self.audio {
                          //    audio.play("notif");
                          //}

                        // let (x, y) = monitor.get_physical_size();
                        // let (sx, sy) = monitor.get_content_scale();
                        // log::!("monitor size: {}x{}", x, y);
                        // log::!("monitor scale: {}x{}", sx, sy);
                        // *v = Some(Vec2::new(x as f32, y as f32));

                        // if let Some(mode) = monitor.get_video_mode() {
                        //     let (x, y) = (mode.width, mode.height);
                        //     log::!("monitor size: {}x{}", x, y);
                        //     *v = Some(Vec2::new(x as f32, y as f32));
                        // } // THIS SCREWED UP MY MONITOR LOL

                        let (x1, y1, x2, y2) = monitor.get_workarea();
                        log::info!("monitor workarea: {}x{} {}x{}", x1, y1, x2, y2);
                        self.size = Some(egui::Vec2::new(x2 as f32, y2 as f32));
                    }
                }
            });
        } else if self.time.elapsed().as_millis() > self.config_lock.get().appearance_delay as u128 {
            if let Some(size) = self.size {
                if !self.positioned {
                    glfw_backend.window.set_pos(0, 0);
                    glfw_backend.window.set_size(size.x as i32 - 1, size.y as i32 - 1);
                    self.positioned = true;
                }
            }

            let (midwindowx, midwindowy) = {
                let (x, y) = glfw_backend.window.get_size();
                (x / 2, y / 2)
            };

            egui_context.set_visuals({
                let mut visuals = egui::Visuals::dark();
                visuals.popup_shadow.extrusion = 0.0;
                visuals.window_shadow.extrusion = 0.0;
                visuals
            });

            egui::Window::new("Config")
                .title_bar(false)
                // .fixed_pos(Pos2::new(midwindowx as f32 - 200., midwindowy as f32 - 30.))
                // .fixed_size(Vec2::new(400., 60.))
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0., 0.))
                .show(egui_context, |ui| {
                    ui.add(egui::Slider::new(&mut self.config_lock.get_mut().truncate_title_length, 25..=250).text("Truncate title length"))
                        .on_hover_text("Set the maximum length of the title text for a search result");
                    ui.add(egui::Slider::new(&mut self.config_lock.get_mut().truncate_context_length, 25..=250).text("Truncate context length"))
                        .on_hover_text("Set the maximum length of the context text for a search result");
                    ui.add(egui::Slider::new(&mut self.config_lock.get_mut().appearance_delay, 0..=1000).text("Appearance delay"))
                        .on_hover_text("Set the delay in ms before the search bar appears after the hotkey is pressed, lower values may cause flickering on some systems.");
                    ui.add(egui::Slider::new(&mut self.config_lock.get_mut().entries_around_cursor, 0..=7).text("Entries around cursor"))
                        .on_hover_text("Set the number of entries around the cursor to display while scrolling. e.g. if set to 2, 5 entries centered around the cursor will be displayed.");
                    ui.add(egui::Slider::new(&mut self.config_lock.get_mut().group_entries_while_unselected, 0..=10).text("Entries while unselected"))
                        .on_hover_text("Set the number of entries to display from each group while the search bar is not selected. set to 0 to display all entries.");
                    ui.add(egui::Slider::new(&mut self.config_lock.get_mut().total_search_delay, 0..=10000).text("Search delay"))
                        .on_hover_text("Set the debounce time in ms, lower values may run excessive searches, higher values mean a longer delay before the search is run.");
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.config_lock.get_mut().audio_enabled, "Sound effects")
                            .on_hover_text("Enable or disable sound effects when the search bar is opened");
                        // ui.checkbox(&mut self.config_lock.get_mut().flash_taskbar, "Flash taskbar")
                        //     .on_hover_text("Enable or disable flashing the taskbar when the search bar is opened");
                        // ui.checkbox(&mut self.config_lock.get_mut().show_countdown, "Show countdown")
                        //     .on_hover_text("Enable or disable the countdown until the searches are dispatched");
                        if let Some(ref mut autolaunchinfo) = self.autolaunchinfo {
                            ui.horizontal(|ui| {
                                if ui
                                    .checkbox(&mut autolaunchinfo.enabled, "Run on startup")
                                    .on_hover_text("Enable or disable running QuickSearch on startup")
                                    .changed()
                                {
                                    if autolaunchinfo.enabled {
                                        if let Err(e) = autolaunchinfo.autolaunch.enable() {
                                            let error = format!("failed to enable autolaunch: {}", e);
                                            log::error!("{}", error);
                                            autolaunchinfo.error = Some(error);
                                            autolaunchinfo.enabled = false;
                                        };
                                    } else if let Err(e) = autolaunchinfo.autolaunch.disable() {
                                        let error = format!("failed to disable autolaunch: {}", e);
                                        log::error!("{}", error);
                                        autolaunchinfo.enabled = true;
                                    }
                                }
                                if let Some(error) = &autolaunchinfo.error {
                                    ui.label(RichText::new(error).color(Color32::RED));
                                }
                            });
                        } else {
                            ui.label("AutoLaunch not available, run QuickSearch from the correct location to enable it.");
                        }
                    });
                    ui.separator();

                    if self.states.is_empty() || self.no_plugins_including_missing {
                        ui.label("No plugins found");
                    } else {
                        // for (name, state) in self.states.iter_mut() {
                        //     ui.horizontal(|ui| {
                        //         ui.label(name);
                        //         ui.checkbox(&mut state.enabled, "Enabled");
                        //         ui.label("Priority");
                        //         ui.add(egui::Slider::new(&mut state.priority, 0..=128).text("##priority"));
                        //     });
                        // }
                        TableBuilder::new(ui)
                            .column(Column::auto().resizable(false))
                            .column(Column::auto().resizable(false))
                            .column(Column::auto().resizable(false))
                            .column(Column::remainder())
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.add(nowrap_heading("Plugin")).on_hover_text("The name of the plugin");
                                });
                                header.col(|ui| {
                                    ui.add(nowrap_heading("Enabled")).on_hover_text("Enable or disable the plugin");
                                });
                                header.col(|ui| {
                                    ui.add(nowrap_heading("Priority")).on_hover_text("Set the priority of the plugin (higher priority shows up above lower priority)");
                                });
                                header.col(|ui| {
                                    ui.add(nowrap_heading("Delay")).on_hover_text("Set the delay in ms before the plugin is queried after the search bar changes. Lower values may cause excessive queries, higher values may cause the plugin to be slow to respond.");
                                });
                            })
                            .body(|mut body| {
                                for (i, (name, state)) in self.states.iter_mut().enumerate() {
                                    if !self.loadresults.missing.contains(name) {
                                        body.row(20.0, |mut row| {
                                            row.col(|ui| {
                                                if !state.plugin_config.empty() {
                                                    if self.menu_open_for == Some(i) {
                                                        if ui
                                                            .add(Button::new(RichText::new(&*name).italics().color(Color32::LIGHT_GREEN)).wrap(false))
                                                            .on_hover_cursor(egui::CursorIcon::Alias)
                                                            .on_hover_text("Plugin has extra configurations")
                                                            .clicked()
                                                        {
                                                            log::trace!("Close menu for {}", i);
                                                            self.menu_open_for = None;
                                                        }

                                                        // todo: window for plugin config
                                                        egui::Window::new(format!("{} extra configurations", name))
                                                            .title_bar(true).collapsible(false)
                                                            // .fixed_pos(Pos2::new(midwindowx as f32 - 200., midwindowy as f32 - 30.))
                                                            // .fixed_size(Vec2::new(400., 60.))
                                                            .resizable(false)
                                                            // .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0., 0.))
                                                            .pivot(egui::Align2::CENTER_CENTER)
                                                            .default_pos(egui::Pos2::new(midwindowx as f32, midwindowy as f32))
                                                            .show(egui_context, |ui| {
                                                                for (k, v) in state.plugin_config.iter_mut() {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(k.as_str());
                                                                        match v {
                                                                            quick_search_lib::EntryType::Bool { ref mut value } => {
                                                                                ui.checkbox(value, "");
                                                                            }
                                                                            quick_search_lib::EntryType::Int { ref mut value, min, max } => match (min.into_rust(), max.into_rust()) {
                                                                                (Some(min), Some(max)) => {
                                                                                    ui.add(egui::Slider::new(value, min..=max));
                                                                                }
                                                                                _ => {
                                                                                    ui.label("no range provided, refer to the documentation for this plugin and configure it manually in the config file.");
                                                                                }
                                                                            },
                                                                            quick_search_lib::EntryType::Float { ref mut value, min, max } => match (min.into_rust(), max.into_rust()) {
                                                                                (Some(min), Some(max)) => {
                                                                                    ui.add(egui::Slider::new(value, min..=max));
                                                                                }
                                                                                _ => {
                                                                                    ui.label("no range provided, refer to the documentation for this plugin and configure it manually in the config file.");
                                                                                }
                                                                            },
                                                                            quick_search_lib::EntryType::String { ref mut value } => {
                                                                                let mut this = value.clone().into_rust();
                                                                                if ui.text_edit_singleline(&mut this).changed() {
                                                                                    if let Ok(rstr) = RString::from_str(&this) {
                                                                                        *value = rstr;
                                                                                    }
                                                                                };

                                                                            }
                                                                            _ => {
                                                                                ui.label("not implemented, refer to the documentation for this plugin and configure it manually in the config file.");
                                                                            }
                                                                        }
                                                                    });
                                                                    ui.separator();
                                                                }
                                                                if ui.button("Close").clicked() {
                                                                    self.menu_open_for = None;
                                                                }
                                                            });
                                                    } else {
                                                        // dummy comment
                                                        if ui
                                                            .add(Button::new(RichText::new(&*name).color(Color32::GREEN)).wrap(false))
                                                            .on_hover_cursor(egui::CursorIcon::Alias)
                                                            .on_hover_text("Plugin has extra configurations")
                                                            .clicked()
                                                        {
                                                            log::trace!("Open menu for {}", i);
                                                            self.menu_open_for = Some(i);
                                                        }
                                                    }
                                                } else {
                                                    ui.add(Label::new(&*name).wrap(false));
                                                }
                                            });
                                            row.col(|ui| {
                                                ui.checkbox(&mut state.enabled, "");
                                            });
                                            row.col(|ui| {
                                                ui.add(egui::Slider::new(&mut state.priority, 0..=128));
                                            });
                                            row.col(|ui| {
                                                ui.add(egui::Slider::new(&mut state.delay, 0..=10000));
                                            });
                                        })
                                    }
                                }
                            });
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Cancel").color(Color32::RED)).clicked() {
                            self.close_at_end = CloseState::CloseNoSave;
                        }
                        ui.spacing();
                        if ui.button(RichText::new("Save").color(Color32::GREEN)).clicked() {
                            self.close_at_end = CloseState::CloseSave;
                        }
                        if !self.loadresults.errors.is_empty() {
                            ui.spacing();
                            ui.label(RichText::new("Errors found while loading plugins").color(Color32::RED)).on_hover_ui(|ui| {
                                ui.vertical(|ui| {
                                    for (i, (path, error)) in self.loadresults.errors.iter().enumerate() {
                                        if i != 0 {
                                            ui.separator();
                                        }
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(path).color(Color32::RED));
                                            ui.separator();
                                            ui.label(RichText::new(error).color(Color32::LIGHT_RED));
                                        });
                                    }
                                });
                            });
                        }
                    })
                });
        }

        let newpassthru = egui_context.wants_pointer_input();

        if newpassthru != self.passthrough {
            self.passthrough = newpassthru;
            if self.passthrough {
                glfw_backend.window.set_mouse_passthrough(false);
            } else {
                glfw_backend.window.set_mouse_passthrough(true);
            }
            self.force_redraw_now = true;
        }

        if self.force_redraw_now {
            egui_context.request_repaint();
        } else {
            egui_context.request_repaint_after(std::time::Duration::from_millis(100));
        }
        self.force_redraw_now = false;

        match self.close_at_end {
            CloseState::DoNothing => {}
            CloseState::CloseNoSave => {
                glfw_backend.window.set_should_close(true);
            }
            CloseState::CloseSave => {
                self.config_lock.get_mut().plugin_states = self.states.clone().into_iter().collect::<HashMap<String, PluginConfig>>();
                // config_lock.get_mut() trips a flag within the config_lock that makes it save the config file
                glfw_backend.window.set_should_close(true);
            }
        }
    }
}

fn nowrap_heading(text: &str) -> Label {
    Label::new(RichText::new(text).heading()).wrap(false)
}
