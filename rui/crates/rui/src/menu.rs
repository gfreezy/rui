use std::num::NonZeroU32;

use druid_shell::{Counter, HotKey, IntoKey, RawMods};

use crate::{
    app_state::CommandQueue,
    commands::{Command, Target},
    id::WindowId,
};

type MenuBuild = Box<dyn FnMut(Option<WindowId>) -> Menu>;

/// This is for completely recreating the menus (for when you want to change the actual menu
/// structure, rather than just, say, enabling or disabling entries).
pub(crate) struct MenuManager {
    // The function for rebuilding the menu. If this is `None` (which is the case for context
    // menus), `menu` will always be `Some(..)`.
    build: Option<MenuBuild>,
    popup: bool,
    menu: Option<Menu>,
}

impl MenuManager {
    /// Create a new [`MenuManager`] for a title-bar menu.
    pub fn new(build: impl FnMut(Option<WindowId>) -> Menu + 'static) -> MenuManager {
        MenuManager {
            build: Some(Box::new(build)),
            popup: false,
            menu: None,
        }
    }

    /// Create a new [`MenuManager`] for a context menu.
    pub fn new_for_popup(menu: Menu) -> MenuManager {
        MenuManager {
            build: None,
            popup: true,
            menu: Some(menu),
        }
    }

    /// If this platform always expects windows to have a menu by default, returns a menu.
    /// Otherwise, returns `None`.
    #[allow(unreachable_code)]
    pub fn platform_default() -> Option<MenuManager> {
        #[cfg(target_os = "macos")]
        return Some(MenuManager::new(|_| mac::application::default()));

        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "openbsd"))]
        return None;

        // we want to explicitly handle all platforms; log if a platform is missing.
        tracing::warn!("MenuManager::platform_default is not implemented for this platform.");
        None
    }

    /// Build an initial menu from the application data.
    pub fn initialize(&mut self, window: Option<WindowId>) -> druid_shell::Menu {
        if let Some(build) = &mut self.build {
            self.menu = Some((build)(window));
        }
        self.refresh()
    }

    /// Builds a new menu for displaying the given data.
    ///
    /// Mostly you should probably use `update` instead, because that actually checks whether a
    /// refresh is necessary.
    pub fn refresh(&mut self) -> druid_shell::Menu {
        if let Some(menu) = self.menu.as_mut() {
            let mut ctx = MenuBuildCtx::new(self.popup);
            menu.refresh_children(&mut ctx);
            ctx.current
        } else {
            tracing::error!("tried to refresh uninitialized menus");
            druid_shell::Menu::new()
        }
    }

    /// Called when a menu event is received from the system.
    pub fn event(&mut self, queue: &mut CommandQueue, window: Option<WindowId>, id: MenuItemId) {
        if let Some(m) = &mut self.menu {
            let mut ctx = MenuEventCtx { window, queue };
            m.activate(&mut ctx, id);
        }
    }
}

/// This context helps menu items to build the platform menu.
struct MenuBuildCtx {
    current: druid_shell::Menu,
}

impl MenuBuildCtx {
    fn new(popup: bool) -> MenuBuildCtx {
        MenuBuildCtx {
            current: if popup {
                druid_shell::Menu::new_for_popup()
            } else {
                druid_shell::Menu::new()
            },
        }
    }

    fn with_submenu(&mut self, text: &str, enabled: bool, f: impl FnOnce(&mut MenuBuildCtx)) {
        let mut child = MenuBuildCtx::new(false);
        f(&mut child);
        self.current.add_dropdown(child.current, text, enabled);
    }

    fn add_item(
        &mut self,
        id: u32,
        text: &str,
        key: Option<&HotKey>,
        enabled: bool,
        selected: bool,
    ) {
        self.current.add_item(id, text, key, enabled, selected);
    }

    fn add_separator(&mut self) {
        self.current.add_separator();
    }
}

trait MenuVisitor {
    fn activate(&mut self, ctx: &mut MenuEventCtx, id: MenuItemId);

    /// Called to refresh the menu.
    fn refresh(&mut self, ctx: &mut MenuBuildCtx);
}

