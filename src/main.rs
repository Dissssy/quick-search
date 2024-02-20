#![windows_subsystem = "windows"]
use std::sync::{Arc, Mutex};

use anyhow::Result;
use egui::{Context, RichText, Vec2};
// use egui_overlay::egui_window_glfw_passthrough::glfw::Context as _;
use search::{SearchResult, Searchable};
use windows_hotkeys::HotkeyManagerImpl;
// use winapi::um::winuser::SetForegroundWindow;

mod search;

fn main() {
    // privelege level, its debugging shit sorrrryyy
    // search::set_clipboard(format!("privelege level: {:?}\nis_elevated: {}", privilege_level::privilege_level(), is_elevated::is_elevated()).as_str());

    // listen for F17 keypress from the keyboard
    let mut hkm = windows_hotkeys::HotkeyManager::new();

    let thread: Arc<Mutex<Option<std::thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    let software_lock = Arc::new(match minwin::sync::Mutex::create_named("Dissy-Quick-search") {
        Ok(lock) => lock,
        Err(e) => {
            println!("Software already running: {}", e);
            return;
        }
    });

    hkm.register(windows_hotkeys::keys::VKey::F17, &[], move || {
        let software_lock = software_lock.clone();
        let thread = thread.clone();
        println!("F17 pressed!");
        match thread.lock() {
            Ok(mut threadopt) => {
                if let Some(thread) = threadopt.take() {
                    if !thread.is_finished() {
                        let _ = threadopt.insert(thread);
                        return;
                    }
                }
                // if we make it here the thread either DOESNT exist, or it is finished
                *threadopt = Some(std::thread::spawn(move || {
                    if let Err(e) = software_lock.lock() {
                        println!("error: {}", e);
                    } else {
                        fakemain();
                        if let Err(e) = software_lock.release() {
                            println!("error: {}", e);
                        }
                    }
                }));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    })
    .expect("failed to register hotkey");

    // let (killer, killee) = std::sync::mpsc::channel();

    let hkm_thread = std::thread::spawn(move || hkm.event_loop());
    // let watcher_thread = std::thread::spawn(move || {
    //     let exe_location = std::path::PathBuf::from("C:\\Users\\ethan\\Desktop\\quick-search\\target\\release\\quick-search.exe")
    //         .canonicalize()
    //         .unwrap();
    //     // watch for changes, copy to current exe location, and restart

    // });

    hkm_thread.join().unwrap();
}

fn fakemain() {
    // let softwarelock = unsafe {
    //     let mut sec = winapi::um::minwinbase::SECURITY_ATTRIBUTES {
    //         nLength: std::mem::size_of::<winapi::um::minwinbase::SECURITY_ATTRIBUTES>() as u32,
    //         lpSecurityDescriptor: std::ptr::null_mut(),
    //         bInheritHandle: 0,
    //     };

    //     let sec_ptr = &mut sec as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES;
    //     let name = b"Dissy-Quick-search".as_ptr() as *mut i8;

    //     winapi::um::synchapi::CreateMutexA(sec_ptr, 0, name)
    // };

    // let _softwarelock = SoftwareLock::new("Dissy-Quick-search").unwrap();

    let mut audio = rusty_audio::Audio::new();

    let path = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().parent().unwrap().join("assets").join("notif.mp3");

    println!("path: {:?}", path);

    audio.add("notif", path);

    let mut app = App::default();
    app.add_player(audio);
    app.add_search(Box::new(search::Math));
    app.add_search(Box::new(search::Everything));
    app.add_search(Box::new(search::Google));
    app.add_search(Box::new(search::Dictionary));
    app.add_search(Box::new(search::UrbanDictionary));
    app.add_search(Box::new(search::WindowsApps));
    egui_overlay::start(app);
}

// struct SoftwareLock {
//     handle: *mut c_void,
// }

// impl SoftwareLock {
//     pub fn new(name: &str) -> Result<Self> {
//         let mut sec = winapi::um::minwinbase::SECURITY_ATTRIBUTES {
//             nLength: std::mem::size_of::<winapi::um::minwinbase::SECURITY_ATTRIBUTES>() as u32,
//             lpSecurityDescriptor: std::ptr::null_mut(),
//             bInheritHandle: 0,
//         };

//         let sec_ptr = &mut sec as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES;
//         let name = name.as_bytes().as_ptr() as *mut i8;

//         let handle = unsafe { winapi::um::synchapi::CreateMutexA(sec_ptr, 0, name) };

//         if handle.is_null() {
//             Err(anyhow::anyhow!("failed to create mutex"))
//         } else {
//             Ok(Self { handle })
//         }
//     }
// }

// impl Drop for SoftwareLock {
//     fn drop(&mut self) {
//         unsafe {
//             winapi::um::synchapi::ReleaseMutex(self.handle);
//         }
//     }
// }

#[derive(Default)]
pub struct App {
    input: String,
    last_changed: Option<std::time::Instant>,
    size: Option<Vec2>,
    positioned: bool,
    selected: bool,
    passthrough: bool,
    results: SearchResults,
    searches: Vec<Box<dyn Searchable>>,
    joinhandles: Vec<std::thread::JoinHandle<Vec<SearchResult>>>,
    oldhandles: Vec<std::thread::JoinHandle<Vec<SearchResult>>>,
    // index: usize,
    scrolling: bool,
    clear_next: bool,
    doubledown: bool,
    doubleup: bool,
    audio_player: Option<rusty_audio::prelude::Audio>,
    force_redraw_now: bool,
}

#[derive(Default)]
struct SearchResults {
    cursor: usize,
    groups: Vec<SearchResultGroup>,
}

impl SearchResults {
    fn clear(&mut self) {
        self.cursor = 0;
        self.groups.clear();
    }

    fn add_results(&mut self, results: Vec<SearchResult>) {
        let mut group = SearchResultGroup {
            // source: match results.first() {
            //     Some(result) => result.source().copy(),
            //     None => return,
            // },
            results,
        };

        group.results.sort_by_key(|b| std::cmp::Reverse(b.source().priority()));

        self.groups.push(group);
    }

    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    fn len(&self) -> usize {
        self.groups.iter().map(|g| g.results.len()).sum()
    }

    fn get_from_cursor(&mut self) -> Option<&mut SearchResult> {
        if self.groups.is_empty() {
            return None;
        }

        let mut cursor = self.cursor;

        for group in self.groups.iter_mut() {
            if cursor < group.results.len() {
                return group.results.get_mut(cursor);
            } else {
                cursor -= group.results.len();
            }
        }

        None
    }

    // fn iter_all(&self) -> impl Iterator<Item = &SearchResult> {
    //     self.groups.iter().flat_map(|g| g.results.iter())
    // }

    fn increment_cursor(&mut self) {
        self.cursor += 1;
        if self.cursor >= self.len() {
            self.cursor = 0;
        }
    }

    fn decrement_cursor(&mut self) {
        if self.cursor == 0 {
            self.cursor = self.len().saturating_sub(1);
        } else {
            self.cursor -= 1;
        }
    }

    fn clear_cursor(&mut self) {
        self.cursor = 0;
    }

    fn _get_cursor(&self) -> usize {
        self.cursor
    }

    fn jump_backward(&mut self, selected: bool) {
        // we want to set self.cursor to the last result of the previous group
        // if there is no previous group, set self.cursor to the last result of the last group

        if selected {
            let mut cursor = None;

            let mut done = false;

            let mut end_of_final_group = 0;

            for (i, (y, _)) in self.groups.iter().flat_map(|g| g.results.iter().enumerate()).enumerate() {
                // i is the index in GLOBAL SPACE
                // the cursor is in GLOBAL SPACE
                // we need to detect the bounds of the previous group, so cursor will be the last time we crossed a group boundary
                // once we cross self.cursor for i, we can break

                if y == 0 && !done {
                    cursor = i.checked_sub(1);
                }
                if i == self.cursor {
                    done = true;
                }
                end_of_final_group = i;
            }

            self.cursor = cursor.unwrap_or(end_of_final_group);
        } else {
            // not selected, jump to final result of last group
            self.cursor = self.groups.iter().flat_map(|g| g.results.iter()).count().saturating_sub(1);
        }
    }

    fn jump_forward(&mut self, selected: bool) {
        // we want to set self.cursor to the first result of the next group
        // if there is no next group, set self.cursor to the first result of the first group

        if selected {
            let mut cursor = None;
            let mut break_next = false;

            for (i, (y, _)) in self.groups.iter().flat_map(|g| g.results.iter().enumerate()).enumerate() {
                // i is the index in GLOBAL SPACE
                // the cursor is in GLOBAL SPACE
                // we need to detect the bounds of the next group, so cursor will be the first time we cross a group boundary
                // once we cross self.cursor for i, we can break

                if y == 0 && break_next {
                    println!("breaking at {}", i);
                    cursor = Some(i);
                    break;
                }
                if i == self.cursor {
                    break_next = true;
                }
            }

            self.cursor = cursor.unwrap_or(0);
        } else {
            // not selected, jump to first result of first group
            self.cursor = 0;
        }
    }

    fn cursor_range(&self) -> std::ops::Range<usize> {
        let mut start = self.cursor.saturating_sub(2);
        let mut end = self.cursor.saturating_add(3);
        let mut range = start..end;
        let mut last_source = "".to_owned();
        for (x, (_y, g)) in self.groups.iter().flat_map(|g| g.results.iter().map(|e| e.source().copy()).enumerate()).enumerate() {
            if range.contains(&x) && last_source != g.name() {
                last_source = g.name();
                if x <= self.cursor {
                    start = x;
                } else {
                    end = x;
                    break;
                }
            }
            range = start..end;
        }
        start..end
    }

    // fn cursor_on(&self, i: usize) -> bool {
    //     self.cursor == i
    // }

    fn iter_nice(&self, selected: bool) -> impl Iterator<Item = NiceIter> {
        // iterate over all groups
        // yield a NewSource every time the source changes
        // yield the first 3 results from each group UNLESS we are selected AND the cursor is in that group, then yield the 7 results centered around the cursor (from the self.cursor_range() function)

        // let mut vec = vec![];

        // for (i, (y, result)) in self.groups.iter().flat_map(|g| g.results.iter().enumerate()).enumerate() {
        //     if y == 0 {
        //         vec.push(NiceIter::NewSource(result.source.copy()));
        //     }
        //     if selected {
        //         // we have to handle the range checks
        //     } else {
        //         if y < 3 {
        //             vec.push(NiceIter::Result { result, cursor_on: false });
        //         }
        //     }
        // }

        // vec.iter()

        self.groups.iter().flat_map(|g| g.results.iter().enumerate()).enumerate().flat_map(move |(i, (y, result))| {
            let res = if selected {
                if self.cursor_range().contains(&i) {
                    Some(NiceIter::Result {
                        result,
                        cursor_on: { self.cursor == i },
                    })
                } else {
                    None
                }
            } else if y < 3 {
                Some(NiceIter::Result { result, cursor_on: false })
            } else {
                None
            };

            if y == 0 {
                if let Some(res) = res {
                    vec![NiceIter::NewSource(result.source().copy()), res]
                } else {
                    vec![NiceIter::NewSource(result.source().copy())]
                }
            } else if let Some(res) = res {
                vec![res]
            } else {
                vec![]
            }
        })
    }
}

enum NiceIter<'a> {
    NewSource(Box<dyn Searchable>),
    Result { result: &'a SearchResult, cursor_on: bool },
}

struct SearchResultGroup {
    // source: Box<dyn Searchable>,
    results: Vec<SearchResult>,
}

impl App {
    pub fn add_search(&mut self, search: Box<dyn Searchable>) {
        self.searches.push(search);
    }

    pub fn try_dispatch_search(&mut self) -> Result<()> {
        // check old handles
        let mut newoldhandles = vec![];
        for handle in self.oldhandles.drain(..) {
            if !handle.is_finished() {
                newoldhandles.push(handle);
            }
        }
        self.oldhandles = newoldhandles;

        if self.oldhandles.len() > 32 {
            println!("oldhandles is too big! not spawning new thread until it is cleared");
            return Ok(());
        }

        if self.input.is_empty() {
            self.results.clear();
            self.joinhandles.clear();
        } else {
            // drain the old handles to oldhandles
            // self.joinhandles.clear();
            for handle in self.joinhandles.drain(..) {
                self.oldhandles.push(handle);
            }

            self.clear_next = true;
            // let searches = {
            //     let mut searches = vec![];
            //     for search in &self.searches {
            //         searches.push(search.copy());
            //     }
            //     searches
            // };
            // let input = self.input.clone();
            // self.joinhandle = Some(std::thread::spawn(move || App::search(searches, input)));

            for search in &self.searches {
                let input = self.input.to_lowercase();
                let search = search.copy();
                self.joinhandles.push(std::thread::spawn(move || search.search(&input)));
            }
        }

        Ok(())
    }

    pub fn check_results(&mut self) {
        // if let Some(handle) = self.joinhandle.take() {
        //     // check if the thread is done
        //     if handle.is_finished() {
        //         // if it is, then get the results
        //         if let Ok(results) = handle.join() {
        //             // and set them
        //             self.results = results;
        //         }
        //     } else {
        //         // if it isn't, then put the handle back
        //         self.joinhandle = Some(handle);
        //     }
        // }

        let mut newhandles = vec![];

        for handle in self.joinhandles.drain(..) {
            if handle.is_finished() {
                self.force_redraw_now = true;
                if let Ok(results) = handle.join() {
                    if self.clear_next {
                        self.results.clear();
                        self.clear_next = false;
                    }
                    println!("{:#?}", results);
                    self.results.add_results(results);
                }
            } else {
                newhandles.push(handle);
            }
        }

        self.joinhandles = newhandles;
    }

    fn add_player(&mut self, audio: rusty_audio::prelude::Audio) {
        self.audio_player = Some(audio);
    }

    // fn search_all(searches: Vec<Box<dyn Searchable>>, input: String) -> Vec<SearchResult> {
    //     let mut res = vec![];

    //     for search in searches {
    //         res.append(&mut search.search(&input));
    //     }

    //     res
    // }
}

impl egui_overlay::EguiOverlay for App {
    fn gui_run(
        &mut self,
        egui_context: &Context,
        _default_gfx_backend: &mut egui_overlay::egui_render_three_d::ThreeDBackend,
        glfw_backend: &mut egui_overlay::egui_window_glfw_passthrough::GlfwBackend,
    ) {
        if self.size.is_none() {
            glfw_backend.glfw.with_connected_monitors(|_glfw, monitors| {
                let monitor = monitors.first();
                match monitor {
                    None => {
                        println!("no monitor");
                    }
                    Some(monitor) => {
                        // this code will literally only run once so we're gonna also request focus
                        // unsafe {
                        //     let window_ptr = egui_overlay::egui_window_glfw_passthrough::glfw::Context::window_ptr(&glfw_backend.window);
                        //     println!("window_ptr: {:p}", window_ptr);
                        //     println!("null: {}", window_ptr.is_null());
                        //     if !window_ptr.is_null() {
                        //         let r = SetForegroundWindow(std::mem::transmute(window_ptr));
                        //         println!("setforegroundwindow: {}", r);
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
                            println!("current window: {}", current_name);
                            current_name
                        };

                        if current_focus_name != "glfw window" {
                            // glfw_backend.window.hide();
                            // glfw_backend.window.show();
                            glfw_backend.window.set_should_close(true);
                        } else if let Some(audio) = &mut self.audio_player {
                            audio.play("notif");
                        }

                        // let (x, y) = monitor.get_physical_size();
                        // let (sx, sy) = monitor.get_content_scale();
                        // println!("monitor size: {}x{}", x, y);
                        // println!("monitor scale: {}x{}", sx, sy);
                        // *v = Some(Vec2::new(x as f32, y as f32));

                        // if let Some(mode) = monitor.get_video_mode() {
                        //     let (x, y) = (mode.width, mode.height);
                        //     println!("monitor size: {}x{}", x, y);
                        //     *v = Some(Vec2::new(x as f32, y as f32));
                        // } // THIS SCREWED UP MY MONITOR LOL

                        let (x1, y1, x2, y2) = monitor.get_workarea();
                        println!("monitor workarea: {}x{} {}x{}", x1, y1, x2, y2);
                        self.size = Some(Vec2::new(x2 as f32, y2 as f32));
                    }
                }
            });
        }

        if let Some(size) = self.size {
            if !self.positioned {
                glfw_backend.window.set_pos(0, 0);
                glfw_backend.window.set_size(size.x as i32 - 1, size.y as i32 - 1);
                self.positioned = true;
            }
        }

        let (_midwindowx, midwindowy) = {
            let (x, y) = glfw_backend.window.get_size();
            (x / 2, y / 2)
        };

        egui_context.set_visuals({
            let mut visuals = egui::Visuals::dark();
            visuals.popup_shadow.extrusion = 0.0;
            visuals.window_shadow.extrusion = 0.0;
            visuals
        });

        egui::Window::new("Search")
            .title_bar(false)
            // .fixed_pos(Pos2::new(midwindowx as f32 - 200., midwindowy as f32 - 30.))
            // .fixed_size(Vec2::new(400., 60.))
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0., 0.))
            .show(egui_context, |ui| {
                let textinput = egui::TextEdit::singleline(&mut self.input).vertical_align(egui::Align::Center).horizontal_align(egui::Align::Center);

                let r = ui.add(textinput);

                // if self.passthrough {
                //     r.surrender_focus();
                // }

                if !self.selected || self.results.is_empty() {
                    r.request_focus();
                    self.selected = true;
                }

                // if self.joinhandles.is_empty() {
                // }
                self.check_results();

                if r.changed() {
                    // println!("input changed!");
                    // if let Err(e) = self.try_dispatch_search() {
                    //     println!("error: {}", e);
                    // }
                    self.last_changed = Some(std::time::Instant::now());
                    self.doubledown = false;
                    self.doubleup = false;
                } else if let Some(changed) = self.last_changed {
                    // if it has been x ms since the last change, then dispatch the search and set last_changed to None
                    if std::time::Instant::now().duration_since(changed).as_millis() >= 300 {
                        println!("input not changed, dispatching search!");
                        if let Err(e) = self.try_dispatch_search() {
                            println!("error: {}", e);
                        }
                        self.last_changed = None;
                    }
                }

                if egui_context.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    println!("arrow down pressed!");

                    if self.doubledown {
                        self.doubledown = false;
                        self.scrolling = true;
                        r.surrender_focus();
                        // self.index = self.results.len().saturating_sub(1);
                        self.results.clear_cursor();
                        self.results.decrement_cursor();
                    }

                    if self.scrolling {
                        // self.index += 1;
                        self.results.increment_cursor();
                        // if self.index >= self.results.len() {
                        //     self.index = 0;
                        // }
                    } else {
                        self.doubledown = true;
                    }
                }

                if egui_context.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    println!("arrow up pressed!");

                    if self.doubleup {
                        self.doubleup = false;
                        self.scrolling = true;
                        r.surrender_focus();
                        // self.index = 0;
                        self.results.clear_cursor();
                    }

                    if self.scrolling {
                        // if self.index == 0 {
                        //     self.index = self.results.len().saturating_sub(1)
                        // } else {
                        //     self.index -= 1;
                        // }
                        self.results.decrement_cursor();
                    } else {
                        self.doubleup = true;
                    }
                }

                if egui_context.input(|i| i.key_pressed(egui::Key::PageUp)) {
                    println!("page up pressed!");
                    self.results.jump_backward(self.scrolling);
                    self.scrolling = true;
                }

                if egui_context.input(|i| i.key_pressed(egui::Key::PageDown)) {
                    println!("page down pressed!");
                    self.results.jump_forward(self.scrolling);
                    self.scrolling = true;
                }

                if egui_context.input(|i| i.key_pressed(egui::Key::Tab)) {
                    println!("Tab pressed!");
                    self.scrolling = !self.scrolling;
                    if self.scrolling {
                        // self.index = 0;
                        self.results.clear_cursor();
                    } else {
                        r.request_focus();
                    }
                }

                if egui_context.input(|i| i.key_pressed(egui::Key::Enter)) {
                    println!("enter pressed!");

                    if !self.scrolling {
                        // if the text lost focus, we should switch to scrolling mode
                        if r.lost_focus() {
                            self.scrolling = true;
                        }
                    } else {
                        // if enter was pressed while scrolling, we should use the selected result and close the window
                        // get result at index and call action
                        if let Some(result) = self.results.get_from_cursor() {
                            result.action();
                        }

                        // close the window
                        glfw_backend.window.set_should_close(true);
                    }
                }

                if egui_context.input(|i| i.key_pressed(egui::Key::Escape)) {
                    println!("escape pressed!");
                    // close the window
                    glfw_backend.window.set_should_close(true);
                };

                egui_context.used_size().x
            });

        // if !self.results.is_empty() {
        // let searchbar_width = if let Some(w) = s.and_then(|s| s.inner) {
        //     w
        // } else {
        //     return;
        // };

        egui::Window::new("Results")
            .title_bar(false)
            // .fixed_pos(Pos2::new(midwindowx as f32, midwindowy as f32))
            // .fixed_size(Vec2::new(400., 60.))
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0., midwindowy as f32 + 30.))
            .show(egui_context, |ui| {
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

                for e in self.results.iter_nice(self.scrolling) {
                    match e {
                        NiceIter::NewSource(source) => {
                            ui.add(egui::Label::new(source.colored_name()).wrap(false));
                            ui.separator();
                        }
                        NiceIter::Result { result, cursor_on } => {
                            if cursor_on {
                                ui.add(egui::Label::new(RichText::new(result.name()).color(egui::Color32::LIGHT_BLUE)).wrap(false));
                                if let Some(context) = result.context() {
                                    ui.add(egui::Label::new(RichText::new(context).color(egui::Color32::from_rgb(0, 128, 255))).wrap(false));
                                }
                            } else {
                                ui.add(egui::Label::new(result.name()).wrap(false));
                            };
                        }
                    }
                }

                egui_context.used_size().x
            });

        // let results_width = if let Some(w) = f.and_then(|f| f.inner) {
        //     w
        // } else {
        //     return;
        // };

        // {
        //     let new_diff = searchbar_width - results_width;
        //     if new_diff != self.width_diff {
        //         self.width_diff = new_diff;
        //         force_redraw_now = true;
        //     }
        // }

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
    // fn run(
    //     &mut self,
    //     egui_context: &Context,
    //     default_gfx_backend: &mut egui_overlay::egui_render_three_d::ThreeDBackend,
    //     glfw_backend: &mut egui_overlay::egui_window_glfw_passthrough::GlfwBackend,
    // ) -> Option<(PlatformOutput, std::time::Duration)> {
    //     let input = glfw_backend.take_raw_input();
    //     // takes a closure that can provide latest framebuffer size.
    //     // because some backends like vulkan/wgpu won't work without reconfiguring the surface after some sort of resize event unless you give it the latest size
    //     default_gfx_backend.prepare_frame(|| {
    //         let latest_size = glfw_backend.window.get_framebuffer_size();
    //         [latest_size.0 as _, latest_size.1 as _]
    //     });
    //     egui_context.begin_frame(input);
    //     self.gui_run(egui_context, default_gfx_backend, glfw_backend);

    //     let egui::FullOutput {
    //         platform_output,
    //         repaint_after,
    //         textures_delta,
    //         shapes,
    //         pixels_per_point,
    //     } = egui_context.end_frame();
    //     let meshes = egui_context.tessellate(shapes);

    //     default_gfx_backend.render_egui(meshes, textures_delta, glfw_backend.window_size_logical);
    //     if glfw_backend.is_opengl() {
    //         use egui_overlay::egui_window_glfw_passthrough::glfw::Context;

    //         glfw_backend.window.swap_buffers();
    //     } else {
    //         // for wgpu backend
    //         #[cfg(target_os = "macos")]
    //         default_gfx_backend.present()
    //     }
    //     Some((platform_output, repaint_after))
    // }
}

// cargo test:

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math() {
        let math = search::Math;

        let res = math.search("2+2");

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name(), "(2 + 2) = 4");
    }

    #[test]
    fn test_google() {
        let google = search::Google;

        let res = google.search("2+2");

        assert_eq!(res[1].name(), "4");
    }

    #[test]
    fn test_definition() {
        let definition = search::Dictionary;

        let res = definition.search("the");

        println!("res: {:#?}", res);
    }

    #[test]
    fn test_windowsapps() {
        let windowsapps = search::WindowsApps;

        let res = windowsapps.search("");

        println!("res: {:#?}", res);
    }
}
