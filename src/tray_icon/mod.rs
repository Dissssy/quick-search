use tray_item::{IconSource, TrayItem};

use crate::LOGGER;
use quick_search_lib::Log;

pub fn create_tray_icon_thread(kill: crossbeam::channel::Sender<bool>, die: crossbeam::channel::Receiver<bool>, ui_opener: crossbeam::channel::Sender<bool>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut tray = match TrayItem::new("Quick Search", IconSource::Resource("default-tray-icon"), 2, 0) {
            Ok(tray) => tray,
            Err(e) => {
                LOGGER.error(&format!("Failed to create tray icon: {}", e));
                return;
            }
        };

        match tray.add_label("Quick Search") {
            Ok(_) => LOGGER.trace("Tray icon label added"),
            Err(e) => LOGGER.error(&format!("Failed to add tray icon label: {}", e)),
        };

        match tray.inner_mut().add_separator() {
            Ok(_) => LOGGER.trace("Tray icon separator added"),
            Err(e) => LOGGER.error(&format!("Failed to add tray icon separator: {}", e)),
        };

        {
            let ui_opener = ui_opener.clone();
            match tray.add_menu_item("Search", move || {
                if let Err(e) = ui_opener.send(true) {
                    LOGGER.error(&format!("Failed to send to ui_opener channel: {}", e));
                }
            }) {
                Ok(_) => LOGGER.trace("Tray icon search menu item added"),
                Err(e) => LOGGER.error(&format!("Failed to add search menu item: {}", e)),
            };
        }

        match tray.add_menu_item("Configure", move || {
            if let Err(e) = ui_opener.send(false) {
                LOGGER.error(&format!("Failed to send to ui_opener channel: {}", e));
            }
        }) {
            Ok(_) => LOGGER.trace("Tray icon configure menu item added"),
            Err(e) => LOGGER.error(&format!("Failed to add configure menu item: {}", e)),
        };

        match tray.add_menu_item("Quit", move || {
            if let Err(e) = kill.send(true) {
                LOGGER.error(&format!("Failed to send to kill channel: {}", e));
            }
        }) {
            Ok(_) => LOGGER.trace("Tray icon quit menu item added"),
            Err(e) => LOGGER.error(&format!("Failed to add quit menu item: {}", e)),
        };

        if let Err(e) = die.recv() {
            LOGGER.error(&format!("Failed to receive from die channel: {}", e));
        };
    })
}
