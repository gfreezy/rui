use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify::{watcher, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;

use tracing::{debug, error};

use crate::ext_event::ExtEventSink;

use crate::ui::Ui;
use style::{parse_style_content, Style};

#[derive(Clone)]
struct StyleWatcher {
    path: String,
    _watcher: Arc<RecommendedWatcher>,
    ext_handle: ExtEventSink,
}

struct StyleCache {
    styles: Vec<Style>,
    hash: u32,
    path: String,
}

impl StyleCache {
    pub fn new(path: &str) -> Self {
        StyleCache {
            styles: Vec::new(),
            hash: 0,
            path: path.to_string(),
        }
    }

    pub fn set_path(&mut self, path: String) -> &mut Self {
        self.path = path;
        self
    }

    pub fn refresh_styles(&mut self) {
        let mut file = File::open(&self.path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let hash = fxhash::hash32(&content);

        if self.hash == hash {
            return;
        }

        let style = parse_style_content(&content).unwrap_or_else(|e| {
            error!("{}", e);
            Vec::new()
        });

        self.styles = style;
        self.hash = hash;
    }

    fn get_style(&mut self, name: &str) -> Style {
        self.refresh_styles();
        self.styles
            .iter()
            .find(|s| s.name == name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

thread_local! {
    static STYLE: RefCell<StyleCache>  = RefCell::new(StyleCache::new(""));
}

impl StyleWatcher {
    fn new(path: &str, ui: &mut Ui) -> Self {
        let (tx, rx) = channel();

        let mut w = watcher(tx, Duration::from_millis(100)).unwrap();
        w.watch(&path, RecursiveMode::NonRecursive).unwrap();

        let s = Self {
            ext_handle: ui.ext_handle().clone(),
            _watcher: Arc::new(w),
            path: path.to_string(),
        };

        let w_c = s.clone();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(event) => {
                    debug!("watch update {:?}", event);
                    w_c.ext_handle.add_idle_callback(|| {});
                }
                Err(e) => {
                    error!("watch error: {:?}", e);
                }
            }
        });

        s
    }

    fn get_style(&self, name: &str) -> Style {
        STYLE.with(|s| s.borrow_mut().set_path(self.path.clone()).get_style(name))
    }

    fn global(ui: &mut Ui) -> &'static StyleWatcher {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("style.css");
        static INSTANCE: OnceCell<StyleWatcher> = OnceCell::new();
        INSTANCE.get_or_init(|| StyleWatcher::new(&*path.to_string_lossy(), ui))
    }
}

pub(crate) fn live_style(ui: &mut Ui, name: &str) -> Style {
    let instance = StyleWatcher::global(ui);
    instance.get_style(name)
}
