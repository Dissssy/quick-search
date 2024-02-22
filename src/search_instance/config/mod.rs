use std::collections::HashMap;

use egui::{Color32, Label, RichText};
use egui_extras::{Column, TableBuilder};

use crate::config::{ConfigLock, PluginConfig};

pub struct App<'a> {
    config_lock: ConfigLock<'a>,
    audio_enabled: bool,
    states: Vec<(String, PluginConfig)>,
    size: Option<egui::Vec2>,
    positioned: bool,
    passthrough: bool,
    force_redraw_now: bool,
    close_at_end: CloseState,
    time: std::time::Instant,
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
    pub fn new() -> Self {
        let config_lock = crate::CONFIG_FILE.lock();
        let mut states: Vec<(String, PluginConfig)> = config_lock.get().plugin_states.iter().map(|(k, v)| (k.clone(), *v)).collect();
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
            audio_enabled: config_lock.get().audio_enabled,
            states,
            config_lock,
            size: None,
            positioned: false,
            passthrough: false,
            force_redraw_now: false,
            close_at_end: CloseState::DoNothing,
            time: std::time::Instant::now(),
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

                        // std::thread::sleep(std::time::Duration::from_millis(100));

                        let current_focus_name = unsafe {
                            let current = winapi::um::winuser::GetForegroundWindow();
                            let mut window_title = [0u16; 1024];
                            let len = winapi::um::winuser::GetWindowTextW(current, window_title.as_mut_ptr(), window_title.len() as i32);
                            let current_name = String::from_utf16_lossy(&window_title[..len as usize]);
                            log::info!("current window: {}", current_name);
                            current_name
                        };

                        if current_focus_name != "glfw window" {
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
        } else if self.time.elapsed().as_millis() > crate::DELAY_TUNING {
            if let Some(size) = self.size {
                if !self.positioned {
                    glfw_backend.window.set_pos(0, 0);
                    glfw_backend.window.set_size(size.x as i32 - 1, size.y as i32 - 1);
                    self.positioned = true;
                }
            }

            // let (_midwindowx, midwindowy) = {
            //     let (x, y) = glfw_backend.window.get_size();
            //     (x / 2, y / 2)
            // };

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
                    ui.checkbox(&mut self.audio_enabled, "Sound effects")
                        .on_hover_text("Enable or disable sound effects when the search bar is opened");
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
                    ui.separator();
                    // todo: plugin state configs
                    if self.states.is_empty() {
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
                            .column(Column::remainder())
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.add(nowrap_heading("Plugin"));
                                });
                                header.col(|ui| {
                                    ui.add(nowrap_heading("Enabled"));
                                });
                                header.col(|ui| {
                                    ui.add(nowrap_heading("Priority"));
                                });
                            })
                            .body(|mut body| {
                                for (name, state) in self.states.iter_mut() {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            ui.add(Label::new(&*name).wrap(false));
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut state.enabled, "");
                                        });
                                        row.col(|ui| {
                                            ui.add(egui::Slider::new(&mut state.priority, 0..=128));
                                        });
                                    })
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
                self.config_lock.get_mut().audio_enabled = self.audio_enabled;
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
