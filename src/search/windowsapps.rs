use super::{SearchResult, Searchable};

use winreg::enums::*;
use winreg::RegKey;

#[derive(Copy, Clone)]
pub struct WindowsApps;

impl Searchable for WindowsApps {
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = vec![];

        {
            let mut apps = vec![];

            // gonna get all apps from windows registry
            // {basekey}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

            if let Ok(basekey) = hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", KEY_READ) {
                for key in basekey.enum_keys().filter_map(Result::ok) {
                    if let Ok(appkey) = basekey.open_subkey_with_flags(&key, KEY_READ) {
                        if let Ok(app_name) = appkey.get_value::<String, _>("DisplayName") {
                            if let Ok(app_path) = appkey.get_value::<String, _>("InstallLocation") {
                                apps.push((app_name.trim().to_owned(), app_path.trim().to_owned()));
                            }
                        }
                    }
                }
            }

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);

            if let Ok(basekey) = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", KEY_READ) {
                for key in basekey.enum_keys().filter_map(Result::ok) {
                    if let Ok(appkey) = basekey.open_subkey_with_flags(&key, KEY_READ) {
                        if let Ok(app_name) = appkey.get_value::<String, _>("DisplayName") {
                            if let Ok(app_path) = appkey.get_value::<String, _>("InstallLocation") {
                                apps.push((app_name.trim().to_owned(), app_path.trim().to_owned()));
                            }
                        }
                    }
                }
            }

            for (app_name, app_path) in apps {
                if app_name.to_lowercase().contains(&query.to_lowercase()) {
                    let app_path = if app_path.is_empty() { None } else { Some(app_path) };
                    results.push(SearchResult {
                        name: app_name.clone(),
                        context: Some(app_path.clone().unwrap_or("NO PATH FOUND, SELECTING DOES NOTHING".to_string())),
                        source: Box::new(*self),
                        action: Some(Box::new(move || match app_path.as_ref() {
                            Some(app_path) => {
                                let path = std::path::PathBuf::from(app_path);
                                super::open(&path);
                            }
                            None => {
                                println!("no path for app: {}", app_name);
                            }
                        })),
                    });
                }
            }
        }

        results.sort_by(|a, b| a.name.cmp(&b.name));
        results.dedup_by(|a, b| a.name == b.name);

        results
    }
    fn copy(&self) -> Box<dyn Searchable> {
        Box::new(*self)
    }
    fn name(&self) -> String {
        "Windows Apps".to_string()
    }
    fn priority(&self) -> i32 {
        800
    }
    // fn color(&self) -> Option<egui::Color32> {
    //     Some(egui::Color32::from_rgb(255, 0, 0))
    // }
    fn colored_name(&self) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        // cycle each letter through these colors:
        // 242, 80, 34
        // 127, 186, 0
        // 0, 164, 239
        // 255, 185, 0

        job.append(
            "W",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(242, 80, 34),
                ..Default::default()
            },
        );

        job.append(
            "i",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(127, 186, 0),
                ..Default::default()
            },
        );

        job.append(
            "n",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(0, 164, 239),
                ..Default::default()
            },
        );

        job.append(
            "d",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(255, 185, 0),
                ..Default::default()
            },
        );

        job.append(
            "o",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(242, 80, 34),
                ..Default::default()
            },
        );

        job.append(
            "w",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(127, 186, 0),
                ..Default::default()
            },
        );

        job.append(
            "s",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(0, 164, 239),
                ..Default::default()
            },
        );

        job.append(
            " ",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(115, 115, 115),
                ..Default::default()
            },
        );

        job.append(
            "A",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(255, 185, 0),
                ..Default::default()
            },
        );

        job.append(
            "p",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(242, 80, 34),
                ..Default::default()
            },
        );

        job.append(
            "p",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(127, 186, 0),
                ..Default::default()
            },
        );

        job.append(
            "s",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(0, 164, 239),
                ..Default::default()
            },
        );

        job
    }
}
