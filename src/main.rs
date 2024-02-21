// #![windows_subsystem = "windows"]

mod config;
mod search_instance;

use config::Config;

use directories::ProjectDirs;
use minwin::sync::Mutex as WinMutex;
use std::{
    sync::{Arc, Mutex as StdMutex},
    thread::JoinHandle,
};
use windows_hotkeys::{keys::VKey as Key, HotkeyManager, HotkeyManagerImpl as _};

include_flate::flate!(pub static AUDIO_FILE_BYTES: [u8] from "assets/notif.mp3");

lazy_static::lazy_static! {
    static ref DIRECTORY: ProjectDirs = ProjectDirs::from("com", "planet-51-devs", "quick-search").expect("Failed to get project directories");
    static ref CONFIG_FILE: Arc<StdMutex<Config>> = Arc::new(StdMutex::new(Config::load()));
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
}

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

    // privelege level, its debugging stuff
    // search::set_clipboard(format!("privelege level: {:?}\nis_elevated: {}", privilege_level::privilege_level(), is_elevated::is_elevated()).as_str());

    // listen for F17 keypress from the keyboard
    let mut hkm = HotkeyManager::new();
    log::trace!("Hotkey manager created");

    let thread: Arc<StdMutex<Option<JoinHandle<()>>>> = Arc::new(StdMutex::new(None));
    log::trace!("Thread mutex created");

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

    match hkm.register(Key::F17, &[], move || {
        log::trace!("F17 pressed!");
        log::trace!("Software lock cloned");
        let thread = thread.clone();
        log::trace!("Thread cloned");
        match thread.lock() {
            Ok(mut threadopt) => {
                log::trace!("Thread mutex locked");
                if threadopt.as_ref().map(|x| x.is_finished()).unwrap_or(true) {
                    log::trace!("Thread is not running");
                    *threadopt = Some(std::thread::spawn(search_instance::instance));
                    log::trace!("Thread spawned");
                } else {
                    log::warn!("Thread is already running");
                }
            }
            Err(e) => {
                log::error!("Failed to lock thread mutex: {}", e);
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

    let (kill, kill_rx) = std::sync::mpsc::channel();
    let interrupt_handle = hkm.interrupt_handle();

    let hkm_thread = {
        let kill = kill.clone();
        std::thread::spawn(move || {
            hkm.event_loop();
            log::trace!("Hotkey manager event loop finished");
            match kill.send(()) {
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
    match ctrlc::set_handler(move || {
        log::info!("Received SIGINT, exiting");
        match kill.send(()) {
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

    log::trace!("Hotkey manager thread spawned");

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

    match hkm_thread.join() {
        Ok(_) => {
            log::info!("Hotkey manager thread finished");
        }
        Err(_) => {
            log::error!("Failed to join hotkey manager thread");
        }
    };

    match CONFIG_FILE.lock() {
        Ok(config) => {
            log::trace!("Config mutex locked");
            config.save();
        }
        Err(e) => {
            log::error!("Failed to lock config mutex: {}", e);
        }
    }

    log::info!("Exiting");
    drop(software_lock);
}
