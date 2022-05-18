use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify::{watcher, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;

use tracing::{debug, error};

use crate::ext_event::ExtEventSink;

use crate::ui::Ui;
use style::{parse_rule, parse_style_content, Style};

struct StyleCache {
    styles: HashMap<StyleId, Style>,
    path: String,
}

const LIVE_MACRO: &str = "live_s!";

impl StyleCache {
    pub fn new(path: &str) -> Self {
        StyleCache {
            styles: HashMap::new(),
            path: path.to_string(),
        }
    }

    pub fn set_path(&mut self, path: String) -> &mut Self {
        self.path = path;
        self
    }

    pub fn refresh_styles(&mut self, path: &Path) {
        let content = read_file_content(path);
        let styles = parse_styles_from_file_content(&content);
        for (i, style) in styles.into_iter().enumerate() {
            let style_id = StyleId {
                file: path.to_string_lossy().to_string(),
                id: i,
            };
            tracing::trace!("refresh style: {:?}", style_id);
            self.styles.insert(style_id, style);
        }
    }

    fn init_style(&mut self, style_id: &StyleId, content: &str) -> Option<()> {
        if !self.styles.contains_key(style_id) {
            tracing::trace!("init style: {:?}", style_id);
            if content.trim().is_empty() {
                self.styles.insert(style_id.clone(), Default::default());
            } else {
                let mut styles = parse_style_content(&content).unwrap();
                self.styles.insert(style_id.clone(), styles.remove(0));
            }
            Some(())
        } else {
            None
        }
    }

    fn get_style(&mut self, style_id: &StyleId) -> Option<Style> {
        tracing::trace!("get style: {:?}", style_id);
        self.styles.get(style_id).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StyleId {
    file: String,
    id: usize,
}

struct StyleCenter {
    initial_file_haystack: HashMap<StylePos, StyleId>,
}

impl StyleCenter {
    pub fn new(path: &str) -> Self {
        let styles = grep_styles(Some(path));
        let mut group = HashMap::new();
        for style in styles {
            group
                .entry(style.file.clone())
                .or_insert_with(|| Vec::new())
                .push(style);
        }

        group.values_mut().for_each(|v| v.sort_by_key(|s| s.line));

        let mut map = HashMap::new();
        for (file, file_styles) in group {
            for (i, style_pos) in file_styles.into_iter().enumerate() {
                map.insert(
                    style_pos,
                    StyleId {
                        file: file.clone(),
                        id: i,
                    },
                );
            }
        }

        tracing::trace!("initial_file_haystack: {:#?}", map);
        StyleCenter {
            initial_file_haystack: map,
        }
    }

    fn get_id(&mut self, pos: &StylePos) -> Option<StyleId> {
        let ret = self.initial_file_haystack.get(pos).cloned();
        tracing::trace!("get id mapping: {:?} -> {:?}", pos, ret);
        ret
    }
}

#[derive(Clone)]
struct StyleWatcher {
    path: String,
    _watcher: Arc<RecommendedWatcher>,
    ext_handle: ExtEventSink,
    cache: Arc<Mutex<StyleCache>>,
    id_mapping: Arc<Mutex<StyleCenter>>,
}

impl StyleWatcher {
    fn new(path: &str, ui: &Ui) -> Self {
        tracing::debug!("watch: {}", path);
        let (tx, rx) = channel();

        let mut w = watcher(tx, Duration::from_millis(100)).unwrap();
        w.watch(&path, RecursiveMode::Recursive).unwrap();

        let s = Self {
            ext_handle: ui.ext_handle().clone(),
            _watcher: Arc::new(w),
            path: path.to_string(),
            cache: Arc::new(Mutex::new(StyleCache::new(path))),
            id_mapping: Arc::new(Mutex::new(StyleCenter::new(path))),
        };

        let w_c = s.clone();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(event) => {
                    if let Some(path) = try_get_style_path(event) {
                        tracing::trace!("refresh style for path: {:?}", path);

                        w_c.cache.lock().unwrap().refresh_styles(&path);
                        w_c.ext_handle.add_idle_callback(|| {});
                    }
                }
                Err(e) => {
                    error!("watch error: {:?}", e);
                }
            }
        });
        s
    }

    fn get_style(&self, style_pos: &StylePos) -> Option<Style> {
        let id = self.id_mapping.lock().unwrap().get_id(style_pos);
        self.cache.lock().unwrap().get_style(&id?)
    }

    fn init_style(&self, style_pos: &StylePos, content: &str) -> Option<()> {
        let id = self.id_mapping.lock().unwrap().get_id(style_pos);
        self.cache.lock().unwrap().init_style(&id?, content)
    }

    fn global(ui: &Ui) -> &'static StyleWatcher {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        static INSTANCE: OnceCell<StyleWatcher> = OnceCell::new();
        INSTANCE.get_or_init(|| StyleWatcher::new(&*path.parent().unwrap().to_string_lossy(), ui))
    }
}

