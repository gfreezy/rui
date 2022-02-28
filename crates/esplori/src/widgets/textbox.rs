use crate::box_constraints::BoxConstraints;
use crate::constraints::Constraints;
use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
use crate::text::editor::Editor;
use crate::text::layout::{LayoutMetrics, TextLayout};
use crate::text::selection::Selection;
use crate::text::text_input::{BasicTextInput, EditAction, TextInput};
use crate::tree::Children;
use crate::ui::Ui;
use druid_shell::kurbo::{Affine, Insets, Point, Size, Vec2};
use druid_shell::piet::{Color, PietText, RenderContext, TextAlignment};
use druid_shell::TimerToken;
use std::panic::Location;
use std::time::Duration;

const CURSOR_BLINK_DURATION: Duration = Duration::from_millis(500);

pub struct TextBox {
    placeholder: String,
    editable: String,
    alignment: TextAlignment,
    text_size: f64,
    width: Option<f64>,
    on_changed: Box<dyn FnMut(String) + 'static>,
}

impl PartialEq for TextBox {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl TextBox {
    pub fn new(text: String) -> Self {
        TextBox {
            placeholder: "".to_string(),
            editable: text,
            alignment: TextAlignment::Start,
            on_changed: Box::new(|_| {}),
            text_size: 14.,
            width: None,
        }
    }

    pub fn placeholder(mut self, text: String) -> Self {
        self.placeholder = text;
        self
    }

    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn text_size(mut self, size: f64) -> Self {
        self.text_size = size;
        self
    }

    pub fn on_changed(mut self, on_changed: impl FnMut(String) + 'static) -> Self {
        self.on_changed = Box::new(on_changed);
        self
    }

    pub fn build(self, ui: &mut Ui) -> bool {
        let caller = Location::caller().into();
        ui.render_object(caller, self, |_| {})
    }
}

impl Properties for TextBox {
    type Object = TextBoxObject;
}

pub struct TextBoxObject {
    placeholder: TextLayout<String>,
    text: String,
    editor: Editor<String>,
    alignment: TextAlignment,
    text_size: f64,
    activated: bool,
    width: Option<f64>,

    // this can be Box<dyn TextInput> in the future
    input_handler: BasicTextInput,
    hscroll_offset: f64,
    // in cases like SelectAll, we don't adjust the viewport after an event.
    suppress_adjust_hscroll: bool,
    cursor_timer: TimerToken,
    cursor_on: bool,
    alignment_offset: f64,
    text_pos: Point,
    /// true if a click event caused us to gain focus.
    ///
    /// On macOS, if focus happens via click then we set the selection based
    /// on the click position; if focus happens automatically (e.g. on tab)
    /// then we select our entire contents.
    was_focused_from_click: bool,
    on_changed: Box<dyn FnMut(String)>,
}

impl TextBoxObject {
    /// The point, relative to the origin, where this text box draws its
    /// [`TextLayout`].
    ///
    /// This is exposed in case the user wants to do additional drawing based
    /// on properties of the text.
    ///
    /// This is not valid until `layout` has been called.
    pub fn text_position(&self) -> Point {
        self.text_pos
    }
}

impl RenderObject<TextBox> for TextBoxObject {
    type Action = bool;

