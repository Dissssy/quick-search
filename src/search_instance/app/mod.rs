use std::collections::HashSet;

use egui::RichText;
use quick_search_lib::SearchResult;
mod holder;
use crate::config::ConfigLock;

use self::holder::NiceIter;

use super::{PluginLoadResult, SearchMetadata};
use holder::ResultHolder;

pub struct App<'a> {
    config_lock: ConfigLock<'a>,

    audio: Option<rusty_audio::Audio>,
    size: Option<egui::Vec2>,
    positioned: bool,

    // search
    input: String,
    selected: bool,
    // results: ResultHolder,
    // last_changed: Option<std::time::Instant>,
    doubledown: bool,
    doubleup: bool,
    scrolling: bool,

    // search threading
    // oldhandles: Vec<std::thread::JoinHandle<(Vec<SearchResult>, SearchMetadata)>>,
    // joinhandles: Vec<std::thread::JoinHandle<(Vec<SearchResult>, SearchMetadata)>>,
    // clear_next: bool,
    force_redraw_now: bool,

    // results stuff
    passthrough: bool,

    time: std::time::Instant,

    searchholder: SearchHolder,
}

impl App<'_> {
    pub fn new(loadresults: PluginLoadResult) -> Self {
        let config_lock = crate::CONFIG_FILE.lock();
        Self {
            searchholder: SearchHolder::new(loadresults),
            audio: if config_lock.get().audio_enabled {
                let mut audio = rusty_audio::Audio::new();
                audio.add("notif", crate::AUDIO_FILE_PATH.clone());
                Some(audio)
            } else {
                None
            },
            config_lock,
            size: None,
            positioned: bool::default(),
            input: String::default(),
            selected: bool::default(),
            // results: ResultHolder::default(),
            // last_changed: Option::default(),
            doubledown: bool::default(),
            doubleup: bool::default(),
            scrolling: bool::default(),
            // oldhandles: Vec::default(),
            // joinhandles: Vec::default(),
            // clear_next: bool::default(),
            force_redraw_now: bool::default(),
            passthrough: bool::default(),
            time: std::time::Instant::now(),
        }
    }
    // pub fn try_dispatch_search(&mut self) -> anyhow::Result<()> {
    //     // check old handles
    //     let mut newoldhandles = vec![];
    //     for handle in self.oldhandles.drain(..) {
    //         if !handle.is_finished() {
    //             newoldhandles.push(handle);
    //         }
    //     }
    //     self.oldhandles = newoldhandles;

    //     if self.oldhandles.len() > 32 {
    //         log::error!("oldhandles is too big! not spawning new thread until it is cleared");
    //         return Ok(());
    //     }

    //     if self.input.is_empty() {
    //         self.results.clear();
    //         self.joinhandles.clear();
    //     } else {
    //         // drain the old handles to oldhandles
    //         // self.joinhandles.clear();
    //         for handle in self.joinhandles.drain(..) {
    //             self.oldhandles.push(handle);
    //         }

    //         self.clear_next = true;
    //         // let searches = {
    //         //     let mut searches = vec![];
    //         //     for search in &self.searches {
    //         //         searches.push(search.copy());
    //         //     }
    //         //     searches
    //         // };
    //         // let input = self.input.clone();
    //         // self.joinhandle = Some(std::thread::spawn(move || App::search(searches, input)));

    //         for search in &self.loadresults.plugins {
    //             let input = self.input.to_lowercase();
    //             self.joinhandles.push(search.search_delayed(&input));
    //         }
    //     }

    //     Ok(())
    // }

    // pub fn check_results(&mut self) {
    //     // if let Some(handle) = self.joinhandle.take() {
    //     //     // check if the thread is done
    //     //     if handle.is_finished() {
    //     //         // if it is, then get the results
    //     //         if let Ok(results) = handle.join() {
    //     //             // and set them
    //     //             self.results = results;
    //     //         }
    //     //     } else {
    //     //         // if it isn't, then put the handle back
    //     //         self.joinhandle = Some(handle);
    //     //     }
    //     // }

    //     let mut newhandles = vec![];

    //     for handle in self.joinhandles.drain(..) {
    //         if handle.is_finished() {
    //             self.force_redraw_now = true;
    //             if let Ok((results, metadata)) = handle.join() {
    //                 if self.clear_next {
    //                     self.results.clear();
    //                     self.clear_next = false;
    //                 }
    //                 self.results.add_results(results, metadata);
    //             }
    //         } else {
    //             newhandles.push(handle);
    //         }
    //     }

    //     self.joinhandles = newhandles;
    // }
}

