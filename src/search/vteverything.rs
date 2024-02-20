use super::{SearchResult, Searchable};
use everything_sys::*;
use widestring::U16CString;

#[derive(Copy, Clone)]
pub struct Everything;

impl Searchable for Everything {
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut res = vec![];

        // attempt to search for query in VOIDTOOLS: Everything

        if let Ok(query_as_wchar) = U16CString::from_str(query) {
            unsafe {
                Everything_SetSearchW(query_as_wchar.as_ptr());
            }
            if unsafe { Everything_QueryW(1) } == 1 {
                let f = unsafe { Everything_GetNumResults() }.clamp(0, 100);
                for i in 0..f {
                    let filename = unsafe {
                        let ptr = Everything_GetResultFileNameW(i);
                        if ptr.is_null() {
                            continue;
                        } else {
                            U16CString::from_ptr_str(ptr).to_string_lossy()
                        }
                    };
                    let extension = unsafe {
                        let ptr = Everything_GetResultExtensionW(i);
                        if !ptr.is_null() {
                            Some(U16CString::from_ptr_str(ptr).to_string_lossy())
                        } else {
                            None
                        }
                    };
                    let path = unsafe {
                        let ptr = Everything_GetResultPathW(i);
                        if ptr.is_null() {
                            continue;
                        } else {
                            U16CString::from_ptr_str(ptr).to_string_lossy()
                        }
                    };
                    // let resstr = format!("{}", filename);
                    let fullfile = match extension {
                        Some(extension) => format!("{}.{}", filename, extension),
                        None => filename.clone(),
                    };

                    res.push(SearchResult {
                        source: Box::new(*self),
                        name: fullfile.clone(),
                        context: Some(format!("{}\\{}", path, fullfile)),
                        action: Some(Box::new(move || {
                            // open file

                            let path = std::path::PathBuf::from(format!("{}\\{}", path, fullfile));

                            super::open(&path);
                        })),
                    });
                }
            }
        } else {
            println!("failed to convert query to wchar");
        }

        res.sort_by(|a, b| a.name.cmp(&b.name));
        res.dedup_by(|a, b| a.name == b.name);

        res
    }
    fn copy(&self) -> Box<dyn Searchable> {
        Box::new(*self)
    }
    fn name(&self) -> String {
        "Everything".to_string()
    }
    fn priority(&self) -> i32 {
        500
    }
    // fn color(&self) -> Option<egui::Color32> {
    //     Some(egui::Color32::from_rgb(255, 0, 0))
    // }
    fn colored_name(&self) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        job.append(
            &self.name(),
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(255, 127, 0),
                ..Default::default()
            },
        );

        job
    }
}
