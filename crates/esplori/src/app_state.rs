use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use std::rc::Rc;

use druid_shell::kurbo::Size;
use druid_shell::piet::Piet;
use druid_shell::{
    Application, IdleToken, KeyEvent, MouseEvent, Region, Scale, TextFieldToken, TimerToken,
    WinHandler, WindowBuilder, WindowHandle,
};
use generational_indextree::Arena;
use tracing::debug;

use crate::app::{PendingWindow, WindowConfig};
use crate::command::{sys as sys_cmd, Command, Target};

use crate::event::Event;
use crate::ext_event::{ExtEventHost, ExtEventSink};
use crate::id::WindowId;

use crate::menu::{MenuItemId, MenuManager};
use crate::tree::Element;
use crate::window::Window;

pub(crate) const RUN_COMMANDS_TOKEN: IdleToken = IdleToken::new(1);

/// A token we are called back with if an external event was submitted.
pub(crate) const EXT_EVENT_IDLE_TOKEN: IdleToken = IdleToken::new(2);

/// An enum for specifying whether an event was handled.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Handled {
    /// An event was already handled, and shouldn't be propagated to other event handlers.
    Yes,
    /// An event has not yet been handled.
    No,
}

impl Handled {
    /// Has the event been handled yet?
    pub fn is_handled(self) -> bool {
        self == Handled::Yes
    }
}