impl egui_overlay::EguiOverlay for App<'_> {
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
                        glfw_backend.window.set_title("QuickSearch");
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

                        if current_focus_name != "QuickSearch" {
                            // glfw_backend.window.hide();
                            // glfw_backend.window.show();
                            glfw_backend.window.set_should_close(true);
                        } else if let Some(audio) = &mut self.audio {
                            audio.play("notif");
                        }

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

            let barheight = egui::Window::new("Search")
                .title_bar(false)
                // .fixed_pos(Pos2::new(midwindowx as f32 - 200., midwindowy as f32 - 30.))
                // .fixed_size(Vec2::new(400., 60.))
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0., 0.))
                .show(egui_context, |ui| {
                    let r = ui
                        .vertical_centered(|ui| {
                            if self.config_lock.get().clock_enabled {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(
                                            chrono::Utc::now()
                                                .with_timezone(&self.config_lock.get().timezone)
                                                .format(&self.config_lock.get().chrono_format_string)
                                                .to_string(),
                                        )
                                        .size(self.config_lock.get().time_font_size)
                                        .color(egui::Color32::LIGHT_RED),
                                    )
                                    .wrap(false),
                                );
                                ui.separator();
                            }

                            let textinput = egui::TextEdit::singleline(&mut self.input).vertical_align(egui::Align::Center).horizontal_align(egui::Align::Center);

                            let mut size = ui.available_size();

                            size.y = size.y.min(20.0);
                            // size.x = size.x.min(20.0);

                            ui.add_sized(size, textinput)
                        })
                        .inner;

                    // if self.passthrough {
                    //     r.surrender_focus();
                    // }

                    if !self.selected || self.searchholder.results.is_empty() {
                        r.request_focus();
                        self.selected = true;
                    }

                    // if self.joinhandles.is_empty() {
                    // }
                    self.searchholder.dispatch(&self.config_lock, &self.input);

                    if r.changed() {
                        // log::!("input changed!");
                        // if let Err(e) = self.try_dispatch_search() {
                        //     log::!("error: {}", e);
                        // }
                        // self.last_changed = Some(std::time::Instant::now());
                        self.searchholder.input_changed();
                        self.doubledown = false;
                        self.doubleup = false;
                    }
                    // } else if let Some(changed) = self.last_changed {
                    //     // if it has been x ms since the last change, then dispatch the search and set last_changed to None
                    //     if std::time::Instant::now().duration_since(changed).as_millis() >= self.config_lock.get().total_search_delay as u128 {
                    //         log::trace!("input not changed, dispatching search!");
                    //         if let Err(e) = self.try_dispatch_search() {
                    //             log::error!("error: {}", e);
                    //         }
                    //         self.last_changed = None;
                    //     }
                    // }

                    if egui_context.input(|i| i.key_pressed(egui::Key::ArrowDown)) || (egui_context.input(|i| i.raw_scroll_delta.y < 0.0) && self.scrolling) {
                        log::trace!("arrow down pressed!");

                        if self.doubledown {
                            self.doubledown = false;
                            self.scrolling = true;
                            r.surrender_focus();
                            // self.index = self.results.len().saturating_sub(1);
                            self.searchholder.results.clear_cursor();
                            self.searchholder.results.decrement_cursor();
                        }

                        if self.scrolling {
                            // self.index += 1;
                            self.searchholder.results.increment_cursor();
                            // if self.index >= self.results.len() {
                            //     self.index = 0;
                            // }
                        } else {
                            self.doubledown = true;
                        }
                    }

                    if egui_context.input(|i| i.key_pressed(egui::Key::ArrowUp)) || (egui_context.input(|i| i.raw_scroll_delta.y > 0.0) && self.scrolling) {
                        log::trace!("arrow up pressed!");

                        if self.doubleup {
                            self.doubleup = false;
                            self.scrolling = true;
                            r.surrender_focus();
                            // self.index = 0;
                            self.searchholder.results.clear_cursor();
                        }

                        if self.scrolling {
                            // if self.index == 0 {
                            //     self.index = self.results.len().saturating_sub(1)
                            // } else {
                            //     self.index -= 1;
                            // }
                            r.surrender_focus();
                            self.searchholder.results.decrement_cursor();
                        } else {
                            self.doubleup = true;
                        }
                    }

                    if egui_context.input(|i| i.key_pressed(egui::Key::PageUp)) {
                        log::trace!("page up pressed!");
                        self.searchholder.results.jump_backward(self.scrolling);
                        self.scrolling = true;
                    }

                    if egui_context.input(|i| i.key_pressed(egui::Key::PageDown)) {
                        log::trace!("page down pressed!");
                        self.searchholder.results.jump_forward(self.scrolling);
                        self.scrolling = true;
                    }

                    if egui_context.input(|i| i.key_pressed(egui::Key::Tab)) {
                        log::trace!("Tab pressed!");
                        self.scrolling = !self.scrolling;
                        if self.scrolling {
                            // self.index = 0;
                            self.searchholder.results.clear_cursor();
                        } else {
                            r.request_focus();
                        }
                    }

                    if egui_context.input(|i| i.key_pressed(egui::Key::Enter)) {
                        log::trace!("enter pressed!");

                        if !self.scrolling {
                            // if the text lost focus, we should switch to scrolling mode
                            if r.lost_focus() {
                                self.scrolling = true;
                            }
                        } else {
                            // if enter was pressed while scrolling, we should use the selected result and close the window
                            // get result at index and call action
                            if let Some((result, plugin_id)) = self.searchholder.results.get_from_cursor() {
                                for plugin in &self.searchholder.loadresults.plugins {
                                    if plugin.id == plugin_id {
                                        plugin.execute(result);
                                    }
                                }
                            }

                            // close the window
                            glfw_backend.window.set_should_close(true);
                        }
                    }

                    if egui_context.input(|i| i.key_pressed(egui::Key::Escape)) {
                        log::trace!("escape pressed!");
                        // close the window
                        glfw_backend.window.set_should_close(true);
                    };

                    egui_context.used_size().x
                })
                .map(|x| x.response.rect.height())
                .unwrap_or(0.0);

            if !self.searchholder.results.is_empty() {
                let mut set_cursor_later = None;

                egui::Window::new("Results")
                    .title_bar(false)
                    // .fixed_pos(Pos2::new(midwindowx as f32, midwindowy as f32))
                    // .fixed_size(Vec2::new(400., 60.))
                    .resizable(false)
                    // .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0., midwindowy as f32 + 30.))
                    .pivot(egui::Align2::CENTER_TOP)
                    .fixed_pos(egui::Pos2::new(
                        midwindowx as f32,
                        midwindowy as f32 + (barheight / 2.0) + self.config_lock.get().gap_between_search_bar_and_results,
                    ))
                    .show(egui_context, |ui| {
                        // if !self.joinhandles.is_empty() {
                        //     ui.spinner();
                        // } else if self.config_lock.get().show_countdown {
                        //     if let Some(changed) = self.last_changed {
                        //         let dur_since = std::time::Instant::now().duration_since(changed).as_millis();
                        //         let delay = self.config_lock.get().total_search_delay as u128;
                        //         if dur_since < delay {
                        //             ui.label(format!("{:.2} seconds until search", delay.saturating_sub(dur_since) as f32 / 1000.));
                        //         }
                        //     }
                        // }
                        // let mut last_result = None;
                        // for (i, result) in self.results.iter_all().enumerate() {
                        //     let dont_realloc = result.source.name();
                        //     if if let Some((last_result_name, _)) = last_result.as_mut() {
                        //         let b = *last_result_name != dont_realloc;
                        //         if b {
                        //             ui.separator();
                        //         }
                        //         b
                        //     } else {
                        //         true
                        //     } {
                        //         // let mut t = RichText::new(&dont_realloc);
                        //         last_result = Some((dont_realloc, i));

                        //         // if let Some(color) =  {
                        //         //     t = t.color(color);
                        //         // }

                        //         ui.add(egui::Label::new(result.source.colored_name()).wrap(false));
                        //         ui.separator();
                        //     }

                        //     let last_index = match last_result {
                        //         Some((_, index)) => index,
                        //         None => 1,
                        //     } - 1;

                        //     // if we are not scrolling, only display the first 2 results for each source
                        //     if !self.scrolling && (i - last_index) >= 2 {
                        //         continue;
                        //     }

                        //     // if we are scrolling, and self.index is greater than last_index, only display the 7 results centered around self.index, throwing away if out of bounds

                        //     if self.scrolling && /*(i < self.index.saturating_sub(2) || i > self.index + 2)*/ !self.results.cursor_range().contains(&i) {
                        //         continue;
                        //     }

                        //     if
                        //     /*i == self.index*/
                        //     self.results.cursor_on(i) && self.scrolling {
                        //         ui.add(egui::Label::new(RichText::new(&result.name).color(egui::Color32::LIGHT_BLUE)).wrap(false));
                        //         if let Some(ref context) = result.context {
                        //             ui.add(egui::Label::new(RichText::new(context).color(egui::Color32::from_rgb(0, 128, 255))).wrap(false));
                        //         }
                        //     } else {
                        //         ui.add(egui::Label::new(&result.name).wrap(false));
                        //     };
                        // }

                        for e in self
                            .searchholder
                            .results
                            .iter_nice(self.scrolling, self.config_lock.get().entries_around_cursor, self.config_lock.get().group_entries_while_unselected)
                        {
                            match e {
                                NiceIter::NewSource(source) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::Label::new(source.pretty_name.clone()).wrap(false));
                                        ui.separator();
                                        ui.add(egui::Label::new(format!("{} Results", source.num_results)).wrap(false));
                                    });
                                    ui.separator();
                                }
                                NiceIter::Result { result, cursor_on, index } => {
                                    let (short_title, title_truncated) = {
                                        let mut title = result.title().to_string();
                                        let mut truncated = false;
                                        if title.len() > self.config_lock.get().truncate_title_length {
                                            title.truncate(self.config_lock.get().truncate_title_length - 3);
                                            title.push_str("...");
                                            truncated = true;
                                        }
                                        (title, truncated)
                                    };

                                    let (short_context, context_truncated) = {
                                        let mut context = result.context().to_string();
                                        let mut truncated = false;
                                        if context.len() > self.config_lock.get().truncate_context_length {
                                            context.truncate(self.config_lock.get().truncate_context_length - 3);
                                            context.push_str("...");
                                            truncated = true;
                                        }
                                        (context, truncated)
                                    };

                                    if cursor_on {
                                        ui.add(egui::Label::new(RichText::new(short_title).color(egui::Color32::LIGHT_BLUE)).wrap(false));
                                        if !result.context().is_empty() {
                                            ui.add(egui::Label::new(RichText::new(short_context).color(egui::Color32::from_rgb(0, 128, 255))).wrap(false));
                                        }
                                        if context_truncated || title_truncated {
                                            // like the current result window but above the search bar
                                            egui::Window::new("Full Result")
                                                .title_bar(false)
                                                // .fixed_pos(Pos2::new(midwindowx as f32, midwindowy as f32))
                                                // .fixed_size(Vec2::new(400., 60.))
                                                .resizable(false)
                                                .pivot(egui::Align2::CENTER_BOTTOM)
                                                .fixed_pos(egui::Pos2::new(
                                                    midwindowx as f32,
                                                    midwindowy as f32 - (barheight / 2.0) - self.config_lock.get().gap_between_search_bar_and_results,
                                                ))
                                                .min_size(egui::Vec2::new(400., 60.))
                                                // .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0., midwindowy as f32 - 30.))
                                                .show(egui_context, |ui| {
                                                    if title_truncated {
                                                        ui.add(egui::Label::new(RichText::new(result.title()).heading().color(egui::Color32::LIGHT_BLUE)).wrap(true));
                                                    }
                                                    if title_truncated && context_truncated {
                                                        ui.separator();
                                                    }
                                                    if context_truncated {
                                                        ui.add(egui::Label::new(RichText::new(result.context()).color(egui::Color32::from_rgb(0, 128, 255))).wrap(true));
                                                    }
                                                });
                                        }
                                    } else if self.scrolling {
                                        ui.add(egui::Label::new(short_title).wrap(false));
                                    } else {
                                        // i dont like inlining if statements if there are side effects
                                        if ui.add_enabled(true, egui::Button::new(short_title).wrap(false).frame(false)).clicked() {
                                            set_cursor_later = Some(index);
                                        }
                                    };
                                }
                            }
                        }

                        egui_context.used_size().x
                    });
                if let Some(index) = set_cursor_later {
                    self.searchholder.results.raw_set_cursor(index);
                    self.scrolling = true;
                }
            }
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
    }
}