/// An entry in a menu.
///
/// An entry is either a [`MenuItem`], a submenu (i.e. [`Menu`]), or one of a few other
/// possibilities (such as one of the two options above, wrapped in a [`MenuLensWrap`]).
pub struct MenuEntry {
    inner: Box<dyn MenuVisitor>,
}

impl MenuVisitor for MenuEntry {
    fn activate(&mut self, ctx: &mut MenuEventCtx, id: MenuItemId) {
        self.inner.activate(ctx, id);
    }

    fn refresh(&mut self, ctx: &mut MenuBuildCtx) {
        self.inner.refresh(ctx);
    }
}

/// A menu.
///
/// Menus can be nested arbitrarily, so this could also be a submenu.
/// See the [module level documentation](crate::menu) for more on how to use menus.
pub struct Menu {
    item: MenuItem,
    children: Vec<MenuEntry>,
    // bloom?
}

/// Uniquely identifies a menu item.
///
/// On the druid-shell side, the id is represented as a u32.
/// We reserve '0' as a placeholder value; on the Rust side
/// we represent this as an `Option<NonZerou32>`, which better
/// represents the semantics of our program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MenuItemId(Option<NonZeroU32>);

impl MenuItemId {
    pub(crate) fn new(id: u32) -> MenuItemId {
        MenuItemId(NonZeroU32::new(id))
    }
}

type MenuCallback = Box<dyn FnMut(&mut MenuEventCtx)>;
type HotKeyCallback = Box<dyn FnMut() -> Option<HotKey>>;

// The resolved state of a menu item.
#[derive(PartialEq)]
struct MenuItemState {
    title: String,
    hotkey: Option<HotKey>,
    selected: bool,
    enabled: bool,
}

/// An item in a menu.
///
/// See the [module level documentation](crate::menu) for more on how to use menus.
pub struct MenuItem {
    id: MenuItemId,
    title: String,
    callback: Option<MenuCallback>,
    hotkey: Option<HotKeyCallback>,
    selected: Option<Box<dyn FnMut() -> bool>>,
    enabled: Option<Box<dyn FnMut() -> bool>>,
    state: Option<MenuItemState>,
}

impl Menu {
    /// Create an empty menu.
    pub fn empty() -> Menu {
        Menu {
            item: MenuItem::new(""),
            children: Vec::new(),
        }
    }

    /// Create a menu with the given name.
    pub fn new(title: impl Into<String>) -> Menu {
        Menu {
            item: MenuItem::new(title),
            children: Vec::new(),
        }
    }

    /// Append a menu entry to this menu, returning the modified menu.
    pub fn entry(mut self, entry: impl Into<MenuEntry>) -> Self {
        self.children.push(entry.into());
        self
    }

    /// Append a separator to this menu, returning the modified menu.
    pub fn separator(self) -> Self {
        self.entry(Separator)
    }

    // This is like MenuVisitor::refresh, but it doesn't add a submenu for the current level.
    // (This is the behavior we need for the top-level (unnamed) menu, which contains (e.g.) File,
    // Edit, etc. as submenus.)
    fn refresh_children(&mut self, ctx: &mut MenuBuildCtx) {
        for child in &mut self.children {
            child.refresh(ctx);
        }
    }
}

struct Separator;

impl MenuVisitor for Separator {
    fn activate(&mut self, _ctx: &mut MenuEventCtx, _id: MenuItemId) {}

    fn refresh(&mut self, ctx: &mut MenuBuildCtx) {
        ctx.add_separator();
    }
}

impl From<Separator> for MenuEntry {
    fn from(s: Separator) -> MenuEntry {
        MenuEntry { inner: Box::new(s) }
    }
}

impl MenuVisitor for MenuItem {
    fn activate(&mut self, ctx: &mut MenuEventCtx, id: MenuItemId) {
        if id == self.id {
            if let Some(callback) = &mut self.callback {
                callback(ctx);
            }
        }
    }

    fn refresh(&mut self, ctx: &mut MenuBuildCtx) {
        self.resolve();
        let state = self.state.as_ref().unwrap();
        ctx.add_item(
            self.id.0.map(|x| x.get()).unwrap_or(0),
            &self.title,
            state.hotkey.as_ref(),
            state.enabled,
            state.selected,
        );
    }
}

