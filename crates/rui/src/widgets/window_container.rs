use std::time::Instant;

use druid_shell::{
    kurbo::{Point, Rect, Size, Vec2},
    piet::{Color, PaintBrush, RenderContext},
};

use crate::{
    box_constraints::BoxConstraints,
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::RenderObjectInterface,
    perf::FPSCounter,
    text::layout::TextLayout,
    tree::Children,
};

pub(crate) struct WindowContainer {
    fps_counter: FPSCounter,
}

const DEBUG: bool = false;

impl RenderObjectInterface for WindowContainer {
    fn sized_by_parent(&self) -> bool {
        true
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        let instant = Instant::now();

        for child in children {
            child.event(ctx, event);
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer event took {} us",
                instant.elapsed().as_micros()
            );
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children) {
        let instant = Instant::now();

        for child in children {
            child.lifecycle(ctx, event)
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer lifecycle took {} us",
                instant.elapsed().as_micros()
            );
        }
    }

    fn dry_layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        let instant = Instant::now();

        let mut size = Size::ZERO;
        for child in children {
            let child_size = child.dry_layout_box(ctx, bc);
            size = Size::new(
                child_size.width.max(size.width),
                child_size.height.max(size.height),
            );
        }
        if DEBUG {
            tracing::debug!(
                "windowcontainer dry_layout_box took {} us",
                instant.elapsed().as_micros()
            );
        }
        size
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        let instant = Instant::now();
        for child in children {
            let child_size = child.layout_box(ctx, bc, false);
            child.set_origin(ctx, Point::ZERO);
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer layout_box took {} us",
                instant.elapsed().as_micros()
            );
        }
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        let instant = Instant::now();

        for child in children {
            child.paint(ctx)
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer paint took {} us",
                instant.elapsed().as_micros()
            );
        }
        let fps = self.fps_counter.tick();
        draw_fps(fps, ctx.size(), ctx);
    }
}

fn draw_fps(fps: usize, window_size: Size, paint_ctx: &mut PaintCtx) {
    let mut layout: TextLayout<String> = TextLayout::from_text(format!("{}", fps));
    layout.rebuild_if_needed(&mut paint_ctx.text());
    let text_size = layout.size();
    let win_size = window_size;
    let origin = Point::new(win_size.width - text_size.width, 0.);
    let text_rect = Rect::from_origin_size(origin, text_size) - Vec2::new(5., 0.);
    let bg_rect = text_rect.inset(5.);
    paint_ctx.fill(
        bg_rect,
        &PaintBrush::Color(Color::from_hex_str("#fff").unwrap()),
    );
    paint_ctx.draw_text(layout.layout().unwrap(), text_rect.origin());
}

impl WindowContainer {
    pub fn new() -> Self {
        Self {
            fps_counter: FPSCounter::new(),
        }
    }
}
