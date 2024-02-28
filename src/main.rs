#![windows_subsystem = "windows"]

mod config;
mod search_instance;
mod tray_icon;

use config::ConfigLoader;

use directories::ProjectDirs;
use egui_overlay::egui_window_glfw_passthrough::glfw::PixelImage;
use minwin::sync::Mutex as WinMutex;
use std::{
    sync::{Arc, Mutex as StdMutex},
    thread::JoinHandle,
};
use windows_hotkeys::{keys::VKey as Key, HotkeyManager, HotkeyManagerImpl as _};

include_flate::flate!(pub static AUDIO_FILE_BYTES: [u8] from "assets/notif.mp3");
include_flate::flate!(pub static ICON_BYTES_16: [u8] from "assets/icon-16.png");
include_flate::flate!(pub static ICON_BYTES_32: [u8] from "assets/icon-32.png");
include_flate::flate!(pub static ICON_BYTES_64: [u8] from "assets/icon-64.png");
include_flate::flate!(pub static ICON_BYTES_128: [u8] from "assets/icon-128.png");

lazy_static::lazy_static! {
    static ref DIRECTORY: ProjectDirs = ProjectDirs::from("com", "planet-51-devs", "quick-search").expect("Failed to get project directories");
    static ref CONFIG_FILE: Arc<ConfigLoader> = Arc::new(ConfigLoader::new());
    static ref AUDIO_FILE_PATH: std::path::PathBuf = {
        let path = DIRECTORY.data_dir().join("notif.mp3");
        if !path.exists() {
            match std::fs::write(&path, &*AUDIO_FILE_BYTES) {
                Ok(_) => {
                    log::info!("Created notif.mp3");
                }
                Err(e) => {
                    log::error!("Failed to create notif.mp3: {}", e);
                }
            }
        }
        path
    };
    static ref CURRENT_PATH: std::path::PathBuf = std::env::current_exe().expect("Failed to get current exe path");
    static ref CORRECT_PATH: std::path::PathBuf = get_correct_path();
}

fn to_pixel_image(bytes: &[u8]) -> PixelImage {
    // PixelImage is a struct from egui_overlay that contains a width, height, and a Vec<u32> of pixels
    let img = image::load_from_memory(bytes).expect("Failed to load image from memory");
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();
    let mut pixels = Vec::with_capacity((width * height) as usize);
    // The image data is 32-bit, little-endian, non-premultiplied RGBA, i.e. eight bits per channel with the red channel first. The pixels are arranged canonically as sequential rows, starting from the top-left corner.
    for pixel in img.pixels() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        let a = pixel[3] as u32;
        pixels.push((a << 24) | (b << 16) | (g << 8) | r);
    }
    PixelImage { width, height, pixels }
}

// const DELAY_TUNING: u128 = 250;
// const TRUNCATE_CONTEXT_LENGTH: usize = 100;
// const TRUNCATE_TITLE_LENGTH: usize = 100;

