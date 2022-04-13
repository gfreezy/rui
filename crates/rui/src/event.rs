use druid_shell::kurbo::Size;
use druid_shell::{Clipboard, KeyEvent, MouseEvent, TimerToken};

use crate::commands::{Command, Notification};

#[derive(Debug, Clone)]
pub enum Event {
    /// Sent to all widgets in a given window when that window is first instantiated.
    ///
    /// This should always be the first `Event` received, although widgets will
    /// receive [`LifeCycle::WidgetAdded`] first.
    ///
    /// Widgets should handle this event if they need to do some addition setup
    /// when a window is first created.
    ///
    /// [`LifeCycle::WidgetAdded`]: enum.LifeCycle.html#variant.WidgetAdded
    WindowConnected,
    /// Sent to all widgets in a given window when the system requests to close the window.
    ///
    /// If the event is handled (with [`set_handled`]), the window will not be closed.
    /// All widgets are given an opportunity to handle this event; your widget should not assume
    /// that the window *will* close just because this event is received; for instance, you should
    /// avoid destructive side effects such as cleaning up resources.
    ///
    /// [`set_handled`]: crate::EventCtx::set_handled
    WindowCloseRequested,
    /// Sent to all widgets in a given window when the system is going to close that window.
    ///
    /// This event means the window *will* go away; it is safe to dispose of resources and
    /// do any other cleanup.
    WindowDisconnected,
    /// Called on the root widget when the window size changes.
    ///
    /// Discussion: it's not obvious this should be propagated to user
    /// widgets. It *is* propagated through the RootWidget and handled
    /// in the WindowPod, but after that it might be considered better
    /// to just handle it in `layout`.
    WindowSize(Size),
    /// Called when a mouse button is pressed.
    MouseDown(MouseEvent),
    /// Called when a mouse button is released.
    MouseUp(MouseEvent),
    /// Called when the mouse is moved.
    ///
    /// The `MouseMove` event is propagated to the active widget, if
    /// there is one, otherwise to hot widgets (see `HotChanged`).
    /// If a widget loses its hot status due to `MouseMove` then that specific
    /// `MouseMove` event is also still sent to that widget.
    ///
    /// The `MouseMove` event is also the primary mechanism for widgets
    /// to set a cursor, for example to an I-bar inside a text widget. A
    /// simple tactic is for the widget to unconditionally call
    /// [`set_cursor`] in the MouseMove handler, as `MouseMove` is only
    /// propagated to active or hot widgets.
    ///
    /// [`set_cursor`]: struct.EventCtx.html#method.set_cursor
    MouseMove(MouseEvent),
    /// Called when the mouse wheel or trackpad is scrolled.
    Wheel(MouseEvent),
    /// Called when a key is pressed.
    KeyDown(KeyEvent),
    /// Called when a key is released.
    ///
    /// Because of repeat, there may be a number `KeyDown` events before
    /// a corresponding `KeyUp` is sent.
    KeyUp(KeyEvent),
    /// Called when a paste command is received.
    Paste(Clipboard),
    /// Called when the trackpad is pinched.
    ///
    /// The value is a delta.
    Zoom(f64),
    /// Called on a timer event.
    ///
    /// Request a timer event through [`EventCtx::request_timer()`]. That will
    /// cause a timer event later.
    ///
    /// Note that timer events from other widgets may be delivered as well. Use
    /// the token returned from the `request_timer()` call to filter events more
    /// precisely.
    ///
    /// [`EventCtx::request_timer()`]: struct.EventCtx.html#method.request_timer
    Timer(TimerToken),
    /// Called at the beginning of a new animation frame.
    ///
    /// On the first frame when transitioning from idle to animating, `interval`
    /// will be 0. (This logic is presently per-window but might change to
    /// per-widget to make it more consistent). Otherwise it is in nanoseconds.
    ///
    /// The `paint` method will be called shortly after this event is finished.
    /// As a result, you should try to avoid doing anything computationally
    /// intensive in response to an `AnimFrame` event: it might make Druid miss
    /// the monitor's refresh, causing lag or jerky animation.
    AnimFrame(u64),
    /// An event containing a [`Command`] to be handled by the widget.
    ///
    /// [`Command`]s are messages, optionally with attached data, that can
    /// may be generated from a number of sources:
    ///
    /// - If your application uses  menus (either window or context menus)
    /// then the [`MenuItem`]s in the menu will each correspond to a `Command`.
    /// When the menu item is selected, that [`Command`] will be delivered to
    /// the root widget of the appropriate window.
    /// - If you are doing work in another thread (using an [`ExtEventSink`])
    /// then [`Command`]s are the mechanism by which you communicate back to
    /// the main thread.
    /// - Widgets and other Druid components can send custom [`Command`]s at
    /// runtime, via methods such as [`EventCtx::submit_command`].
    ///
    /// [`Command`]: struct.Command.html
    /// [`Widget`]: trait.Widget.html
    /// [`EventCtx::submit_command`]: struct.EventCtx.html#method.submit_command
    /// [`ExtEventSink`]: crate::ExtEventSink
    /// [`MenuItem`]: crate::MenuItem
    Command(Command),
    /// A [`Notification`] from one of this widget's descendants.
    ///
    /// While handling events, widgets can submit notifications to be
    /// delivered to their ancestors immdiately after they return.
    ///
    /// If you handle a [`Notification`], you should call [`EventCtx::set_handled`]
    /// to stop the notification from being delivered to further ancestors.
    ///
    /// ## Special considerations
    ///
    /// Notifications are slightly different from other events; they originate
    /// inside Druid, and they are delivered as part of the handling of another
    /// event. In this sense, they can sort of be thought of as an augmentation
    /// of an event; they are a way for multiple widgets to coordinate the
    /// handling of an event.
    ///
    /// [`EventCtx::set_handled`]: crate::EventCtx::set_handled
    Notification(Notification),
    /// Sent to a widget when the platform may have mutated shared IME state.
    ///
    /// This is sent to a widget that has an attached IME session anytime the
    /// platform has released a mutable lock on shared state.
    ///
    /// This does not *mean* that any state has changed, but the widget
    /// should check the shared state, perform invalidation, and update `Data`
    /// as necessary.
    ImeStateChange,
}
