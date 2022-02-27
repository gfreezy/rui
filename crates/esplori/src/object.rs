pub trait Properties: Sized {
    type Widget: Widget<Self> + 'static;
}

pub struct Ctx {}

pub trait Widget<Props>: WidgetInterface {
    fn create(props: Props) -> Self;
    fn update(&mut self, ctx: &mut Ctx, props: Props);
}

pub trait WidgetInterface {
    fn paint(&mut self, ctx: &mut Ctx);
}
