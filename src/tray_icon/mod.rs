// use tray_icon::{menu::MenuEvent, Icon, TrayIconBuilder, TrayIconEvent};

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

        // tray.add_label("Tray Label").unwrap();

        // tray.add_menu_item("Hello", || {
        //     println!("Hello!");
        // })
        // .unwrap();

        // tray.inner_mut().add_separator().unwrap();

        die.recv().unwrap();
        // let tray_icon = match TrayIconBuilder::new()
        //     .with_icon({
        //         match load_icon(&ICON_PATH) {
        //             Ok(icon) => {
        //                 log::info!("Tray icon loaded");
        //                 icon
        //             }
        //             Err(e) => {
        //                 log::error!("Failed to load tray icon: {}", e);
        //                 return;
        //             }
        //         }
        //     })
        //     .build()
        // {
        //     Ok(tray_icon) => {
        //         log::info!("Tray icon created");
        //         tray_icon
        //     }
        //     Err(e) => {
        //         log::error!("Failed to create tray icon: {}", e);
        //         return;
        //     }
        // };
        // log::trace!("Tray icon created");

        // let (tray_sender, tray_channel) = crossbeam::channel::unbounded();
        // let (menu_sender, menu_channel) = crossbeam::channel::unbounded();
        // log::trace!("Tray icon event channels created");

        // MenuEvent::set_event_handler(Some(Box::new(move |event| {
        //     log::trace!("menu event: {:?}", event);
        //     if let Err(e) = menu_sender.send(event) {
        //         log::error!("Failed to send menu event: {}", e);
        //     }
        // })));
        // log::trace!("Menu event handler set");

        // TrayIconEvent::set_event_handler(Some(Box::new(move |event| {
        //     log::trace!("tray icon event: {:?}", event);
        //     if let Err(e) = tray_sender.send(event) {
        //         log::error!("Failed to send tray icon event: {}", e);
        //     }
        // })));
        // log::trace!("Tray icon event handler set");

        // loop {
        //     log::trace!("Tray icon event loop tick");
        //     crossbeam::select! {
        //         recv(tray_channel) -> event => {
        //             log::trace!("tray icon event: {:?}", event);
        //         }
        //         recv(menu_channel) -> event => {
        //             log::trace!("menu event: {:?}", event);
        //         }
        //         recv(die) -> _ => {
        //             log::info!("Received kill signal");
        //             break;
        //         }
        //     }
        // }

        // while let Ok(event) = TrayIconEvent::receiver().recv() {
        //     log::trace!("tray icon event: {:?}", event);
        // }
    })
}

// fn load_icon(path: &std::path::Path) -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
//     let (icon_rgba, icon_width, icon_height) = {
//         let image = image::open(path)?.into_rgba8();
//         let (width, height) = image.dimensions();
//         let rgba = image.into_raw();
//         (rgba, width, height)
//     };
//     Ok(tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)?)
// }
