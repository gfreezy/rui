#[macro_use]
mod macros;
mod diagnostics;
mod render_object;
mod rendering;
mod widget;

use druid_shell::{
    piet::Piet, Application, HotKey, Menu, MouseEvent, SysMods, WinHandler, WindowBuilder,
    WindowHandle,
};
use render_object::{
    pipeline_owner::PipelineOwner,
    render_box::{HitTestResult, Size},
    render_object::{HitTestEntry, Offset, PointerEvent, RenderObject},
};
use tracing::metadata::LevelFilter;
use widget::text::RenderText;

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

    fn handle_pointer_event_immediately(&self, event: &PointerEvent) {
        tracing::debug!("--------------- hit test --------------");
        let mut hit_test_result = HitTestResult::new();
        let position = event.pos.into();
        self.root_view().hit_test(&mut hit_test_result, position);
        self.dispatch_event(event, &hit_test_result);
        self.handle.invalidate();
    }

    fn dispatch_event(&self, event: &PointerEvent, hit_test_result: &HitTestResult) {
        tracing::debug!("--------------- dispatch event --------------");
        for entry in hit_test_result.entries() {
            tracing::debug!("hit test entry: {:?}", entry.target());
            entry.target().handle_event(event.clone(), entry.clone());
        }
        // todo!("clean current");
    }
}

impl WinHandler for MainState {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
        let root_node =
            RenderObject::new_render_box(Box::new(RenderText::new("hello, world.".to_string())));
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

    fn mouse_up(&mut self, event: &PointerEvent) {
        self.handle_pointer_event_immediately(event);
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
