#[macro_use]
mod macros;
mod arithmatic;
mod diagnostics;
mod render_object;
mod rendering;

use druid_shell::{
    piet::Piet, Application, HotKey, Menu, MouseEvent, SysMods, WinHandler, WindowBuilder,
    WindowHandle,
};
use render_object::{
    pipeline_owner::PipelineOwner,
    render_box::{HitTestResult, Size},
    render_object::{PointerEvent, RenderObject},
};
use rendering::{
    render_flex::RenderFlex,
    render_pointer_listener::{HitTestBehavior, RenderPointerListener},
    render_text::RenderText,
};
use tracing::metadata::LevelFilter;

const QUIT_MENU_ID: u32 = 0x100;

struct MainState {
    handle: WindowHandle,
    root_view: Option<RenderObject>,
    pipeline_owner: Option<PipelineOwner>,
}

impl MainState {
    fn new() -> Self {
        MainState {
            handle: WindowHandle::default(),
            root_view: None,
            pipeline_owner: None,
        }
    }

    fn pipeline_owner(&self) -> &PipelineOwner {
        self.pipeline_owner.as_ref().unwrap()
    }

    fn root_view(&self) -> &RenderObject {
        self.root_view.as_ref().unwrap()
    }

    fn begin_frame(&mut self) {
        tracing::debug!("--------------- begin frame --------------");
    }

    fn draw_frame(&mut self, piet: &mut Piet) {
        tracing::debug!("--------------- draw frame --------------");
        self.pipeline_owner().flush_layout();
        self.pipeline_owner().flush_paint(piet);
        self.root_view().render_view().composite_frame(piet);
    }

    fn handle_pointer_event_immediately(&self, event: PointerEvent) {
        tracing::debug!("--------------- hit test --------------");
        let mut hit_test_result = HitTestResult::new();
        let position = event.position();
        self.root_view().hit_test(&mut hit_test_result, position);
        self.dispatch_event(event, &hit_test_result);
        self.handle.invalidate();
    }

    fn dispatch_event(&self, event: PointerEvent, hit_test_result: &HitTestResult) {
        tracing::debug!("--------------- dispatch event --------------");
        for entry in hit_test_result.entries() {
            tracing::debug!("dispatch event to: {:?}", entry.target());
            entry.target().handle_event(event.clone(), entry.clone());
        }

        // todo!("clean current");
    }

    fn root_object(&self) -> RenderObject {
        let flex = RenderObject::new_render_box(RenderFlex::new(
            style::axis::Axis::Vertical,
            style::layout::MainAxisSize::Max,
            style::layout::MainAxisAlignment::Center,
            style::layout::CrossAxisAlignment::Center,
            style::layout::TextDirection::Ltr,
            style::layout::VerticalDirection::Down,
        ));
        for i in 0..12 {
            let text = RenderObject::new_render_box(RenderText::new(i.to_string(), 16., None));
            let listener = RenderObject::new_render_box(RenderPointerListener::new(
                Some(Box::new(|ctx, e| {
                    let txt = ctx.first_child();
                    txt.update::<RenderText>(|r| {
                        r.set_font_size(&txt, r.font_size() + 4.);
                    });
                    tracing::debug!("pointer listener: {:?}", e);
                })),
                None,
                None,
                None,
                Some(HitTestBehavior::Opaque),
            ));
            listener.add(text);
            flex.add(listener);
        }
        flex
    }
}

impl WinHandler for MainState {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
        let root_node = self.root_object();
        let root_view = RenderObject::new_render_view(root_node, Size::from(handle.get_size()));
        let pipeline_owner = PipelineOwner::new(handle.text());
        pipeline_owner.set_render_view(&root_view);
        self.pipeline_owner = Some(pipeline_owner);
        self.root_view = Some(root_view);
        self.root_view().prepare_initial_frame();
    }

    fn prepare_paint(&mut self) {
        self.begin_frame();
    }

    fn paint(&mut self, piet: &mut Piet, _invalid: &druid_shell::Region) {
        self.draw_frame(piet);
    }

    fn mouse_up(&mut self, event: &druid_shell::MouseEvent) {
        self.handle_pointer_event_immediately(PointerEvent::MouseUp(event.clone()));
    }

    fn mouse_down(&mut self, event: &druid_shell::MouseEvent) {
        self.handle_pointer_event_immediately(PointerEvent::MouseDown(event.clone()));
    }

    fn mouse_move(&mut self, event: &druid_shell::MouseEvent) {
        // self.handle_pointer_event_immediately(PointerEvent::MouseMove(event.clone()));
    }

    fn command(&mut self, id: u32) {
        match id {
            QUIT_MENU_ID => {
                self.request_close();
            }
            _ => {}
        }
    }

    fn request_close(&mut self) {
        self.handle.close();
        Application::global().quit();
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    let mut file_menu = Menu::new();
    file_menu.add_item(
        QUIT_MENU_ID,
        "E&xit",
        Some(&HotKey::new(SysMods::Cmd, "q")),
        true,
        false,
    );
    let mut menubar = Menu::new();
    menubar.add_dropdown(Menu::new(), "Application", true);
    menubar.add_dropdown(file_menu, "&File", true);

    let app = Application::new().unwrap();
    let mut builder = WindowBuilder::new(app.clone());
    builder.set_handler(Box::new(MainState::new()));
    builder.set_title("App");
    builder.set_menu(menubar);
    let window = builder.build().unwrap();
    window.show();
    app.run(None);
}