impl From<MenuItem> for MenuEntry {
    fn from(i: MenuItem) -> MenuEntry {
        MenuEntry { inner: Box::new(i) }
    }
}

impl From<Menu> for MenuEntry {
    fn from(menu: Menu) -> MenuEntry {
        MenuEntry {
            inner: Box::new(menu),
        }
    }
}
impl MenuVisitor for Menu {
    fn activate(&mut self, ctx: &mut MenuEventCtx, id: MenuItemId) {
        for child in &mut self.children {
            child.activate(ctx, id);
        }
    }

    fn refresh(&mut self, ctx: &mut MenuBuildCtx) {
        self.item.resolve();
        let children = &mut self.children;
        ctx.with_submenu(self.item.text(), self.item.is_enabled(), |ctx| {
            for child in children {
                child.refresh(ctx);
            }
        });
    }
}

static COUNTER: Counter = Counter::new();

/// This context is available to the callback that is called when a menu item is activated.
///
/// Currently, it only allows for submission of [`Command`]s.
///
/// [`Command`]: crate::Command
pub struct MenuEventCtx<'a> {
    window: Option<WindowId>,
    queue: &'a mut CommandQueue,
}

impl<'a> MenuEventCtx<'a> {
    /// Submit a [`Command`] to be handled by the main widget tree.
    ///
    /// If the command's target is [`Target::Auto`], it will be sent to the menu's window if the
    /// menu is associated with a window, or to [`Target::Global`] if the menu is not associated
    /// with a window.
    ///
    /// See [`EventCtx::submit_command`] for more information.
    ///
    /// [`Command`]: crate::Command
    /// [`EventCtx::submit_command`]: crate::EventCtx::submit_command
    /// [`Target::Auto`]: crate::Target::Auto
    /// [`Target::Global`]: crate::Target::Global
    pub fn submit_command(&mut self, cmd: impl Into<Command>) {
        self.queue.push_back(
            cmd.into()
                .default_to(self.window.map(Target::Window).unwrap_or(Target::Global)),
        );
    }
}

impl MenuItem {
    /// Create a new menu item with a given name.
    pub fn new(title: impl Into<String>) -> MenuItem {
        let mut id = COUNTER.next() as u32;
        if id == 0 {
            id = COUNTER.next() as u32;
        }
        MenuItem {
            id: MenuItemId(std::num::NonZeroU32::new(id)),
            title: title.into(),
            callback: None,
            hotkey: None,
            selected: None,
            enabled: None,
            state: None,
        }
    }

    /// Provide a callback that will be invoked when this menu item is chosen.
    pub fn on_activate(mut self, on_activate: impl FnMut(&mut MenuEventCtx) + 'static) -> Self {
        self.callback = Some(Box::new(on_activate));
        self
    }

    /// Provide a [`Command`] that will be sent when this menu item is chosen.
    ///
    /// This is equivalent to `self.on_activate(move |ctx, _data, _env| ctx.submit_command(cmd))`.
    /// If the command's target is [`Target::Auto`], it will be sent to the menu's window if the
    /// menu is associated with a window, or to [`Target::Global`] if the menu is not associated
    /// with a window.
    ///
    /// [`Command`]: crate::Command
    /// [`Target::Auto`]: crate::Target::Auto
    /// [`Target::Global`]: crate::Target::Global
    pub fn command(self, cmd: impl Into<Command>) -> Self {
        let cmd = cmd.into();
        self.on_activate(move |ctx| ctx.submit_command(cmd.clone()))
    }

    /// Provide a hotkey for activating this menu item.
    ///
    /// This is equivalent to
    /// `self.dynamic_hotkey(move |_, _| Some(HotKey::new(mods, key))`
    pub fn hotkey(self, mods: impl Into<Option<RawMods>>, key: impl IntoKey) -> Self {
        let hotkey = HotKey::new(mods, key);
        self.dynamic_hotkey(move || Some(hotkey.clone()))
    }

