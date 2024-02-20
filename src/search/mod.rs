mod englishdictionary;
mod google;
mod math;
mod urbandictionary;
mod vteverything;
mod windowsapps;

pub use englishdictionary::Dictionary;
pub use google::Google;
pub use math::Math;
pub use urbandictionary::UrbanDictionary;
pub use vteverything::Everything;
pub use windowsapps::WindowsApps;

const LENLIM: usize = 512;

pub trait Searchable
where
    Self: Send + Sync + 'static,
{
    fn search(&self, query: &str) -> Vec<SearchResult>;
    fn copy(&self) -> Box<dyn Searchable>;
    fn name(&self) -> String;
    fn priority(&self) -> i32 {
        0
    }
    fn colored_name(&self) -> egui::text::LayoutJob;
}

pub struct SearchResult {
    name: String,
    context: Option<String>,
    action: Option<Box<dyn Fn() + Send>>,
    source: Box<dyn Searchable>,
}

impl SearchResult {
    pub fn name(&self) -> &str {
        // clamp len to LENLIM
        if self.name.len() > LENLIM {
            &self.name[..LENLIM]
        } else {
            &self.name
        }
    }
    pub fn context(&self) -> Option<&str> {
        // clamp len to LENLIM
        if let Some(context) = &self.context {
            if context.len() > LENLIM {
                Some(&context[..LENLIM])
            } else {
                Some(context)
            }
        } else {
            None
        }
    }

    pub fn action(&mut self) {
        // (self.action)();
        self.action.take().map(|x| {
            std::thread::spawn(move || {
                (x)();
            })
        });
    }

    pub fn source(&self) -> &dyn Searchable {
        &*self.source
    }
}

impl std::fmt::Debug for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchResult")
            .field("name", &self.name)
            .field("context", &self.context)
            .field("source", &self.source.name())
            .finish()
    }
}

pub fn set_clipboard(s: &str) {
    if let Ok::<clipboard::ClipboardContext, Box<dyn std::error::Error>>(mut clipboard) = clipboard::ClipboardProvider::new() {
        if let Ok(()) = clipboard::ClipboardProvider::set_contents(&mut clipboard, s.to_owned()) {
            println!("copied to clipboard: {}", s);
        } else {
            println!("failed to copy to clipboard: {}", s);
        }
    } else {
        println!("failed to copy to clipboard: {}", s);
    }
}

pub fn open(path: &std::path::Path) {
    println!("opening file: {}", path.display());

    if let Err(e) = opener::open(path) {
        match e {
            opener::OpenError::Io(ioe) => println!("io error: {}", ioe),
            _ => println!("error: {}", e),
        }
    } else {
        println!("opened file");
    };
}
