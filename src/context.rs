use druid_shell::piet::{Piet, PietText, RenderContext};
use druid_shell::WindowHandle;

#[derive(Clone)]
pub(crate) struct GlobalCtx {
    pub(crate) window: WindowHandle,
    pub(crate) text: PietText,
}

pub struct EventCtx {}

pub struct LayoutCtx {
    pub(crate) global_ctx: GlobalCtx,
}

impl LayoutCtx {
    pub fn text(&mut self) -> &mut PietText {
        &mut self.global_ctx.text
    }
}

pub struct PaintCtx<'a, 'c> {
    pub(crate) global_ctx: GlobalCtx,
    pub(crate) render_ctx: &'a mut Piet<'c>,
}

impl PaintCtx<'_, '_> {
    pub fn with_save(&mut self, f: impl FnOnce(&mut PaintCtx)) {
        if let Err(e) = self.render_ctx.save() {
            log::error!("Failed to save RenderContext: '{}'", e);
            return;
        }

        f(self);

        if let Err(e) = self.render_ctx.restore() {
            log::error!("Failed to restore RenderContext: '{}'", e);
        }
    }
}