    fn create(props: TextBox) -> Self {
        let mut editor = Editor::from_text(&*props.editable);
        editor.layout_mut().set_text_size(props.text_size);

        TextBoxObject {
            placeholder: TextLayout::from_text(props.placeholder),
            text: String::from(&*props.editable),
            editor,
            input_handler: BasicTextInput::default(),
            activated: false,
            text_size: props.text_size,
            width: props.width,
            hscroll_offset: 0.,
            suppress_adjust_hscroll: false,
            cursor_timer: TimerToken::INVALID,
            cursor_on: false,
            alignment: TextAlignment::Start,
            alignment_offset: 0.0,
            text_pos: Point::ZERO,
            was_focused_from_click: false,
            on_changed: props.on_changed,
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: TextBox) -> Self::Action {
        if self.text != props.editable {
            self.text = props.editable.to_owned();
            self.editor.set_text(self.text.clone());
            ctx.request_layout();
        }

        if self.text_size != props.text_size {
            self.editor.layout_mut().set_text_size(props.text_size);
            ctx.request_layout();
        }
        if props.width != self.width {
            self.width = props.width;
            ctx.request_layout();
        }
        if Some(&props.placeholder) != self.placeholder.text() {
            self.placeholder.set_text(props.placeholder.to_owned());
            ctx.request_layout();
        }
        if props.alignment != self.alignment {
            self.alignment = props.alignment;
            ctx.request_layout();
        }
        self.on_changed = props.on_changed;

        let was_activated = self.activated;
        self.activated = false;
        was_activated
    }
}

impl RenderObjectInterface for TextBoxObject {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _children: &mut Children) {
        self.suppress_adjust_hscroll = false;
        let mut new_text = self.text.clone();
        match event {
            Event::MouseDown(mouse) => {
                // ctx.request_focus();
                ctx.set_active(true);
                let mut mouse = mouse.clone();
                mouse.pos += Vec2::new(self.hscroll_offset - self.alignment_offset, 0.0);

                if !mouse.focus {
                    self.was_focused_from_click = true;
                    self.reset_cursor_blink(ctx.request_timer(CURSOR_BLINK_DURATION));
                    self.editor.click(&mouse, &mut new_text);
                }

                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                let mut mouse = mouse.clone();
                mouse.pos += Vec2::new(self.hscroll_offset - self.alignment_offset, 0.0);
                if ctx.is_active() {
                    self.editor.drag(&mouse, &mut new_text);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    ctx.request_paint();
                }
            }
            Event::Timer(id) => {
                if *id == self.cursor_timer {
                    self.cursor_on = !self.cursor_on;
                    ctx.request_paint();
                    self.cursor_timer = ctx.request_timer(CURSOR_BLINK_DURATION);
                }
            }
            // Event::Command(ref cmd) if ctx.is_focused() && cmd.is(crate::commands::COPY) => {
            //     self.editor.copy(data);
            //     ctx.set_handled();
            // }
            // Event::Command(ref cmd) if ctx.is_focused() && cmd.is(crate::commands::CUT) => {
            //     self.editor.cut(data);
            //     ctx.set_handled();
            // }
            // Event::Command(cmd) if cmd.is(TextBox::PERFORM_EDIT) => {
            //     let edit = cmd.get_unchecked(TextBox::PERFORM_EDIT);
            //     self.editor.do_edit(edit.to_owned(), data);
            // }
            Event::Paste(ref item) => {
                if let Some(string) = item.get_string() {
                    self.editor.paste(string, &mut new_text);
                }
            }
            Event::KeyDown(key_event) => {
                match key_event {
                    // Tab and shift+tab
                    // k_e if HotKey::new(None, KbKey::Tab).matches(k_e) => ctx.focus_next(),
                    // k_e if HotKey::new(SysMods::Shift, KbKey::Tab).matches(k_e) => ctx.focus_prev(),
                    // k_e if !self.editor.multiline()
                    //     && HotKey::new(None, KbKey::Enter).matches(k_e) =>
                    // {
                    //     self.activated = true;
                    //     ctx.request_update();
                    // }
                    // k_e if HotKey::new(SysMods::Cmd, KbKey::Enter).matches(k_e) => {
                    //     // TODO: Figure out if Cmd/Ctrl is the right modifier for this.
                    //     self.activated = true;
                    //     ctx.request_update();
                    // }
                    k_e => {
                        if let Some(edit) = self.input_handler.handle_event(k_e) {
                            self.suppress_adjust_hscroll = matches!(edit, EditAction::SelectAll);
                            self.editor.do_edit(edit, &mut new_text);
                            ctx.request_update();
                            ctx.request_paint();
                        }
                    }
                };
                self.reset_cursor_blink(ctx.request_timer(CURSOR_BLINK_DURATION));
                ctx.request_paint();
            }
            _ => (),
        }
        if &self.text != &new_text {
            (self.on_changed)(new_text);
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        _children: &mut Children,
    ) -> Size {
        let bc: BoxConstraints = c.into();

        let width = self.width.unwrap_or(f64::INFINITY);

        self.placeholder.rebuild_if_needed(&mut ctx.text());
        if self.editor.multiline() {
            self.editor.set_wrap_width(bc.max().width.min(width));
        }
        self.editor.rebuild_if_needed(&mut ctx.text());

        let text_metrics = if self.text.is_empty() {
            self.placeholder.layout_metrics()
        } else {
            self.editor.layout().layout_metrics()
        };

        let height = text_metrics.size.height;
        let size = bc.constrain((width, height));
        // if we have a non-left text-alignment, we need to manually adjust our position.
        self.update_alignment_adjustment(size.width, &text_metrics);

        let bottom_padding = (size.height - text_metrics.size.height) / 2.0;
        let baseline_off =
            bottom_padding + (text_metrics.size.height - text_metrics.first_baseline);
        ctx.set_baseline_offset(baseline_off);

        size
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, _children: &mut Children) -> Size {
        let bc: BoxConstraints = c.into();

        let width = 200.0;
        let text_insets = Insets::uniform(3.0);

        self.placeholder.rebuild_if_needed(&mut ctx.text());
        if self.editor.multiline() {
            self.editor
                .set_wrap_width(bc.max().width - text_insets.x_value());
        }
        self.editor.rebuild_if_needed(&mut ctx.text());

        let text_metrics = if self.text.is_empty() {
            self.placeholder.layout_metrics()
        } else {
            self.editor.layout().layout_metrics()
        };

        let height = text_metrics.size.height + text_insets.y_value();
        let size = bc.constrain((width, height));
        // if we have a non-left text-alignment, we need to manually adjust our position.
        self.update_alignment_adjustment(size.width - text_insets.x_value(), &text_metrics);
        self.text_pos = Point::new(text_insets.x0 + self.alignment_offset, text_insets.y0);

        let bottom_padding = (size.height - text_metrics.size.height) / 2.0;
        let baseline_off =
            bottom_padding + (text_metrics.size.height - text_metrics.first_baseline);
        ctx.set_baseline_offset(baseline_off);

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _children: &mut Children) {
        let size = ctx.size();
        let background_color = Color::GRAY;
        let selection_color = Color::BLUE;
        let cursor_color = Color::WHITE;
        let border_width = 1.0;
        let text_insets = Insets::uniform(3.0);

        let is_focused = true;

        let border_color = if is_focused {
            Color::WHITE
        } else {
            Color::BLACK
        };

        // Paint the background
        let clip_rect = Size::new(size.width - border_width, size.height)
            .to_rect()
            .inset(-border_width / 2.0)
            .to_rounded_rect(3.0);

        ctx.fill(clip_rect, &background_color);

        // Render text, selection, and cursor inside a clip
        ctx.with_save(|rc| {
            rc.clip(clip_rect);

            // Shift everything inside the clip by the hscroll_offset
            rc.transform(Affine::translate((-self.hscroll_offset, 0.)));

            let text_pos = self.text_position();
            // Draw selection rect
            if !self.text.is_empty() {
                if is_focused {
                    for sel in self.editor.selection_rects() {
                        let sel = sel + text_pos.to_vec2();
                        let rounded = sel.to_rounded_rect(1.0);
                        rc.fill(rounded, &selection_color);
                    }
                }
                self.editor.draw(rc, text_pos);
            } else {
                self.placeholder.draw(rc, text_pos);
            }

            // Paint the cursor if focused and there's no selection
            if is_focused && self.should_draw_cursor() {
                // if there's no data, we always draw the cursor based on
                // our alignment.
                let cursor = if self.text.is_empty() {
                    let dx = match self.alignment {
                        TextAlignment::Start | TextAlignment::Justified => text_insets.x0,
                        TextAlignment::Center => size.width / 2.0,
                        TextAlignment::End => size.width - text_insets.x1,
                    };
                    self.editor.cursor_line() + Vec2::new(dx, text_insets.y0)
                } else {
                    // the cursor position can extend past the edge of the layout
                    // (commonly when there is trailing whitespace) so we clamp it
                    // to the right edge.
                    let mut cursor = self.editor.cursor_line() + text_pos.to_vec2();
                    let dx = size.width + self.hscroll_offset - text_insets.x0 - cursor.p0.x;
                    if dx < 0.0 {
                        cursor = cursor + Vec2::new(dx, 0.);
                    }
                    cursor
                };
                rc.stroke(cursor, &cursor_color, 1.);
            }
        });

        // Paint the border
        ctx.stroke(clip_rect, &border_color, border_width);
    }
}

impl TextBoxObject {
    /// Set the textbox's selection.
    pub fn set_selection(&mut self, selection: Selection) {
        self.editor.set_selection(selection);
    }

