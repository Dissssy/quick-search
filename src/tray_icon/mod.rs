use tray_item::{IconSource, TrayItem};

pub fn create_tray_icon_thread(kill: crossbeam::channel::Sender<bool>, die: crossbeam::channel::Receiver<bool>, ui_opener: crossbeam::channel::Sender<bool>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut tray = TrayItem::new("Quick Search", IconSource::Resource("default-tray-icon"), 2, 0).expect("Failed to create tray icon");

        tray.add_label("Quick Search").ok();

        tray.inner_mut().add_separator().ok();

        {
            let ui_opener = ui_opener.clone();
            tray.add_menu_item("Search", move || {
                ui_opener.send(true).ok();
            })
            .ok();
        }

        tray.add_menu_item("Configure", move || {
            ui_opener.send(false).ok();
        })
        .ok();

        tray.add_menu_item("Quit", move || {
            kill.send(true).ok();
        })
        .ok();

        if let Err(e) = die.recv() {
            log::error!("Failed to receive kill signal: {}", e);
        };
    })
}