pub struct SearchHolder {
    loadresults: PluginLoadResult,
    joinhandles: Vec<std::thread::JoinHandle<(Vec<SearchResult>, SearchMetadata)>>,
    oldhandles: Vec<std::thread::JoinHandle<(Vec<SearchResult>, SearchMetadata)>>,
    last_changed: Option<std::time::Instant>,
    dispatched_searches: HashSet<String>,
    results: ResultHolder,
}

impl SearchHolder {
    pub fn new(loadresults: PluginLoadResult) -> Self {
        Self {
            loadresults,
            joinhandles: Vec::default(),
            oldhandles: Vec::default(),
            last_changed: Option::default(),
            dispatched_searches: HashSet::default(),
            results: ResultHolder::default(),
        }
    }
    pub fn input_changed(&mut self) {
        self.last_changed = Some(std::time::Instant::now());
        self.dispatched_searches.clear();
        self.oldhandles.append(&mut self.joinhandles);
        self.results.clear();
    }
    pub fn dispatch(&mut self, config: &ConfigLock<'_>, input: &str) {
        let config = config.get();
        self.oldhandles.retain(|handle| !handle.is_finished());

        let time_since_last_change = self
            .last_changed
            .map(|changed| std::time::Instant::now().duration_since(changed).as_millis())
            .unwrap_or(0)
            .saturating_sub(config.total_search_delay as u128);

        if !input.is_empty() {
            for plugin in self.loadresults.plugins.iter() {
                // if it has been long enough since the last change, and the search has not been dispatched, then dispatch the search
                if (config.get_plugin(plugin.name).map(|p| p.delay).unwrap_or(100) as u128) < time_since_last_change && !self.dispatched_searches.contains(plugin.name) {
                    log::trace!("dispatching search for {} after {}ms", plugin.name, time_since_last_change);
                    self.joinhandles.push(plugin.search_delayed(input));
                    self.dispatched_searches.insert(plugin.name.to_string());
                }
            }
        }

        let mut newhandles = vec![];

        for handle in self.joinhandles.drain(..) {
            if handle.is_finished() {
                if let Ok((r, m)) = handle.join() {
                    log::trace!("search thread finished for {} with {} results", m.raw_name, r.len());
                    if !r.is_empty() {
                        self.results.add_results(r, m);
                    }
                } else {
                    log::error!("search thread failed");
                }
            } else {
                newhandles.push(handle);
            }
        }

        self.joinhandles = newhandles;
    }
}