    /// Provide a dynamic hotkey for activating this menu item.
    ///
    /// The hotkey can change depending on the data.
    pub fn dynamic_hotkey(mut self, hotkey: impl FnMut() -> Option<HotKey> + 'static) -> Self {
        self.hotkey = Some(Box::new(hotkey));
        self
    }

    // Panics if we haven't been resolved.
    fn text(&self) -> &str {
        &self.state.as_ref().unwrap().title
    }

    // Panics if we haven't been resolved.
    fn is_enabled(&self) -> bool {
        self.state.as_ref().unwrap().enabled
    }

    fn resolve(&mut self) {
        let new_state = MenuItemState {
            title: self.title.clone(),
            hotkey: self.hotkey.as_mut().and_then(|h| h()),
            selected: self.selected.as_mut().map(|s| s()).unwrap_or(false),
            enabled: self.enabled.as_mut().map(|e| e()).unwrap_or(true),
        };
        self.state = Some(new_state);
    }
}

/// macOS.
pub mod mac {
    use super::*;

    /// A basic macOS menu bar.
    pub fn menu_bar() -> Menu {
        Menu::new(String::from(""))
            .entry(application::default())
            .entry(file::default())
    }

    /// The application menu
    pub mod application {
        use druid_shell::SysMods;

        use crate::commands;

        use super::*;

        /// The default Application menu.
        pub fn default() -> Menu {
            #[allow(deprecated)]
            Menu::new(String::from("macos-menu-application-menu"))
                .entry(about())
                .separator()
                .entry(preferences())
                .separator()
                //.entry(MenuDesc::new(String::from("macos-menu-services")))
                .entry(hide())
                .entry(hide_others())
                .entry(show_all())
                .separator()
                .entry(quit())
        }

        /// The 'About App' menu item.
        pub fn about() -> MenuItem {
            MenuItem::new(String::from("macos-menu-about-app")).command(commands::sys::SHOW_ABOUT)
        }

        /// The preferences menu item.
        pub fn preferences() -> MenuItem {
            MenuItem::new(String::from("macos-menu-preferences"))
                .command(commands::sys::SHOW_PREFERENCES)
                .hotkey(SysMods::Cmd, ",")
        }

        /// The 'Hide' builtin menu item.
        #[cfg_attr(
            not(target_os = "macos"),
            deprecated = "hide does nothing on platforms other than macOS"
        )]
        pub fn hide() -> MenuItem {
            #[allow(deprecated)]
            MenuItem::new(String::from("macos-menu-hide-app"))
                .command(commands::sys::HIDE_APPLICATION)
                .hotkey(SysMods::Cmd, "h")
        }

        /// The 'Hide Others' builtin menu item.
        #[cfg_attr(
            not(target_os = "macos"),
            deprecated = "hide_others does nothing on platforms other than macOS"
        )]
        pub fn hide_others() -> MenuItem {
            #[allow(deprecated)]
            MenuItem::new(String::from("macos-menu-hide-others"))
                .command(commands::sys::HIDE_OTHERS)
                .hotkey(SysMods::AltCmd, "h")
        }

        /// The 'show all' builtin menu item
        //FIXME: this doesn't work
        pub fn show_all() -> MenuItem {
            MenuItem::new(String::from("macos-menu-show-all")).command(commands::sys::SHOW_ALL)
        }

        /// The 'Quit' menu item.
        pub fn quit() -> MenuItem {
            MenuItem::new(String::from("macos-menu-quit-app"))
                .command(commands::sys::QUIT_APP)
                .hotkey(SysMods::Cmd, "q")
        }
    }

    /// The file menu.
    pub mod file {
        use druid_shell::SysMods;

        use super::*;

        use crate::commands;

        /// A default file menu.
        ///
        /// This will not be suitable for many applications; you should
        /// build the menu you need manually, using the items defined here
        /// where appropriate.
        pub fn default() -> Menu {
            Menu::new(String::from("common-menu-file-menu")).entry(close())
        }

        /// The 'Close' menu item.
        pub fn close() -> MenuItem {
            MenuItem::new(String::from("common-menu-file-close"))
                .command(commands::sys::CLOSE_WINDOW)
                .hotkey(SysMods::Cmd, "w")
        }
    }
}