    /// Set the text and force the editor to update.
    ///
    /// This should be rarely needed; the main use-case would be if you need
    /// to manually set the text and then immediately do hit-testing or other
    /// tasks that rely on having an up-to-date text layout.
    pub fn force_rebuild(&mut self, text: String, factory: &mut PietText) {
        self.editor.set_text(text);
        self.editor.rebuild_if_needed(factory);
    }

    #[allow(dead_code)]
    // TODO: Figure out what this was good for.
    /// Calculate a stateful scroll offset
    fn update_hscroll(&mut self, self_width: f64) {
        let cursor_x = self.editor.cursor_line().p0.x;
        // if the text ends in trailing whitespace, that space is not included
        // in its reported width, but we need to include it for these calculations.
        // see https://github.com/linebender/druid/issues/1430
        let overall_text_width = self.editor.layout().size().width.max(cursor_x);
        let text_insets = Insets::ZERO;

        //// when advancing the cursor, we want some additional padding
        if overall_text_width < self_width - text_insets.x_value() {
            // There's no offset if text is smaller than text box
            //
            // [***I*  ]
            // ^
            self.hscroll_offset = 0.;
        } else if cursor_x > self_width - text_insets.x_value() + self.hscroll_offset {
            // If cursor goes past right side, bump the offset
            //       ->
            // **[****I]****
            //   ^
            self.hscroll_offset = cursor_x - self_width + text_insets.x_value();
        } else if cursor_x < self.hscroll_offset {
            // If cursor goes past left side, match the offset
            //    <-
            // **[I****]****
            //   ^
            self.hscroll_offset = cursor_x;
        } else if self.hscroll_offset > overall_text_width - self_width + text_insets.x_value() {
            // If the text is getting shorter, keep as small offset as possible
            //        <-
            // **[****I]
            //   ^
            self.hscroll_offset = overall_text_width - self_width + text_insets.x_value();
        }
    }

    fn reset_cursor_blink(&mut self, token: TimerToken) {
        self.cursor_on = true;
        self.cursor_timer = token;
    }

    // on macos we only draw the cursor if the selection is non-caret
    #[cfg(target_os = "macos")]
    fn should_draw_cursor(&self) -> bool {
        self.cursor_on && self.editor.selection().is_caret()
    }

    #[cfg(not(target_os = "macos"))]
    fn should_draw_cursor(&self) -> bool {
        self.cursor_on
    }

    fn update_alignment_adjustment(&mut self, available_width: f64, metrics: &LayoutMetrics) {
        self.alignment_offset = if self.editor.multiline() {
            0.0
        } else {
            let extra_space = (available_width - metrics.size.width).max(0.0);
            match self.alignment {
                TextAlignment::Start | TextAlignment::Justified => 0.0,
                TextAlignment::End => extra_space,
                TextAlignment::Center => extra_space / 2.0,
            }
        }
    }
}