impl From<bool> for Handled {
    /// Returns `Handled::Yes` if `handled` is true, and `Handled::No` otherwise.
    fn from(handled: bool) -> Handled {
        if handled {
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

/// State shared by all windows in the UI.
#[derive(Clone)]
pub(crate) struct AppState {
    inner: Rc<RefCell<InnerAppState>>,
}
/// Our queue type
pub(crate) type CommandQueue = VecDeque<Command>;

pub struct InnerAppState {
    app: Application,
    command_queue: CommandQueue,
    root_menu: Option<MenuManager>,
    ext_event_host: ExtEventHost,
    windows: Windows,
    arena: Arena<Element>,
}

#[derive(Default)]
/// All active windows.
struct Windows {
    pending: HashMap<WindowId, PendingWindow>,
    windows: HashMap<WindowId, Window>,
}

impl Windows {
    fn connect(
        &mut self,
        id: WindowId,
        arena: Arena<Element>,
        handle: WindowHandle,
        ext_handle: ExtEventSink,
    ) {
        if let Some(pending) = self.pending.remove(&id) {
            let win = Window::new(id, arena, handle, pending, ext_handle);
            assert!(self.windows.insert(id, win).is_none(), "duplicate window");
        } else {
            tracing::error!("no window for connecting handle {:?}", id);
        }
    }

    fn add(&mut self, id: WindowId, win: PendingWindow) {
        assert!(self.pending.insert(id, win).is_none(), "duplicate pending");
    }

    fn remove(&mut self, id: WindowId) -> Option<Window> {
        self.windows.remove(&id)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut Window> {
        self.windows.values_mut()
    }

    fn get(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    fn get_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    fn count(&self) -> usize {
        self.windows.len() + self.pending.len()
    }
}

impl AppState {
    pub(crate) fn new(app: Application, ext_event_host: ExtEventHost) -> Self {
        let inner = Rc::new(RefCell::new(InnerAppState {
            app,
            root_menu: None,
            command_queue: VecDeque::new(),
            windows: Windows::default(),
            ext_event_host,
            arena: Arena::new(),
        }));

        AppState { inner }
    }

    pub(crate) fn app(&self) -> Application {
        self.inner.borrow().app.clone()
    }
}

impl InnerAppState {
    fn handle_menu_cmd(&mut self, cmd_id: MenuItemId, window_id: Option<WindowId>) {
        debug!("handle menu cmd {window_id:?}");
        let queue = &mut self.command_queue;
        match window_id {
            Some(id) => self.windows.get_mut(id).map(|w| w.menu_cmd(queue, cmd_id)),
            None => self
                .root_menu
                .as_mut()
                .map(|m| m.event(queue, None, cmd_id)),
        };
    }

    fn connect(&mut self, id: WindowId, handle: WindowHandle) {
        self.windows
            .connect(id, self.arena, handle, self.ext_event_host.make_sink());

        // If the external event host has no handle, it cannot wake us
        // when an event arrives.
        if self.ext_event_host.handle_window_id.is_none() {
            self.set_ext_event_idle_handler(id);
        }
    }

    /// Called after this window has been closed by the platform.
    ///
    /// We clean up resources and notifiy the delegate, if necessary.
    fn remove_window(&mut self, window_id: WindowId) {
        // when closing the last window:
        if let Some(mut win) = self.windows.remove(window_id) {
            if self.windows.windows.is_empty() {
                // on mac we need to keep the menu around
                self.root_menu = win.menu.take();
                // If there are even no pending windows, we quit the run loop.
                if self.windows.count() == 0 {
                    #[cfg(any(target_os = "windows", feature = "x11"))]
                    self.app.quit();
                }
            }
        }

        // if we are closing the window that is currently responsible for
        // waking us when external events arrive, we want to pass that responsibility
        // to another window.
        if self.ext_event_host.handle_window_id == Some(window_id) {
            self.ext_event_host.handle_window_id = None;
            // find any other live window
            let win_id = self.windows.windows.keys().find(|k| *k != &window_id);
            if let Some(any_other_window) = win_id.cloned() {
                self.set_ext_event_idle_handler(any_other_window);
            }
        }
    }

    /// Set the idle handle that will be used to wake us when external events arrive.
    fn set_ext_event_idle_handler(&mut self, id: WindowId) {
        if let Some(mut idle) = self
            .windows
            .get_mut(id)
            .and_then(|win| win.handle.get_idle_handle())
        {
            if self.ext_event_host.has_pending_items() {
                idle.schedule_idle(EXT_EVENT_IDLE_TOKEN);
            }
            self.ext_event_host.set_idle(idle, id);
        }
    }

    /// triggered by a menu item or other command.
    ///
    /// This doesn't close the window; it calls the close method on the platform
    /// window handle; the platform should close the window, and then call
    /// our handlers `destroy()` method, at which point we can do our cleanup.
    fn request_close_window(&mut self, window_id: WindowId) {
        if let Some(win) = self.windows.get_mut(window_id) {
            win.handle.close();
        }
    }

    /// Requests the platform to close all windows.
    fn request_close_all_windows(&mut self) {
        for win in self.windows.iter_mut() {
            win.handle.close();
        }
    }

    fn show_window(&mut self, id: WindowId) {
        if let Some(win) = self.windows.get_mut(id) {
            win.handle.bring_to_front_and_focus();
        }
    }

    fn prepare_paint(&mut self, window_id: WindowId) {
        if let Some(win) = self.windows.get_mut(window_id) {
            win.prepare_paint();
        }
        // self.do_update();
    }

    fn paint(&mut self, window_id: WindowId, piet: &mut Piet, invalid: &Region) {
        if let Some(win) = self.windows.get_mut(window_id) {
            win.paint(piet, invalid, &mut self.command_queue);
        }
    }

    fn dispatch_cmd(&mut self, cmd: Command) -> Handled {
        match cmd.target() {
            Target::Window(id) => {
                if cmd.is(sys_cmd::SHOW_CONTEXT_MENU) {
                    self.show_context_menu(id, &cmd);
                    return Handled::Yes;
                }
                if let Some(w) = self.windows.get_mut(id) {
                    return if cmd.is(sys_cmd::CLOSE_WINDOW) {
                        let handled = w.event(&mut self.command_queue, Event::WindowCloseRequested);
                        if !handled.is_handled() {
                            w.event(&mut self.command_queue, Event::WindowDisconnected);
                        }
                        handled
                    } else {
                        w.event(&mut self.command_queue, Event::Command(cmd))
                    };
                }
            }
            // in this case we send it to every window that might contain
            // this widget, breaking if the event is handled.
            Target::Widget(_id) => {
                unimplemented!()
            }
            Target::Global => {
                for w in self.windows.iter_mut() {
                    let event = Event::Command(cmd.clone());
                    if w.event(&mut self.command_queue, event).is_handled() {
                        return Handled::Yes;
                    }
                }
            }
            Target::Auto => {
                tracing::error!("{:?} reached window handler with `Target::Auto`", cmd);
            }
        }
        Handled::No
    }

    fn do_window_event(&mut self, source_id: WindowId, event: Event) -> Handled {
        match event {
            Event::Command(..) => {
                panic!("commands should be dispatched via dispatch_cmd");
            }
            _ => (),
        }

        if let Some(win) = self.windows.get_mut(source_id) {
            win.event(&mut self.command_queue, event)
        } else {
            Handled::No
        }
    }

    fn show_context_menu(&mut self, window_id: WindowId, _cmd: &Command) {
        if let Some(_win) = self.windows.get_mut(window_id) {
            unimplemented!()
        }
    }

    fn do_update(&mut self) {
        // we send `update` to all windows, not just the active one:
        for window in self.windows.iter_mut() {
            window.update(&mut self.command_queue);
        }
        self.invalidate_and_finalize();
    }

    /// invalidate any window handles that need it.
    ///
    /// This should always be called at the end of an event update cycle,
    /// including for lifecycle events.
    fn invalidate_and_finalize(&mut self) {
        for win in self.windows.iter_mut() {
            win.invalidate_and_finalize();
        }
    }
}

impl AppState {
    pub(crate) fn add_window(&self, id: WindowId, window: PendingWindow) {
        self.inner.borrow_mut().windows.add(id, window);
    }

    fn connect_window(&mut self, window_id: WindowId, handle: WindowHandle) {
        self.inner.borrow_mut().connect(window_id, handle)
    }

    fn remove_window(&mut self, window_id: WindowId) {
        self.inner.borrow_mut().remove_window(window_id)
    }

    /// Send an event to the widget hierarchy.
    ///
    /// Returns `true` if the event produced an action.
    ///
    /// This is principally because in certain cases (such as keydown on Windows)
    /// the OS needs to know if an event was handled.
    fn do_window_event(&mut self, event: Event, window_id: WindowId) -> Handled {
        let result = self.inner.borrow_mut().do_window_event(window_id, event);
        self.process_commands();
        self.inner.borrow_mut().do_update();
        result
    }

    fn prepare_paint_window(&mut self, window_id: WindowId) {
        self.inner.borrow_mut().prepare_paint(window_id);
    }

    fn paint_window(&mut self, window_id: WindowId, piet: &mut Piet, invalid: &Region) {
        self.inner.borrow_mut().paint(window_id, piet, invalid);
    }

    fn idle(&mut self, token: IdleToken) {
        match token {
            RUN_COMMANDS_TOKEN => {
                self.process_commands();
                self.inner.borrow_mut().do_update();
            }
            EXT_EVENT_IDLE_TOKEN => {
                self.process_ext_events();
                self.process_commands();
                self.inner.borrow_mut().do_update();
            }
            other => tracing::warn!("unexpected idle token {:?}", other),
        }
    }

    pub(crate) fn handle_idle_callback(&mut self, cb: impl FnOnce()) {
        let mut inner = self.inner.borrow_mut();
        cb();
        inner.do_update();
    }

    fn process_commands(&mut self) {
        loop {
            let next_cmd = self.inner.borrow_mut().command_queue.pop_front();
            match next_cmd {
                Some(cmd) => self.handle_cmd(cmd),
                None => break,
            }
        }
    }

    fn process_ext_events(&mut self) {
        loop {
            let ext_cmd = self.inner.borrow_mut().ext_event_host.recv();
            match ext_cmd {
                Some(cmd) => self.handle_cmd(cmd),
                None => break,
            }
        }
    }

    /// Handle a 'command' message from druid-shell. These map to  an item
    /// in an application, window, or context (right-click) menu.
    ///
    /// If the menu is  associated with a window (the general case) then
    /// the `window_id` will be `Some(_)`, otherwise (such as if no window
    /// is open but a menu exists, as on macOS) it will be `None`.
    fn handle_system_cmd(&mut self, cmd_id: u32, window_id: Option<WindowId>) {
        self.inner
            .borrow_mut()
            .handle_menu_cmd(MenuItemId::new(cmd_id), window_id);
        self.process_commands();
        self.inner.borrow_mut().do_update();
    }

    /// Handle a command. Top level commands (e.g. for creating and destroying
    /// windows) have their logic here; other commands are passed to the window.
    fn handle_cmd(&mut self, cmd: Command) {
        use crate::command::Target as T;
        match cmd.target() {
            // these are handled the same no matter where they come from
            _ if cmd.is(sys_cmd::QUIT_APP) => self.quit(),
            #[cfg(target_os = "macos")]
            _ if cmd.is(sys_cmd::HIDE_APPLICATION) => self.hide_app(),
            #[cfg(target_os = "macos")]
            _ if cmd.is(sys_cmd::HIDE_OTHERS) => self.hide_others(),
            _ if cmd.is(sys_cmd::CLOSE_ALL_WINDOWS) => self.request_close_all_windows(),
            T::Window(id) if cmd.is(sys_cmd::INVALIDATE_IME) => self.invalidate_ime(cmd, id),
            // these should come from a window
            // FIXME: we need to be able to open a file without a window handle
            T::Window(id) if cmd.is(sys_cmd::CLOSE_WINDOW) => {
                if !self.inner.borrow_mut().dispatch_cmd(cmd).is_handled() {
                    self.request_close_window(id);
                }
            }
            T::Window(id) if cmd.is(sys_cmd::SHOW_WINDOW) => self.show_window(id),
            T::Window(id) if cmd.is(sys_cmd::PASTE) => self.do_paste(id),
            _ if cmd.is(sys_cmd::CLOSE_WINDOW) => {
                tracing::warn!("CLOSE_WINDOW command must target a window.")
            }
            _ if cmd.is(sys_cmd::SHOW_WINDOW) => {
                tracing::warn!("SHOW_WINDOW command must target a window.")
            }
            _ if cmd.is(sys_cmd::SHOW_OPEN_PANEL) => {
                tracing::warn!("SHOW_OPEN_PANEL command must target a window.")
            }
            _ => {
                self.inner.borrow_mut().dispatch_cmd(cmd);
            }
        }
    }

    fn request_close_window(&mut self, id: WindowId) {
        self.inner.borrow_mut().request_close_window(id);
    }

    fn request_close_all_windows(&mut self) {
        self.inner.borrow_mut().request_close_all_windows();
    }

    fn show_window(&mut self, id: WindowId) {
        self.inner.borrow_mut().show_window(id);
    }

    fn configure_window(&mut self, _cmd: Command, _id: WindowId) {}

    fn do_paste(&mut self, window_id: WindowId) {
        let event = Event::Paste(self.inner.borrow().app.clipboard());
        self.inner.borrow_mut().do_window_event(window_id, event);
    }

    fn invalidate_ime(&mut self, _cmd: Command, _id: WindowId) {}

    fn release_ime_lock(&mut self, _window_id: WindowId, _token: TextFieldToken) {}

    fn quit(&self) {
        self.inner.borrow().app.quit()
    }

    #[cfg(target_os = "macos")]
    fn hide_app(&self) {
        use druid_shell::platform::mac::ApplicationExt as _;
        self.inner.borrow().app.hide()
    }

    #[cfg(target_os = "macos")]
    fn hide_others(&mut self) {
        use druid_shell::platform::mac::ApplicationExt as _;
        self.inner.borrow().app.hide_others();
    }
    pub(crate) fn build_native_window(
        &mut self,
        id: WindowId,
        mut pending: PendingWindow,
        config: WindowConfig,
    ) -> Result<WindowHandle, druid_shell::Error> {
        let mut builder = WindowBuilder::new(self.app());
        config.apply_to_builder(&mut builder);

        pending.size_policy = config.size_policy;
        builder.set_title(pending.title.clone());

        let platform_menu = pending.menu.as_mut().map(|m| m.initialize(Some(id)));
        if let Some(menu) = platform_menu {
            builder.set_menu(menu);
        }

        let handler = WindowHandler::new_shared((*self).clone(), id);
        builder.set_handler(Box::new(handler));

        self.add_window(id, pending);
        builder.build()
    }
}

pub struct AppHandler {
    app_state: AppState,
}

impl AppHandler {
    pub(crate) fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

impl druid_shell::AppHandler for AppHandler {
    fn command(&mut self, id: u32) {
        self.app_state.handle_system_cmd(id, None)
    }
}

pub struct WindowHandler {
    /// The shared app state.
    pub(crate) app_state: AppState,
    /// The id for the current window.
    window_id: WindowId,
}

impl WindowHandler {
    pub(crate) fn new_shared(app_state: AppState, window_id: WindowId) -> Self {
        Self {
            app_state,
            window_id,
        }
    }
}

impl WinHandler for WindowHandler {
    // #[instrument(skip(self, handle))]
    fn connect(&mut self, handle: &WindowHandle) {
        self.app_state
            .connect_window(self.window_id, handle.clone());

        let event = Event::WindowConnected;
        self.app_state.do_window_event(event, self.window_id);
    }

    fn prepare_paint(&mut self) {
        self.app_state.prepare_paint_window(self.window_id);
    }

    fn paint(&mut self, piet: &mut Piet, region: &Region) {
        self.app_state.paint_window(self.window_id, piet, region);
    }

    fn size(&mut self, size: Size) {
        let event = Event::WindowSize(size);
        self.app_state.do_window_event(event, self.window_id);
    }

    fn scale(&mut self, _scale: Scale) {
        // TODO: Do something with the scale
    }

    fn command(&mut self, id: u32) {
        self.app_state.handle_system_cmd(id, Some(self.window_id));
    }

    fn mouse_down(&mut self, event: &MouseEvent) {
        // TODO: double-click detection (or is this done in druid-shell?)
        let event = Event::MouseDown(event.clone().into());
        self.app_state.do_window_event(event, self.window_id);
    }

    fn mouse_up(&mut self, event: &MouseEvent) {
        let event = Event::MouseUp(event.clone().into());
        self.app_state.do_window_event(event, self.window_id);
    }

    fn mouse_move(&mut self, event: &MouseEvent) {
        let event = Event::MouseMove(event.clone().into());
        self.app_state.do_window_event(event, self.window_id);
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        self.app_state
            .do_window_event(Event::KeyDown(event), self.window_id)
            .is_handled()
    }

    fn key_up(&mut self, event: KeyEvent) {
        self.app_state
            .do_window_event(Event::KeyUp(event), self.window_id);
    }

    fn wheel(&mut self, event: &MouseEvent) {
        self.app_state
            .do_window_event(Event::Wheel(event.clone().into()), self.window_id);
    }

    fn zoom(&mut self, delta: f64) {
        let event = Event::Zoom(delta);
        self.app_state.do_window_event(event, self.window_id);
    }

    fn timer(&mut self, token: TimerToken) {
        self.app_state
            .do_window_event(Event::Timer(token), self.window_id);
    }

    fn idle(&mut self, token: IdleToken) {
        self.app_state.idle(token);
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn request_close(&mut self) {
        self.app_state
            .handle_cmd(sys_cmd::CLOSE_WINDOW.to(self.window_id));
        self.app_state.process_commands();
        self.app_state.inner.borrow_mut().do_update();
    }

    fn destroy(&mut self) {
        self.app_state.remove_window(self.window_id);
    }
}