fn main() {
    // setup logging
    env_logger::init();
    log::trace!("Logging initialized");

    match std::fs::create_dir_all(DIRECTORY.config_dir()) {
        Ok(_) => {
            log::info!("Created config directory");
        }
        Err(e) => {
            log::error!("Failed to create config directory: {}", e);
            return;
        }
    };
    match std::fs::create_dir_all(DIRECTORY.data_dir().join("plugins")) {
        Ok(_) => {
            log::info!("Created plugins directory");
        }
        Err(e) => {
            log::error!("Failed to create plugins directory: {}", e);
            return;
        }
    };

    // ensure the exe is being run from the correct path, if not, copy it to the correct path and prompt the user to run it from there, then exit

    log::info!("Exe path: {:?}", *CURRENT_PATH);
    log::info!("Correct path: {:?}", *CORRECT_PATH);

    if *CURRENT_PATH != *CORRECT_PATH {
        let res = rfd::MessageDialog::new()
            .set_title("Quick Search")
            .set_description(
                "The exe is not being run from the correct path, would you like it to be copied to the correct path and run from there? If you choose no, then some features may not work correctly.",
            )
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if res == rfd::MessageDialogResult::Yes {
            log::info!("User chose yes");
            match std::fs::copy(&*CURRENT_PATH, &*CORRECT_PATH) {
                Ok(_) => {
                    log::info!("Copied exe to correct path");
                }
                Err(e) => {
                    log::error!("Failed to copy exe to correct path: {}", e);
                    return;
                }
            };
            match std::process::Command::new(&*CORRECT_PATH)
                .env("RUST_LOG", std::env::var("RUST_LOG").unwrap_or_default())
                // .stdout(std::process::Stdio::piped())
                // .stderr(std::process::Stdio::piped())
                .spawn()
            {
                // Ok(mut handle) => {
                //     log::info!("Spawned correct exe");
                //     let (kill, kill_rx) = crossbeam::channel::unbounded::<bool>();
                //     ctrlc::set_handler(move || {
                //         match kill.send(true) {
                //             Ok(_) => {
                //                 log::info!("Sent kill signal to correct exe");
                //             }
                //             Err(_) => {
                //                 log::error!("Failed to send kill signal to correct exe");
                //             }
                //         };
                //     })
                //     .expect("Failed to set SIGINT handler");

                //     kill_rx.recv().expect("Failed to receive kill signal from correct exe");

                //     handle.kill().expect("Failed to kill correct exe");
                // }
                Ok(_) => {
                    log::info!("Spawned correct exe");
                }
                Err(e) => {
                    log::error!("Failed to spawn correct exe: {}", e);
                }
            };
            return;
        }
    }

    search_instance::preload();

    // privelege level, its debugging stuff
    // search::set_clipboard(format!("privelege level: {:?}\nis_elevated: {}", privilege_level::privilege_level(), is_elevated::is_elevated()).as_str());

    // listen for F17 keypress from the keyboard
    let mut hkm = HotkeyManager::new();
    log::trace!("Hotkey manager created");

    // Acquiring a windows mutex to ensure only one instance of the software is running, we also use this mutex to lock and ensure only one thread can ever possibly run at a time
    let software_lock = Arc::new(match WinMutex::create_named("Dissy-Quick-search") {
        Ok(lock) => {
            log::trace!("Software lock acquired");
            lock
        }
        Err(e) => {
            log::error!("Failed to acquire software lock: {}", e);
            return;
        }
    });

    let (ui_opener, ui_signal) = crossbeam::channel::unbounded::<bool>();

    {
        let ui_opener = ui_opener.clone();
        match hkm.register(Key::F17, &[], move || {
            log::trace!("F17 pressed!");
            match ui_opener.send(true) {
                Ok(_) => {
                    log::info!("Sent UI opener signal");
                }
                Err(e) => {
                    log::error!("Failed to send UI opener signal: {}", e);
                }
            };
        }) {
            Ok(_) => {
                log::info!("F17 hotkey registered");
            }
            Err(e) => {
                log::error!("Failed to register F17 hotkey: {}", e);
                return;
            }
        };
    }

    let (kill_ui, kill_ui_rx) = crossbeam::channel::unbounded::<bool>();

    let ui_opening_thread = {
        let thread: Arc<StdMutex<Option<JoinHandle<()>>>> = Arc::new(StdMutex::new(None));
        log::trace!("Thread mutex created");
        let ui_signal = ui_signal;
        let kill_ui_rx = kill_ui_rx;

        std::thread::spawn(move || {
            loop {
                crossbeam::select! {
                    recv(kill_ui_rx) -> _ => {
                        log::trace!("Received kill signal");
                        break
                    }
                    recv(ui_signal) -> msg => {
                        let regular = match msg {
                            Ok(val) => val,
                            Err(e) => {
                                log::error!("Failed to receive UI opener signal: {}", e);
                                continue;
                            }
                        };
                        log::trace!("Received UI opener signal");
                        let thread = thread.clone();
                        log::trace!("Thread cloned");
                        match thread.lock() {
                            Ok(mut threadopt) => {
                                log::trace!("Thread mutex locked");
                                if threadopt.as_ref().map(|x| x.is_finished()).unwrap_or(true) {
                                    log::trace!("Thread is not running");
                                    *threadopt = Some(std::thread::spawn(move || search_instance::instance(regular)));
                                    log::trace!("Thread spawned");
                                } else {
                                    log::warn!("Thread is already running");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to lock thread mutex: {}", e);
                            }
                        };
                    }
                }
            }
            log::trace!("UI opening thread done");
        })
    };

    let (kill, kill_rx) = crossbeam::channel::unbounded::<bool>();
    let interrupt_handle = hkm.interrupt_handle();

    let hkm_thread = {
        let kill = kill.clone();
        std::thread::spawn(move || {
            hkm.event_loop();
            log::trace!("Hotkey manager event loop finished");
            match kill.send(true) {
                Ok(_) => {
                    log::info!("Hotkey manager event loop finished");
                }
                Err(_) => {
                    log::error!("Failed to send kill signal to hotkey manager event loop");
                }
            };
        })
    };

    // let signal_thread = std::thread::spawn(move || {});
    {
        let kill = kill.clone();
        match ctrlc::set_handler(move || {
            log::info!("Received SIGINT, exiting");
            match kill.send(true) {
                Ok(_) => {
                    log::info!("Sent kill signal to hotkey manager event loop");
                }
                Err(_) => {
                    log::error!("Failed to send kill signal to hotkey manager event loop");
                }
            };
        }) {
            Ok(_) => {
                log::info!("SIGINT handler set");
            }
            Err(e) => {
                log::error!("Failed to set SIGINT handler: {}", e);
            }
        }
    }

    log::trace!("Hotkey manager thread spawned");

    let (kill_tray_icon, kill_tray_icon_rx) = crossbeam::channel::unbounded::<bool>();

    let tray_icon_thread = tray_icon::create_tray_icon_thread(kill, kill_tray_icon_rx, ui_opener);

    // wait for the kill signal or a sigterm or sigint
    match kill_rx.recv() {
        Ok(_) => {
            log::info!("Received kill signal");
        }
        Err(_) => {
            log::error!("Failed to receive kill signal");
        }
    };

    interrupt_handle.interrupt();

    match kill_ui.send(true) {
        Ok(_) => {
            log::info!("Sent kill signal to UI opening thread");
        }
        Err(_) => {
            log::error!("Failed to send kill signal to UI opening thread");
        }
    };

    match ui_opening_thread.join() {
        Ok(_) => {
            log::info!("UI opening thread finished");
        }
        Err(_) => {
            log::error!("Failed to join UI opening thread");
        }
    };

    match hkm_thread.join() {
        Ok(_) => {
            log::info!("Hotkey manager thread finished");
        }
        Err(_) => {
            log::error!("Failed to join hotkey manager thread");
        }
    };

    match kill_tray_icon.send(true) {
        Ok(_) => {
            log::info!("Sent kill signal to tray icon thread");
        }
        Err(_) => {
            log::error!("Failed to send kill signal to tray icon thread");
        }
    };

    match tray_icon_thread.join() {
        Ok(_) => {
            log::info!("Tray icon thread finished");
        }
        Err(_) => {
            log::error!("Failed to join tray icon thread");
        }
    };

    log::info!("Exiting");
    drop(software_lock);
}

fn get_correct_path() -> std::path::PathBuf {
    DIRECTORY.data_dir().join("quick-search.exe")
}

fn icon_pixelimages() -> Vec<PixelImage> {
    vec![
        to_pixel_image(&ICON_BYTES_16),
        to_pixel_image(&ICON_BYTES_32),
        to_pixel_image(&ICON_BYTES_64),
        to_pixel_image(&ICON_BYTES_128),
    ]
}
