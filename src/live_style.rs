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

use crate::style::{parse_style_content, Style};
use crate::ui::Ui;

#[derive(Clone)]
struct StyleWatcher {
    content: Arc<Mutex<String>>,
    _watcher: Arc<RecommendedWatcher>,
    ext_handle: ExtEventSink,
    path: String,
}

impl StyleWatcher {
    fn new(path: &str, ui: &mut Ui) -> Self {
        let content = Arc::new(Mutex::new(String::new()));
        let (tx, rx) = channel();

        let mut w = watcher(tx, Duration::from_millis(100)).unwrap();
        w.watch(&path, RecursiveMode::NonRecursive).unwrap();

        let s = Self {
            content,
            path: path.to_string(),
            ext_handle: ui.ext_handle().clone(),
            _watcher: Arc::new(w),
        };
        s.refresh_data();

        let w_c = s.clone();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(event) => {
                    debug!("watch update {:?}", event);
                    w_c.refresh_data();
                }
                Err(e) => {
                    error!("watch error: {:?}", e);
                }
            }
        });

        s
    }

    fn refresh_data(&self) {
        let mut f = File::open(&self.path).unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();
        *self.content.lock().unwrap() = content;
        // trigger update
        self.ext_handle.add_idle_callback(|| {});
    }

    fn get_style(&self, name: &str) -> Style {
        let content = self.content.lock().unwrap();
        let styles = match parse_style_content(&content) {
            Ok(s) => s,
            Err(e) => {
                error!("parse style error: {}", e);
                Default::default()
            }
        };
        styles
            .into_iter()
            .find(|s| s.name == name)
            .map(|v| v.into())
            .unwrap_or_default()
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