fn try_get_style_path(event: notify::DebouncedEvent) -> Option<PathBuf> {
    match event {
        notify::DebouncedEvent::Write(path) if did_file_contains_live_styles(&path) => Some(path),
        _ => None,
    }
}

fn did_file_contains_live_styles(path: &Path) -> bool {
    if path.extension().unwrap() != "rs" {
        return false;
    }
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content.contains(LIVE_MACRO)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct StylePos {
    pub line: usize,
    pub file: String,
}

impl StylePos {
    pub fn new(line: usize, path: &str) -> Self {
        let path: PathBuf = std::path::PathBuf::from(path);
        let abs_path = path.canonicalize().unwrap().to_string_lossy().to_string();
        Self {
            line,
            file: abs_path,
        }
    }
}

fn grep_styles(path: Option<&str>) -> Vec<StylePos> {
    let arg = format!(r#"--no-heading -e {}\("#, LIVE_MACRO);
    let mut cmd = std::process::Command::new("/opt/homebrew/bin/rg");
    cmd.args(arg.split_whitespace());
    cmd.arg("-n");
    if let Some(path) = path {
        cmd.arg(path);
    }
    let output = cmd.output().unwrap();
    if !output.status.success() {
        panic!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    tracing::trace!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.split(':').collect::<Vec<_>>())
        .map(|groups| StylePos {
            file: groups[0].to_string(),
            line: groups[1].parse::<usize>().unwrap(),
        })
        .collect()
}

fn read_file_content(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content
}

fn parse_styles_from_file_content(content: &str) -> Vec<Style> {
    let blocks = find_live_s_blocks(&content);
    blocks
        .into_iter()
        .filter_map(|block| {
            if block.trim().is_empty() {
                return Some(Default::default());
            }

            match parse_rule(&block) {
                Ok((_, rule)) => {
                    tracing::trace!("parse rule successfully: {}", block);
                    Some(rule)
                }
                Err(e) => {
                    tracing::error!("parse rule error: {:?}, content: {}", e, block);
                    None
                }
            }
        })
        .collect()
}

macro_rules! regex {
    ($re:expr $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

fn find_live_s_blocks(content: &str) -> Vec<String> {
    let rexp = format!(
        r##"(?s){}\s*\(\s*.*?,\s*(?:r#"|")(.*?)(:?"#|")\s*\)"##,
        LIVE_MACRO
    );
    tracing::trace!("regexp: {}", rexp);
    let re = regex!(&rexp);
    re.captures_iter(content)
        .map(|c| c[1].to_string())
        .collect()
}

pub fn live_style(ui: &Ui, style_pos: &StylePos, style: &str) -> Style {
    let instance = StyleWatcher::global(ui);
    match instance.get_style(style_pos) {
        Some(s) => {
            tracing::trace!("found style: {:?}", style_pos);
            s
        }
        None => {
            tracing::trace!("init style");

            if instance.init_style(style_pos, style).is_none() {
                panic!("style not found: {:?}", style_pos);
            } else {
                instance.get_style(style_pos).unwrap()
            }
        }
    }
}

#[macro_export]
macro_rules! live_s {
    ($ui:ident, $style:expr) => {{
        let line = std::line!() as usize;
        let file = std::file!();
        let pos = rui::live_style::StylePos::new(line, file);

        $crate::live_style::live_style($ui, &pos, $style)
    }};
}
#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = r##"
    use rui::{
        app::{AppLauncher, WindowDesc},
        live_s,
        menu::mac::menu_bar,
        prelude::*,
        ui::Ui,
    };

    fn win(ui: &mut Ui) {
        let count = ui.state_node(|| 0usize);
        flex(ui, live_s!(ui, r#".b {font-size: 32.0;}"#), |ui| {
            text(
                ui,
                &format!("{}", *count),
                live_s!(
                    ui,
                    ".a { }"
                ),
            );
            button(ui, "Increment", move || {
                count.update(|c| *c += 1);
            });
        });
    }

    fn main() {
        let desc = WindowDesc::new("app".to_string(), move |ui| win(ui)).menu(|_| menu_bar());
        let app = AppLauncher::with_windows(vec![desc]).log_to_console();
        app.launch().unwrap();
    }

    "##;
    #[test]
    fn test_find_live_blocks() {
        let blocks = find_live_s_blocks(CONTENT);
        assert!(blocks.len() == 2);
        let rules: Vec<_> = blocks
            .into_iter()
            .map(|block| {
                let (_, rule) = parse_rule(&block).unwrap();
                rule
            })
            .collect();

        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_parse_styles() {
        dbg!(parse_styles_from_file_content(
            ".counter { font-size: 10.0; }"
        ));
    }

    #[test]
    fn test_try_get_style_path() {
        assert!(did_file_contains_live_styles(Path::new(
            "/Users/feichao/Developer/allsunday/rui/crates/esplori/examples/counter.rs"
        )));
    }
}
